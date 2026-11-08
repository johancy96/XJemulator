use super::types::{AxisSlot, BtnSlot, RawCapture};
use crate::mapper::{parse_abs, Mapper};
use crate::virtual_device::VirtualXbox360;
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// --- Estándares de Ingeniería de Entrada (Baja Latencia 500Hz) ---
const HW_DEADZONE_PERCENT: f32 = 0.20;  // 20% de zona muerta física (Hardware Gate)
const AXIS_CONFIRM_PERCENT: f32 = 0.35;  // 35% de recorrido para confirmar intención
const EMU_POLLING_MS: u64 = 2;           // Latencia de emulación (500Hz)
const READER_POLLING_MS: u64 = 2;        // Latencia de captura/calibración (500Hz)
const BACKOFF_MS: u64 = 80;             // Retraso entre reintentos de grab
const MAX_GRAB_ATTEMPTS: usize = 6;      // Intentos de secuestro exclusivos

/// Registra errores técnicos en el directorio de telemetría sin molestar al usuario.
fn log_telemetry(msg: &str) {
    let path = crate::paths::AppPaths::telemetry_dir().join("engine.log");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        use std::io::Write;
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(file, "[{}] {}", now, msg);
    }
}

/// Determina si un valor está en un estado de "Reposo de Hardware".
/// Un eje es neutral si está cerca de su Mínimo, su Máximo o su Centro Matemático.
/// Esto cubre Joysticks (Centro) y Gatillos (Mín/Máx).
pub(crate) fn is_hardware_centered(
    code: evdevil::event::Abs,
    val: i32,
    ranges: &HashMap<evdevil::event::Abs, (i32, i32)>,
) -> bool {
    if let Some(&(min, max)) = ranges.get(&code) {
        let full_range = (max - min).abs() as f32;
        if full_range == 0.0 {
            return true;
        }

        let deadzone = full_range * HW_DEADZONE_PERCENT;
        let center = (min + max) / 2;

        // Comprobación Tri-Modal:
        let near_min = ((val - min).abs() as f32) < deadzone;
        let near_max = ((val - max).abs() as f32) < deadzone;
        let near_center = ((val - center).abs() as f32) < deadzone;

        return near_min || near_max || near_center;
    }
    // Fallback estándar
    val.abs() < 2000 || (val - 128).abs() < 20 || (val - 255).abs() < 20
}

pub(crate) fn detect_axis_movement(
    axis_values: &HashMap<evdevil::event::Abs, i32>,
    resting: &HashMap<evdevil::event::Abs, i32>,
    exclude_axes: &HashSet<evdevil::event::Abs>,
    ranges: &HashMap<evdevil::event::Abs, (i32, i32)>,
) -> Option<(evdevil::event::Abs, bool)> {
    let mut best_axis = None;
    let mut max_p = 0.0f32;

    for (&code, &cur) in axis_values {
        if exclude_axes.contains(&code) {
            continue;
        }

        // Usar resting capturado o centro de hardware
        let rest = resting
            .get(&code)
            .copied()
            .unwrap_or_else(|| ranges.get(&code).map(|&(min, max)| (min + max) / 2).unwrap_or(0));

        let delta = (cur - rest).abs() as f32;
        let (min, max) = ranges.get(&code).copied().unwrap_or((-32768, 32767));
        let full_range = (max - min).abs() as f32;
        if full_range == 0.0 {
            continue;
        }

        let percent = delta / full_range;
        let is_hat = format!("{:?}", code).contains("HAT");
        let threshold = if is_hat { 0.45 } else { AXIS_CONFIRM_PERCENT };

        if percent > threshold && percent > max_p {
            max_p = percent;
            best_axis = Some((code, cur > rest));
        }
    }
    best_axis
}

pub(crate) fn is_all_centered(
    axis_values: &HashMap<evdevil::event::Abs, i32>,
    ranges: &HashMap<evdevil::event::Abs, (i32, i32)>,
) -> bool {
    for (&code, &cur) in axis_values {
        if !is_hardware_centered(code, cur, ranges) {
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

    // Agrupar calibraciones por eje de destino (Xbox) manteniendo orden determinista
    let mut axes_map: HashMap<&str, Vec<&AxisSlot>> = HashMap::new();
    for ax in axes {
        if ax.source.is_some() {
            axes_map.entry(ax.xbox_axis).or_default().push(ax);
        }
    }

    for (xbox_name, slots) in axes_map {
        let base = slots[0];
        if let Some(ref src) = base.source {
            let dz = if xbox_name.contains("HAT") || xbox_name.ends_with('Z') {
                0
            } else {
                8000
            };
            let abs_code = parse_abs(src).unwrap_or(evdevil::event::Abs::X);
            let center = resting.get(&abs_code).copied().unwrap_or(0);

            // Usamos la inversión detectada en el primer slot calibrado para ese eje
            let final_invert = base.invert;

            // Tomamos la máxima excursión registrada entre las direcciones calibradas del mismo eje
            let max_delta = slots.iter().filter_map(|s| s.max_val).max();
            let mut scale = 1.0;
            if let Some(mv) = max_delta {
                if mv > 0 {
                    let target_max = if xbox_name.contains("HAT") {
                        1.0
                    } else if xbox_name == "ABS_Z" || xbox_name == "ABS_RZ" {
                        255.0
                    } else {
                        32767.0
                    };
                    scale = target_max / (mv as f32);
                    if scale > 0.95 && scale < 1.05 {
                        scale = 1.0;
                    }
                }
            }

            t.push_str("[[axes]]\n");
            t.push_str(&format!("source = {:?}\n", src));
            t.push_str(&format!("target = {:?}\n", xbox_name));
            t.push_str(&format!("invert = {}\n", final_invert));
            t.push_str(&format!("deadzone = {}\n", dz));
            t.push_str(&format!("scale = {:.3}\n", scale));
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
    // Buffer reutilizable sin realocaciones en el bucle caliente
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
                            if let Some(target) = mapper.map_button(ke.key()) {
                                batch_out.push(KeyEvent::new(target, ke.state()).into());
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
