use evdevil::event::{Abs, Key};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mapping from source axis to Xbox axis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisMapping {
    pub source: String,
    pub target: String,
    pub invert: bool,
    pub deadzone: i32,
    pub scale: f32,
    #[serde(default)]
    pub center: i32,
}

/// Mapping from source button to Xbox button
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonMapping {
    pub source: String,
    pub target: String,
}

/// A complete mapping profile for a controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProfile {
    pub name: String,
    pub description: Option<String>,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub axes: Vec<AxisMapping>,
    pub buttons: Vec<ButtonMapping>,
}

/// Parsed and ready-to-use mapping table
#[derive(Debug, Clone)]
pub struct Mapper {
    pub axis_map: HashMap<Abs, (Abs, bool, i32, f32, i32)>,
    pub button_map: HashMap<Key, Key>,
}

impl Mapper {
    /// Create a mapper from a profile
    pub fn from_profile(profile: &MappingProfile) -> Result<Self, String> {
        let mut axis_map = HashMap::new();
        let mut button_map = HashMap::new();

        for am in &profile.axes {
            let source = parse_abs(&am.source)?;
            let target = parse_abs(&am.target)?;
            let deadzone = if am.deadzone > 0 { am.deadzone } else { 0 };
            let scale = if am.scale > 0.0 { am.scale } else { 1.0 };
            axis_map.insert(source, (target, am.invert, deadzone, scale, am.center));
        }

        for bm in &profile.buttons {
            let source = parse_key(&bm.source)?;
            let target = parse_key(&bm.target)?;
            button_map.insert(source, target);
        }

        Ok(Self {
            axis_map,
            button_map,
        })
    }

    /// Create a default 1:1 identity mapping
    pub fn identity() -> Self {
        let axis_map = HashMap::from([
            (Abs::X, (Abs::X, false, 8000, 1.0, 0)),
            (Abs::Y, (Abs::Y, false, 8000, 1.0, 0)),
            (Abs::Z, (Abs::Z, false, 0, 1.0, 0)),
            (Abs::RX, (Abs::RX, false, 8000, 1.0, 0)),
            (Abs::RY, (Abs::RY, false, 8000, 1.0, 0)),
            (Abs::RZ, (Abs::RZ, false, 0, 1.0, 0)),
            (Abs::HAT0X, (Abs::HAT0X, false, 0, 1.0, 0)),
            (Abs::HAT0Y, (Abs::HAT0Y, false, 0, 1.0, 0)),
        ]);

        let button_map = HashMap::from([
            (Key::BTN_SOUTH, Key::BTN_A),
            (Key::BTN_EAST, Key::BTN_B),
            (Key::BTN_NORTH, Key::BTN_Y),
            (Key::BTN_WEST, Key::BTN_X),
            (Key::BTN_TL, Key::BTN_TL),
            (Key::BTN_TR, Key::BTN_TR),
            (Key::BTN_SELECT, Key::BTN_SELECT),
            (Key::BTN_START, Key::BTN_START),
            (Key::BTN_MODE, Key::BTN_MODE),
            (Key::BTN_THUMBL, Key::BTN_THUMBL),
            (Key::BTN_THUMBR, Key::BTN_THUMBR),
            (Key::BTN_A, Key::BTN_A),
            (Key::BTN_B, Key::BTN_B),
            (Key::BTN_X, Key::BTN_X),
            (Key::BTN_Y, Key::BTN_Y),
        ]);

        Self {
            axis_map,
            button_map,
        }
    }

    /// Map a source axis value to a target axis value
    pub fn map_axis(&self, source: Abs, value: i32) -> Option<(Abs, i32)> {
        self.axis_map
            .get(&source)
            .map(|(target, invert, deadzone, scale, center)| {
                // 1. Remove center offset -> v is now a pure delta
                let mut v = value - center;

                // 2. Scale up to Xbox standard range
                if *scale != 1.0 {
                    v = (v as f32 * scale) as i32;
                }

                // 3. Apply deadzone AFTER scaling (so standard 8000 works for any controller)
                if v.abs() < *deadzone {
                    v = 0;
                }

                // 4. Invert if needed
                if *invert {
                    v = -v;
                }

                // Emulated Xbox target clamp bounds
                if target == &Abs::Z || target == &Abs::RZ {
                    v = v.clamp(0, 255);
                } else if target == &Abs::HAT0X || target == &Abs::HAT0Y {
                    v = v.clamp(-1, 1);
                } else {
                    v = v.clamp(-32768, 32767);
                }

                (*target, v)
            })
    }

    /// Map a source button to a target button
    pub fn map_button(&self, source: Key) -> Option<Key> {
        self.button_map.get(&source).copied()
    }
}

/// Parse an axis name string to an Abs value
fn parse_abs(s: &str) -> Result<Abs, String> {
    // evdevil uses serde with names like "X", "Y", "ABS_X" etc.
    // We handle both formats
    let normalized = s.strip_prefix("ABS_").unwrap_or(s);
    match normalized {
        "X" => Ok(Abs::X),
        "Y" => Ok(Abs::Y),
        "Z" => Ok(Abs::Z),
        "RX" => Ok(Abs::RX),
        "RY" => Ok(Abs::RY),
        "RZ" => Ok(Abs::RZ),
        "HAT0X" => Ok(Abs::HAT0X),
        "HAT0Y" => Ok(Abs::HAT0Y),
        "HAT1X" => Ok(Abs::HAT1X),
        "HAT1Y" => Ok(Abs::HAT1Y),
        "THROTTLE" => Ok(Abs::THROTTLE),
        "RUDDER" => Ok(Abs::RUDDER),
        "WHEEL" => Ok(Abs::WHEEL),
        "GAS" => Ok(Abs::GAS),
        "BRAKE" => Ok(Abs::BRAKE),
        "PRESSURE" => Ok(Abs::PRESSURE),
        "DISTANCE" => Ok(Abs::DISTANCE),
        "TILT_X" => Ok(Abs::TILT_X),
        "TILT_Y" => Ok(Abs::TILT_Y),
        _ => Err(format!("Eje desconocido: {}", s)),
    }
}

/// Parse a button name string to a Key value
fn parse_key(s: &str) -> Result<Key, String> {
    match s {
        "BTN_A" => Ok(Key::BTN_A),
        "BTN_B" => Ok(Key::BTN_B),
        "BTN_C" => Ok(Key::BTN_C),
        "BTN_X" => Ok(Key::BTN_X),
        "BTN_Y" => Ok(Key::BTN_Y),
        "BTN_Z" => Ok(Key::BTN_Z),
        "BTN_TL" => Ok(Key::BTN_TL),
        "BTN_TR" => Ok(Key::BTN_TR),
        "BTN_TL2" => Ok(Key::BTN_TL2),
        "BTN_TR2" => Ok(Key::BTN_TR2),
        "BTN_SELECT" => Ok(Key::BTN_SELECT),
        "BTN_START" => Ok(Key::BTN_START),
        "BTN_MODE" => Ok(Key::BTN_MODE),
        "BTN_THUMBL" => Ok(Key::BTN_THUMBL),
        "BTN_THUMBR" => Ok(Key::BTN_THUMBR),
        "BTN_SOUTH" => Ok(Key::BTN_SOUTH),
        "BTN_EAST" => Ok(Key::BTN_EAST),
        "BTN_NORTH" => Ok(Key::BTN_NORTH),
        "BTN_WEST" => Ok(Key::BTN_WEST),
        "BTN_TRIGGER" => Ok(Key::BTN_TRIGGER),
        "BTN_THUMB" => Ok(Key::BTN_THUMB),
        "BTN_THUMB2" => Ok(Key::BTN_THUMB2),
        "BTN_TOP" => Ok(Key::BTN_TOP),
        "BTN_TOP2" => Ok(Key::BTN_TOP2),
        "BTN_PINKIE" => Ok(Key::BTN_PINKIE),
        "BTN_BASE" => Ok(Key::BTN_BASE),
        "BTN_JOYSTICK" => Ok(Key::BTN_JOYSTICK),
        _ => Err(format!("Boton desconocido: {}", s)),
    }
}
