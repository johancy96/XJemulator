use evdevil::event::{EventType, Key};
use evdevil::Evdev;
use tracing::{debug, info, warn};

/// Information about a detected gamepad
#[derive(Debug, Clone)]
pub struct GamepadInfo {
    pub path: String,
    pub name: String,
}

/// Check if a device is likely a gamepad/joystick
fn is_gamepad(device: &Evdev) -> bool {
    let has_abs = device
        .supported_events()
        .map(|s| s.contains(EventType::ABS))
        .unwrap_or(false);

    let has_gamepad_buttons = device
        .supported_keys()
        .map(|keys| {
            keys.contains(Key::BTN_A)
                || keys.contains(Key::BTN_SOUTH)
                || keys.contains(Key::BTN_GAMEPAD)
                || keys.contains(Key::BTN_TRIGGER)
                || keys.contains(Key::BTN_THUMB)
                || keys.contains(Key::BTN_JOYSTICK)
        })
        .unwrap_or(false);

    let has_joystick_buttons = device
        .supported_keys()
        .map(|keys| {
            keys.contains(Key::BTN_JOYSTICK)
                || keys.contains(Key::BTN_TRIGGER)
                || keys.contains(Key::BTN_THUMB)
        })
        .unwrap_or(false);

    (has_abs && has_gamepad_buttons) || has_joystick_buttons
}

/// Scan /dev/input/ for gamepad devices
pub fn scan_gamepads() -> Vec<GamepadInfo> {
    let mut gamepads = Vec::new();

    let entries = match std::fs::read_dir("/dev/input") {
        Ok(e) => e,
        Err(e) => {
            warn!("No se puede leer /dev/input: {}", e);
            return gamepads;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();

        if !path_str.contains("event") {
            continue;
        }

        let device = match Evdev::open(&path) {
            Ok(d) => d,
            Err(e) => {
                debug!("No se puede abrir {}: {}", path_str, e);
                continue;
            }
        };

        if !is_gamepad(&device) {
            continue;
        }

        let name = device.name().unwrap_or_else(|_| "Desconocido".into());
        let _phys = device.phys().unwrap_or(None);
        let input_id = device.input_id().ok();
        let _vendor_id = input_id.map(|id| id.vendor()).unwrap_or(0);
        let _product_id = input_id.map(|id| id.product()).unwrap_or(0);

        let axes_count = device
            .supported_abs_axes()
            .map(|a| a.iter().count())
            .unwrap_or(0);

        let buttons_count = device
            .supported_keys()
            .map(|k| k.iter().count())
            .unwrap_or(0);

        info!(
            "Gamepad detectado: {} [{}] - {} ejes, {} botones",
            name, path_str, axes_count, buttons_count
        );

        gamepads.push(GamepadInfo {
            path: path_str,
            name,
        });
    }

    gamepads
}
