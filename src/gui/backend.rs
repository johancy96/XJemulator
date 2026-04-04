use super::types::{AxisSlot, BtnSlot, RawCapture};
use crate::mapper::Mapper;
use crate::virtual_device::VirtualXbox360;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub(crate) fn calib_delta_threshold(name: &str) -> i32 {
    if name.contains("HAT") {
        1
    } else if name.ends_with('Z') || name.contains("GAS") || name.contains("BRAKE") {
        30
    } else {
        40
    }
}

pub(crate) fn detect_axis_movement(
    axis_values: &HashMap<String, i32>,
    resting: &HashMap<String, i32>,
    exclude_axes: &HashSet<String>,
) -> Option<(String, bool)> {
    let mut best_name = String::new();
    let mut best_delta = 0i32;
    let mut best_pos = true;

    for (name, &cur) in axis_values {
        if exclude_axes.contains(name) {
            continue;
        }
        let rest = resting.get(name).copied().unwrap_or(0);
        let delta = cur - rest;
        let thr = calib_delta_threshold(name);
        if delta.abs() >= thr && delta.abs() > best_delta.abs() {
            best_name = name.clone();
            best_delta = delta;
            best_pos = delta > 0;
        }
    }

    if best_name.is_empty() {
        None
    } else {
        Some((best_name, best_pos))
    }
}

pub(crate) fn scan_profiles() -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(entries) = std::fs::read_dir(".") {
        for e in entries.flatten() {
            if e.file_type().map(|t| t.is_file()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(".toml") && name != "Cargo.toml" && name != "config.toml" {
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
        if let Ok(content) = std::fs::read_to_string(name) {
            if let Ok(p) = toml::from_str::<crate::mapper::MappingProfile>(&content) {
                if let Ok(m) = Mapper::from_profile(&p) {
                    return (m, Some(name.clone()));
                }
            }
        }
    }
    (Mapper::identity(), None)
}

pub(crate) fn generate_profile_toml(
    btns: &[BtnSlot],
    axes: &[AxisSlot],
    name: &str,
    resting: &HashMap<String, i32>,
) -> String {
    let mut t = format!(
        "name = {:?}\ndescription = \"Calibrado con XJEmulator\"\n\n",
        name
    );

    let mut written_axes: HashSet<&str> = HashSet::new();

    for ax in axes {
        if let Some(ref src) = ax.source {
            if written_axes.insert(ax.xbox_axis) {
                let dz = if ax.xbox_axis.contains("HAT") || ax.xbox_axis.ends_with('Z') {
                    0
                } else {
                    8000
                };
                let center = resting.get(src).copied().unwrap_or(0);

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
                    let name = format!("{:?}", ke.key());
                    if ke.state() == KeyState::PRESSED {
                        if cap.pressed_keys.insert(name.clone()) {
                            cap.key_queue.push_back(name);
                        }
                    } else {
                        cap.pressed_keys.remove(&name);
                    }
                }
                EventKind::Abs(ae) => {
                    let name = format!("{:?}", ae.abs());
                    let value = ae.value();
                    cap.axis_values.insert(name, value);
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

        let mut out: Vec<InputEvent> = Vec::new();

        if let Ok(mut cap) = capture.lock() {
            for ev in &buf[..count] {
                match ev.kind() {
                    EventKind::Abs(ae) => {
                        if let Some((target, mapped)) = mapper.map_axis(ae.abs(), ae.value()) {
                            let tname = format!("{:?}", target);
                            cap.axis_values.insert(tname, mapped);
                            out.push(AbsEvent::new(target, mapped).into());
                        }
                    }
                    EventKind::Key(ke) => {
                        if let Some(target) = mapper.map_button(ke.key()) {
                            let tname = format!("{:?}", target);
                            if ke.state() == KeyState::PRESSED {
                                cap.pressed_keys.insert(tname);
                            } else {
                                cap.pressed_keys.remove(&tname);
                            }
                            out.push(KeyEvent::new(target, ke.state()).into());
                        }
                    }
                    _ => {}
                }
            }
        }

        if !out.is_empty() {
            if let Err(e) = vx.write_batch(&out) {
                tracing::error!("Error emulando: {}", e);
                break;
            }
        }
    }
    tracing::info!("emulation_loop: finalizado");
}
