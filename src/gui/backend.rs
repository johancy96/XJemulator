use super::types::{AxisSlot, BtnSlot, RawCapture};
use crate::mapper::{parse_abs, parse_key, Mapper};
use crate::virtual_device::VirtualXbox360;
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// --- Estándares de Ingeniería de Entrada (Baja Latencia 500Hz + Sensores Hall/Continuos) ---
const HW_DEADZONE_PERCENT: f32 = 0.15;   // 15% de compuerta física para retorno al reposo
const AXIS_START_TRACK_RATIO: f32 = 0.18; // 18% de recorrido para iniciar lectura continua
const EMU_POLLING_MS: u64 = 2;            // Latencia de emulación (500Hz)
const READER_POLLING_MS: u64 = 2;         // Latencia de captura/calibración (500Hz)
const BACKOFF_MS: u64 = 80;              // Retraso entre reintentos de grab
const MAX_GRAB_ATTEMPTS: usize = 6;       // Intentos de secuestro exclusivos

#[derive(Debug, Clone, Copy)]
pub(crate) struct AxisMovement {
    pub axis: evdevil::event::Abs,
    pub positive: bool,
    pub delta: i32,
    pub travel_ratio: f32,
    pub is_digital: bool,
}

/// Registra errores técnicos en el directorio de telemetría sin molestar al usuario.
fn log_telemetry(msg: &str) {
    let path = crate::paths::AppPaths::telemetry_dir().join("engine.log");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        use std::io::Write;
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(file, "[{}] {}", now, msg);
    }
}

/// Determina si un eje ha regresado a su estado de reposo.
/// Si ya conocemos el punto de reposo exacto (`known_rest`), verifica proximidad a ese punto
/// (evitando falsos centros en `0` cuando un gatillo `-32768..32767` va a mitad de recorrido).
pub(crate) fn is_hardware_centered(
    code: evdevil::event::Abs,
    val: i32,
    ranges: &HashMap<evdevil::event::Abs, (i32, i32)>,
    known_rest: Option<i32>,
) -> bool {
    if let Some(&(min, max)) = ranges.get(&code) {
        let full_range = (max - min).abs() as f32;
        if full_range == 0.0 {
            return true;
        }

        if full_range <= 2.0 {
            let rest = known_rest.unwrap_or(0);
            return val == rest;
        }

        if let Some(rest) = known_rest {
            let max_travel = (max - rest).abs().max((rest - min).abs()).max(1) as f32;
            return ((val - rest).abs() as f32) <= max_travel * HW_DEADZONE_PERCENT;
        }

        let deadzone = full_range * HW_DEADZONE_PERCENT;
        let center = (min + max) / 2;

        // Comprobación Tri-Modal inicial antes de conocer `resting`
        let near_min = ((val - min).abs() as f32) <= deadzone;
        let near_max = ((val - max).abs() as f32) <= deadzone;
        let near_center = ((val - center).abs() as f32) <= deadzone;

        return near_min || near_max || near_center;
    }
    val.abs() < 2000 || (val - 128).abs() < 20 || (val - 255).abs() < 20
}

/// Detecta movimiento en ejes tanto continuos (Hall Effect / analógicos) como discretos (HAT / 0-1),
/// calculando la excursión real respecto al límite físico en la dirección del movimiento.
pub(crate) fn detect_axis_movement(
    axis_values: &HashMap<evdevil::event::Abs, i32>,
    resting: &HashMap<evdevil::event::Abs, i32>,
    exclude_axes: &HashSet<evdevil::event::Abs>,
    ranges: &HashMap<evdevil::event::Abs, (i32, i32)>,
    expected_same_axis: Option<(evdevil::event::Abs, bool)>,
) -> Option<AxisMovement> {
    let mut best: Option<AxisMovement> = None;

    for (&code, &cur) in axis_values {
        if exclude_axes.contains(&code) {
            continue;
        }

        if let Some((req_axis, req_pos)) = expected_same_axis {
            if code != req_axis {
                continue;
            }
            let rest = resting
                .get(&code)
                .copied()
                .unwrap_or_else(|| ranges.get(&code).map(|&(min, max)| (min + max) / 2).unwrap_or(0));
            if cur == rest || (cur > rest) != req_pos {
                continue;
            }
        }

        let (min, max) = ranges.get(&code).copied().unwrap_or((-32768, 32767));
        let full_range = (max - min).abs();
        if full_range == 0 {
            continue;
        }

        let rest = resting
            .get(&code)
            .copied()
            .unwrap_or_else(|| (min + max) / 2);

        let signed_delta = cur - rest;
        let delta = signed_delta.abs();
        if delta == 0 {
            continue;
        }

        let positive = signed_delta > 0;
        let is_hat_or_digital = full_range <= 2 || format!("{:?}", code).contains("HAT");

        // Distancia desde el reposo hasta el tope teórico en el sentido del movimiento
        let directional_max = if positive {
            (max - rest).abs()
        } else {
            (rest - min).abs()
        }
        .max(1);

        let travel_ratio = (delta as f32 / directional_max as f32).clamp(0.0, 1.0);
        let threshold = if is_hat_or_digital {
            0.45
        } else {
            AXIS_START_TRACK_RATIO
        };

        if travel_ratio >= threshold {
            let replace = match best {
                None => true,
                Some(prev) => travel_ratio > prev.travel_ratio,
            };
            if replace {
                best = Some(AxisMovement {
                    axis: code,
                    positive,
                    delta,
                    travel_ratio,
                    is_digital: is_hat_or_digital,
                });
            }
        }
    }

    best
}

pub(crate) fn is_all_centered(
    axis_values: &HashMap<evdevil::event::Abs, i32>,
    ranges: &HashMap<evdevil::event::Abs, (i32, i32)>,
    resting: &HashMap<evdevil::event::Abs, i32>,
) -> bool {
    for (&code, &cur) in axis_values {
        let known_rest = resting.get(&code).copied();
        if !is_hardware_centered(code, cur, ranges, known_rest) {
            return false;
        }
    }
    true
}

pub(crate) fn scan_profiles() -> Vec<String> {
    let mut v = Vec::new();
    let dir = crate::paths::AppPaths::profiles_dir();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            if e.file_type().map(|t| t.is_file()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(".toml") {
                    v.push(name);
                }
            }
        }
    }
    v.sort();
    v
}

pub(crate) fn load_best_profile(profiles: &[String]) -> (Mapper, Option<String>) {
    for name in profiles {
        let path = crate::paths::AppPaths::profile_path(name);
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(p) = toml::from_str::<crate::mapper::MappingProfile>(&content) {
                if let Ok(m) = Mapper::from_profile(&p) {
                    return (m, Some(name.clone()));
                }
            }
        }
    }
    (Mapper::identity(), None)
}

pub(crate) fn generate_profile_toml_with_ids(
    btns: &[BtnSlot],
    axes: &[AxisSlot],
    name: &str,
    resting: &HashMap<evdevil::event::Abs, i32>,
    vid: Option<u16>,
    pid: Option<u16>,
) -> String {
    let safe_name = crate::paths::AppPaths::sanitize_profile_name(name);
    let mut t = format!(
        "name = {:?}\ndescription = \"Calibrado con XJEmulator\"\n",
        safe_name
    );

    if let Some(v) = vid {
        t.push_str(&format!("vendor_id = {}\n", v));
    }
    if let Some(p) = pid {
        t.push_str(&format!("product_id = {}\n", p));
    }
    t.push('\n');

    // Separar slots de ejes que provienen de ejes analógicos/HAT vs botones digitales (mandos antiguos)
    let mut axes_map: HashMap<&str, Vec<&AxisSlot>> = HashMap::new();
    let mut digital_btn_axes: Vec<&AxisSlot> = Vec::new();

    for ax in axes {
        if let Some(ref src) = ax.source {
            if parse_key(src).is_ok() {
                digital_btn_axes.push(ax);
            } else {
                axes_map.entry(ax.xbox_axis).or_default().push(ax);
            }
        }
    }

    for (xbox_name, slots) in axes_map {
        let base = slots[0];
        if let Some(ref src) = base.source {
            // Zona muerta calibrada para conservar progresividad en sensores Hall Effect y potenciómetros
            let dz = if xbox_name.contains("HAT") {
                0
            } else if xbox_name.ends_with('Z') {
                4
            } else {
                3000
            };

            let abs_code = parse_abs(src).unwrap_or(evdevil::event::Abs::X);
            let center = resting.get(&abs_code).copied().unwrap_or(0);
            let final_invert = base.invert;

            // Para que el usuario alcance el 100% de velocidad en ambos extremos del stick/gatillo,
            // usamos el menor de los topes máximos medidos (con un 2% de margen de saturación en el tope).
            let valid_peaks: Vec<i32> = slots
                .iter()
                .filter_map(|s| s.max_val)
                .filter(|&v| v > 0)
                .collect();

            let effective_peak = if valid_peaks.len() >= 2 {
                valid_peaks.iter().copied().min()
            } else {
                valid_peaks.first().copied()
            };

            let mut scale = 1.0;
            if let Some(mv) = effective_peak {
                if mv > 0 {
                    let target_max = if xbox_name.contains("HAT") {
                        1.0
                    } else if xbox_name == "ABS_Z" || xbox_name == "ABS_RZ" {
                        255.0
                    } else {
                        32767.0
                    };
                    let saturated_peak = if xbox_name.contains("HAT") {
                        mv as f32
                    } else {
                        (mv as f32 * 0.98).max(1.0)
                    };
                    scale = target_max / saturated_peak;
                    if scale > 0.98 && scale < 1.02 {
                        scale = 1.0;
                    }
                }
            }

            t.push_str("[[axes]]\n");
            t.push_str(&format!("source = {:?}\n", src));
            t.push_str(&format!("target = {:?}\n", xbox_name));
            t.push_str(&format!("invert = {}\n", final_invert));
            t.push_str(&format!("deadzone = {}\n", dz));
            t.push_str(&format!("scale = {:.4}\n", scale));
            t.push_str(&format!("center = {}\n\n", center));
        }
    }

    for btn in btns {
        if let Some(ref src) = btn.source {
            t.push_str("[[buttons]]\n");
            t.push_str(&format!("source = {:?}\n", src));
            t.push_str(&format!("target = {:?}\n\n", btn.xbox_key));
        }
    }

    // Mapeo híbrido para mandos antiguos cuyos gatillos o cruceta son botones (0 y 1)
    for dax in digital_btn_axes {
        if let Some(ref src) = dax.source {
            let active_val = if dax.xbox_axis == "ABS_Z" || dax.xbox_axis == "ABS_RZ" {
                255
            } else if dax.xbox_axis.contains("HAT") {
                if dax.positive_expected {
                    1
                } else {
                    -1
                }
            } else if dax.positive_expected {
                32767
            } else {
                -32768
            };

            t.push_str("[[button_axes]]\n");
            t.push_str(&format!("source = {:?}\n", src));
            t.push_str(&format!("target = {:?}\n", dax.xbox_axis));
            t.push_str(&format!("active_value = {}\n\n", active_val));
        }
    }

    t
}

pub(crate) fn raw_reader_loop(
    path: String,
    capture: Arc<Mutex<RawCapture>>,
    running: Arc<AtomicBool>,
    egui_ctx: Option<egui::Context>,
) {
    use evdevil::event::{EventKind, KeyState};

    let device = match evdevil::Evdev::open(&path) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("raw_reader: No se pudo abrir {}: {}", path, e);
            return;
        }
    };

    let _ = device.set_nonblocking(true);
    let _ = device.grab();
    tracing::info!("raw_reader: Iniciado a 500Hz ({})", path);

    let mut buf = [evdevil::event::InputEvent::zeroed(); 64];

    while running.load(Ordering::Relaxed) {
        match device.read_events(&mut buf) {
            Ok(count) => {
                if count == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(READER_POLLING_MS));
                    continue;
                }
                let mut had_input = false;
                if let Ok(mut cap) = capture.lock() {
                    for ev in &buf[..count] {
                        match ev.kind() {
                            EventKind::Key(ke) => {
                                let key = ke.key();
                                if ke.state() == KeyState::PRESSED {
                                    if cap.pressed_keys.insert(key) {
                                        cap.key_queue.push_back(key);
                                    }
                                } else {
                                    cap.pressed_keys.remove(&key);
                                }
                                had_input = true;
                            }
                            EventKind::Abs(ae) => {
                                cap.axis_values.insert(ae.abs(), ae.value());
                                had_input = true;
                            }
                            _ => {}
                        }
                    }
                }
                if had_input {
                    if let Some(ref ctx) = egui_ctx {
                        ctx.request_repaint();
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(READER_POLLING_MS));
            }
            Err(e) => {
                log_telemetry(&format!("Error crítico en lector: {}", e));
                break;
            }
        }
    }
    let _ = device.ungrab();
    tracing::info!("raw_reader: Finalizado y liberado ({})", path);
}

pub(crate) fn emulation_loop(
    path: String,
    capture: Arc<Mutex<RawCapture>>,
    running: Arc<AtomicBool>,
    mapper: Mapper,
    vx: VirtualXbox360,
) {
    use evdevil::event::{AbsEvent, EventKind, InputEvent, KeyEvent, KeyState};

    let device = match evdevil::Evdev::open(&path) {
        Ok(d) => d,
        Err(e) => {
            log_telemetry(&format!("emulation_loop: Error al abrir {}: {}", path, e));
            running.store(false, Ordering::SeqCst);
            return;
        }
    };

    let _ = device.set_nonblocking(true);

    // Secuestro exclusivo (EVIOCGRAB) con reintentos para evitar doble entrada
    let mut grab_ok = false;
    for i in 0..MAX_GRAB_ATTEMPTS {
        if device.grab().is_ok() {
            grab_ok = true;
            break;
        }
        tracing::warn!(
            "emulation_loop: Reintento de secuestro {}/{}...",
            i + 1,
            MAX_GRAB_ATTEMPTS
        );
        std::thread::sleep(std::time::Duration::from_millis(BACKOFF_MS));
    }

    if !grab_ok {
        log_telemetry(
            "CRÍTICO: Acceso denegado al hardware. Otro proceso mantiene EVIOCGRAB activo.",
        );
        running.store(false, Ordering::SeqCst);
        return;
    }

    // Espera de asentamiento de SDL2/udev en segundo plano sin congelar la UI
    vx.wait_for_sdl2_detection(&running);
    if !running.load(Ordering::Relaxed) {
        let _ = device.ungrab();
        return;
    }

    tracing::info!(
        "emulation_loop: Mando físico capturado con éxito. Emulación activa a 500Hz."
    );

    let mut buf = [InputEvent::zeroed(); 64];
    let mut batch_out: Vec<InputEvent> = Vec::with_capacity(64);

    while running.load(Ordering::Relaxed) {
        match device.read_events(&mut buf) {
            Ok(count) => {
                if count == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(EMU_POLLING_MS));
                    continue;
                }

                batch_out.clear();

                // 1. Transformar toda la ráfaga y emitir al kernel FUERA del Mutex de UI
                for ev in &buf[..count] {
                    match ev.kind() {
                        EventKind::Abs(ae) => {
                            if let Some((target, mapped)) = mapper.map_axis(ae.abs(), ae.value()) {
                                batch_out.push(AbsEvent::new(target, mapped).into());
                            }
                        }
                        EventKind::Key(ke) => {
                            let pressed = ke.state() == KeyState::PRESSED;
                            if let Some(target) = mapper.map_button(ke.key()) {
                                batch_out.push(KeyEvent::new(target, ke.state()).into());
                            }
                            if let Some((target_abs, mapped_val)) =
                                mapper.map_button_to_axis(ke.key(), pressed)
                            {
                                batch_out.push(AbsEvent::new(target_abs, mapped_val).into());
                            }
                        }
                        _ => {}
                    }
                }

                if !batch_out.is_empty() {
                    if let Err(e) = vx.write_batch(&batch_out) {
                        log_telemetry(&format!("Error de escritura uinput: {}", e));
                        break;
                    }

                    // 2. Actualizar telemetría de UI con try_lock para jamás bloquear el hilo de 500Hz
                    if let Ok(mut cap) = capture.try_lock() {
                        for ev in &batch_out {
                            match ev.kind() {
                                EventKind::Abs(ae) => {
                                    cap.axis_values.insert(ae.abs(), ae.value());
                                }
                                EventKind::Key(ke) => {
                                    if ke.state() == KeyState::PRESSED {
                                        cap.pressed_keys.insert(ke.key());
                                    } else {
                                        cap.pressed_keys.remove(&ke.key());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(EMU_POLLING_MS));
            }
            Err(e) => {
                log_telemetry(&format!("Error fatal en bucle de emulación: {}", e));
                break;
            }
        }
    }

    running.store(false, Ordering::SeqCst);
    let _ = device.ungrab();
    tracing::info!("emulation_loop: Mando físico restaurado.");
}
