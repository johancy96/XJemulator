use super::types::{AxisSlot, BtnSlot, RawCapture};
use crate::mapper::{parse_abs, Mapper};
use crate::virtual_device::VirtualXbox360;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub(crate) fn calib_delta_threshold(abs: evdevil::event::Abs) -> i32 {
    use evdevil::event::Abs;
    match abs {
        Abs::HAT0X | Abs::HAT0Y | Abs::HAT1X | Abs::HAT1Y => 1,
        Abs::Z | Abs::RZ | Abs::GAS | Abs::BRAKE => 30,
        _ => 40,
    }
}

pub(crate) fn detect_axis_movement(
    axis_values: &HashMap<evdevil::event::Abs, i32>,
    resting: &HashMap<evdevil::event::Abs, i32>,
    exclude_axes: &HashSet<evdevil::event::Abs>,
) -> Option<(evdevil::event::Abs, bool)> {
    let mut best_code: Option<evdevil::event::Abs> = None;
    let mut best_delta = 0i32;
    let mut best_pos = true;

    for (&code, &cur) in axis_values {
        if exclude_axes.contains(&code) {
            continue;
        }
        let rest = resting.get(&code).copied().unwrap_or(0);
        let delta = cur - rest;
        let thr = calib_delta_threshold(code);
        if delta.abs() >= thr && delta.abs() > best_delta.abs() {
            best_code = Some(code);
            best_delta = delta;
            best_pos = delta > 0;
        }
    }

    best_code.map(|code| (code, best_pos))
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

    let mut written_axes: HashSet<&str> = HashSet::new();

    for ax in axes {
        if let Some(ref src) = ax.source {
            if written_axes.insert(ax.xbox_axis) {
                let dz = if ax.xbox_axis.contains("HAT") || ax.xbox_axis.ends_with('Z') {
                    0
                } else {
                    8000
                };
                let center = resting.get(&parse_abs(src).unwrap_or(evdevil::event::Abs::X)).copied().unwrap_or(0);

                let mut scale = 1.0;
                if let Some(mv) = ax.max_val {
                    if mv > 0 {
                        let target_max = if ax.xbox_axis.contains("HAT") {
                            1.0
                        } else if ax.xbox_axis == "ABS_Z" || ax.xbox_axis == "ABS_RZ" {
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
                t.push_str(&format!("target = {:?}\n", ax.xbox_axis));
                t.push_str(&format!("invert = {}\n", ax.invert));
                t.push_str(&format!("deadzone = {}\n", dz));
                t.push_str(&format!("scale = {:.3}\n", scale));
                t.push_str(&format!("center = {}\n\n", center));
            }
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
            tracing::error!("raw_reader: {}: {}", path, e);
            return;
        }
    };
    let _ = device.grab();
    tracing::info!("raw_reader: iniciado en {}", path);

    let mut buf = [evdevil::event::InputEvent::zeroed(); 64];

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        let count = match device.read_events(&mut buf) {
            Ok(n) => n,
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(2));
                continue;
            }
        };
        if count == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }

        let Ok(mut cap) = capture.lock() else {
            continue;
        };

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
    tracing::info!("raw_reader: finalizado en {}", path);
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
            tracing::error!("emulation_loop: {}: {}", path, e);
            return;
        }
    };
    let _ = device.grab();
    tracing::info!("emulation_loop: iniciado");

    let mut buf = [evdevil::event::InputEvent::zeroed(); 64];

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        let count = match device.read_events(&mut buf) {
            Ok(n) => n,
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(2));
                continue;
            }
        };
        if count == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }

        let mut out: Vec<InputEvent> = Vec::with_capacity(count + 1);

        if let Ok(mut cap) = capture.lock() {
            for ev in &buf[..count] {
                match ev.kind() {
                    EventKind::Abs(ae) => {
                        if let Some((target, mapped)) = mapper.map_axis(ae.abs(), ae.value()) {
                            cap.axis_values.insert(target, mapped);
                            out.push(AbsEvent::new(target, mapped).into());
                        }
                    }
                    EventKind::Key(ke) => {
                        if let Some(target) = mapper.map_button(ke.key()) {
                            if ke.state() == KeyState::PRESSED {
                                cap.pressed_keys.insert(target);
                            } else {
                                cap.pressed_keys.remove(&target);
                            }
                            out.push(KeyEvent::new(target, ke.state()).into());
                        }
                    }
                    _ => {}
                }
            }
        }

        if !out.is_empty() {
            // Sincronización explícita del Kernel para 0 latencia visual
            out.push(evdevil::event::SynEvent::new(evdevil::event::Syn::REPORT).into());
            if let Err(e) = vx.write_batch(&out) {
                tracing::error!("Error emulando: {}", e);
                break;
            }
        }
    }
    tracing::info!("emulation_loop: finalizado");
}
