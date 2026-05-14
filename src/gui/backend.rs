use super::types::{AxisSlot, BtnSlot, RawCapture};
use crate::mapper::{parse_abs, Mapper};
use crate::virtual_device::VirtualXbox360;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

// --- Estándares de Ingeniería de Entrada (Gold Edition) ---
const HW_DEADZONE_PERCENT: f32 = 0.20;  // 20% de zona muerta física (Hardware Gate)
const AXIS_CONFIRM_PERCENT: f32 = 0.35;  // 35% de recorrido para confirmar intención
const EMU_POLLING_MS: u64 = 2;           // Latencia de emulación (500Hz)
const READER_POLLING_MS: u64 = 10;       // Latencia de UI (100Hz)
const BACKOFF_MS: u64 = 150;            // Retraso entre reintentos de grab
const MAX_GRAB_ATTEMPTS: usize = 5;      // Intentos de secuestro exclusivos

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
    ranges: &HashMap<evdevil::event::Abs, (i32, i32)>
) -> bool {
    if let Some(&(min, max)) = ranges.get(&code) {
        let full_range = (max - min).abs() as f32;
        if full_range == 0.0 { return true; }
        
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
        if exclude_axes.contains(&code) { continue; }
        
        // Usar resting capturado o centro de hardware
        let rest = resting.get(&code).copied()
            .unwrap_or_else(|| ranges.get(&code).map(|&(min, max)| (min + max) / 2).unwrap_or(0));
            
        let delta = (cur - rest).abs() as f32;
        let (min, max) = ranges.get(&code).copied().unwrap_or((-32768, 32767));
        let full_range = (max - min).abs() as f32;
        if full_range == 0.0 { continue; }

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
    ranges: &HashMap<evdevil::event::Abs, (i32, i32)>
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
    let mut t = format!(
        "name = {:?}\ndescription = \"Calibrado con XJEmulator\"\n",
        name
    );

    if let Some(v) = vid { t.push_str(&format!("vendor_id = {}\n", v)); }
    if let Some(p) = pid { t.push_str(&format!("product_id = {}\n", p)); }
    t.push_str("\n");

    // Agrupar calibraciones por eje de destino (Xbox)
    let mut axes_map: HashMap<&str, Vec<&AxisSlot>> = HashMap::new();
    for ax in axes {
        if ax.source.is_some() {
            axes_map.entry(ax.xbox_axis).or_default().push(ax);
        }
    }

    for (xbox_name, slots) in axes_map {
        // Usamos el primero como base, pero validaremos con los demás
        let base = slots[0];
        if let Some(ref src) = base.source {
            let dz = if xbox_name.contains("HAT") || xbox_name.ends_with('Z') { 0 } else { 8000 };
            let abs_code = parse_abs(src).unwrap_or(evdevil::event::Abs::X);
            let center = resting.get(&abs_code).copied().unwrap_or(0);

            // Determinar inversión final (si cualquier slot indica invertido, se respeta)
            let mut final_invert = false;
            for s in slots {
                if s.invert { final_invert = true; }
            }

            let mut scale = 1.0;
            // Podríamos promediar escalas si hubiera varios max_val, por ahora usamos el primero
            if let Some(mv) = base.max_val {
                if mv > 0 {
                    let target_max = if xbox_name.contains("HAT") { 1.0 } 
                                     else if xbox_name == "ABS_Z" || xbox_name == "ABS_RZ" { 255.0 } 
                                     else { 32767.0 };
                    scale = target_max / (mv as f32);
                    if scale > 0.95 && scale < 1.05 { scale = 1.0; }
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
) {
    use evdevil::event::{EventKind, KeyState};

    let device = match evdevil::Evdev::open(&path) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("raw_reader: No se pudo abrir {}: {}", path, e);
            return;
        }
    };
    
    // Configurar modo NO BLOQUEANTE para poder salir del loop sin esperar eventos
    let _ = device.set_nonblocking(true);
    let _ = device.grab();
    tracing::info!("raw_reader: Iniciado en modo asíncrono ({})", path);

    let mut buf = [evdevil::event::InputEvent::zeroed(); 64];

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        match device.read_events(&mut buf) {
            Ok(count) => {
                if count == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(READER_POLLING_MS));
                    continue;
                }
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
                            }
                            EventKind::Abs(ae) => {
                                cap.axis_values.insert(ae.abs(), ae.value());
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(READER_POLLING_MS * 2));
            }
            Err(e) => {
                log_telemetry(&format!("Error crítico en lector: {}", e));
                break;
            }
        }
    }
    let _ = device.ungrab();
    tracing::info!("raw_reader: Finalizado y liberado");
}

pub(crate) fn emulation_loop(
    path: String,
    capture: Arc<Mutex<RawCapture>>,
    running: Arc<AtomicBool>,
    mapper: Mapper,
    vx: VirtualXbox360,
) {
    use evdevil::event::{AbsEvent, EventKind, KeyEvent, KeyState};

    let device = match evdevil::Evdev::open(&path) {
        Ok(d) => d,
        Err(e) => {
            log_telemetry(&format!("emulation_loop: Error al abrir {}: {}", path, e));
            running.store(false, std::sync::atomic::Ordering::SeqCst);
            return;
        }
    };

    let _ = device.set_nonblocking(true);

    // SECUESTRO MANDATORIO CON REINTENTOS (Senior Backoff)
    let mut grab_ok = false;
    for i in 0..MAX_GRAB_ATTEMPTS {
        if device.grab().is_ok() {
            grab_ok = true;
            break;
        }
        tracing::warn!("emulation_loop: Reintento de secuestro {}/{}...", i + 1, MAX_GRAB_ATTEMPTS);
        std::thread::sleep(std::time::Duration::from_millis(BACKOFF_MS));
    }

    if !grab_ok {
        log_telemetry("CRÍTICO: Acceso denegado al hardware. Probablemente Steam u otro driver tiene el grab activo.");
        running.store(false, std::sync::atomic::Ordering::SeqCst);
        return;
    }

    tracing::info!("emulation_loop: Mando físico ocultado con éxito. Emulación iniciada.");

    let mut buf = [evdevil::event::InputEvent::zeroed(); 64];

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        match device.read_events(&mut buf) {
            Ok(count) => {
                if count == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(EMU_POLLING_MS));
                    continue;
                }
                
                // Procesamos la ráfaga
                if let Ok(mut cap) = capture.lock() {
                    for ev in &buf[..count] {
                        let mut single_out = Vec::with_capacity(2);
                        
                        match ev.kind() {
                            EventKind::Abs(ae) => {
                                if let Some((target, mapped)) = mapper.map_axis(ae.abs(), ae.value()) {
                                    cap.axis_values.insert(target, mapped);
                                    single_out.push(AbsEvent::new(target, mapped).into());
                                }
                            }
                            EventKind::Key(ke) => {
                                if let Some(target) = mapper.map_button(ke.key()) {
                                    if ke.state() == KeyState::PRESSED {
                                        cap.pressed_keys.insert(target);
                                    } else {
                                        cap.pressed_keys.remove(&target);
                                    }
                                    single_out.push(KeyEvent::new(target, ke.state()).into());
                                }
                            }
                            _ => {}
                        }

                        // Si hay un evento mapeado, lo enviamos inmediatamente con su propio reporte
                        if !single_out.is_empty() {
                            single_out.push(evdevil::event::SynEvent::new(evdevil::event::Syn::REPORT).into());
                            if let Err(e) = vx.write_batch(&single_out) {
                                log_telemetry(&format!("Error de escritura uinput: {}", e));
                                break;
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
    let _ = device.ungrab();
    tracing::info!("emulation_loop: Mando físico restaurado.");
}
