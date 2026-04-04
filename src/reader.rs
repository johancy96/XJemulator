use evdevil::event::{Abs, EventKind, Key, KeyState};
use evdevil::Evdev;
use tracing::{debug, info};

/// Current state of an input device
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub axes: std::collections::HashMap<Abs, i32>,
    pub buttons: std::collections::HashMap<Key, bool>,
}

/// Open and configure an evdev device for reading
pub fn open_device(path: &str) -> Result<Evdev, std::io::Error> {
    let device = Evdev::open(path)?;

    let name = device.name().unwrap_or_else(|_| "Desconocido".into());
    info!("Dispositivo abierto: {} [{}]", name, path);

    // Grab the device so other programs don't see duplicate events
    match device.grab() {
        Ok(()) => info!("Dispositivo capturado (grab) exitosamente"),
        Err(e) => debug!("No se pudo grabar el dispositivo (no critico): {}", e),
    }

    Ok(device)
}

/// Read events from device and update state
pub fn process_events(device: &Evdev, state: &mut InputState) -> Vec<EventKind> {
    let mut events = Vec::new();

    let mut buf = [evdevil::event::InputEvent::zeroed(); 64];
    let count = match device.read_events(&mut buf) {
        Ok(n) => n,
        Err(e) => {
            debug!("Error leyendo eventos: {}", e);
            return events;
        }
    };

    for raw_event in &buf[..count] {
        let kind = raw_event.kind();

        match kind {
            EventKind::Abs(ref abs_event) => {
                let abs = abs_event.abs();
                let value = abs_event.value();
                state.axes.insert(abs, value);
                debug!("Eje {:?} = {}", abs, value);
            }
            EventKind::Key(ref key_event) => {
                let key = key_event.key();
                let pressed = key_event.state() == KeyState::PRESSED;
                state.buttons.insert(key, pressed);
                debug!(
                    "Boton {:?} = {}",
                    key,
                    if pressed { "PRESIONADO" } else { "SOLTADO" }
                );
            }
            _ => {
                debug!("Otro evento: {:?}", kind);
            }
        }

        events.push(kind);
    }

    events
}
