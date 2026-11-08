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

/// Mapping from a digital source button (0/1) to an Xbox axis
/// Allows older controllers with digital triggers or button D-Pads to drive Xbox axes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonAxisMapping {
    pub source: String,
    pub target: String,
    pub active_value: i32,
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
    #[serde(default)]
    pub button_axes: Vec<ButtonAxisMapping>,
}

/// Parsed and ready-to-use mapping table
#[derive(Debug, Clone)]
pub struct Mapper {
    pub axis_map: HashMap<Abs, (Abs, bool, i32, f32, i32)>,
    pub button_map: HashMap<Key, Key>,
    /// Digital button -> list of (target_axis, active_value)
    pub button_axis_map: HashMap<Key, (Abs, i32)>,
}

impl Mapper {
    /// Create a mapper from a profile
    pub fn from_profile(profile: &MappingProfile) -> Result<Self, String> {
        let mut axis_map = HashMap::new();
        let mut button_map = HashMap::new();
        let mut button_axis_map = HashMap::new();

        for am in &profile.axes {
            let source = parse_abs(&am.source)?;
            let target = parse_abs(&am.target)?;
            let deadzone = am.deadzone.max(0);
            let scale = if am.scale > 0.0 { am.scale } else { 1.0 };
            axis_map.insert(source, (target, am.invert, deadzone, scale, am.center));
        }

        for bm in &profile.buttons {
            let source = parse_key(&bm.source)?;
            let target = parse_key(&bm.target)?;
            button_map.insert(source, target);
        }

        for bam in &profile.button_axes {
            let source = parse_key(&bam.source)?;
            let target = parse_abs(&bam.target)?;
            button_axis_map.insert(source, (target, bam.active_value));
        }

        Ok(Self {
            axis_map,
            button_map,
            button_axis_map,
        })
    }

    /// Create a default 1:1 identity mapping
    pub fn identity() -> Self {
        let axis_map = HashMap::from([
            (Abs::X, (Abs::X, false, 3000, 1.0, 0)),
            (Abs::Y, (Abs::Y, false, 3000, 1.0, 0)),
            (Abs::Z, (Abs::Z, false, 4, 1.0, 0)),
            (Abs::RX, (Abs::RX, false, 3000, 1.0, 0)),
            (Abs::RY, (Abs::RY, false, 3000, 1.0, 0)),
            (Abs::RZ, (Abs::RZ, false, 4, 1.0, 0)),
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
            button_axis_map: HashMap::new(),
        }
    }

    /// Map a source axis value to a target Xbox 360 axis value.
    ///
    /// Uses a **Continuous Scaled Deadzone** (Re-escalado Progresivo) so that
    /// Hall-effect/magnetic sticks and triggers transition smoothly from 0% to 100%
    /// velocity without any sudden jump at the deadzone boundary.
    pub fn map_axis(&self, source: Abs, value: i32) -> Option<(Abs, i32)> {
        self.axis_map
            .get(&source)
            .map(|(target, invert, deadzone, scale, center)| {
                // 1. Delta continuo desde el punto de reposo calibrado
                let mut delta = (value - *center) as f32;

                // 2. Invertir polaridad antes del clamp si el eje físico va en sentido opuesto
                if *invert {
                    delta = -delta;
                }

                // 3. Escalar al rango objetivo de Xbox
                let scaled = delta * *scale;
                let dz = (*deadzone).max(0) as f32;

                // 4. Re-escalado continuo por tipo de eje (Sticks vs Gatillos vs D-Pad)
                let out = if target == &Abs::Z || target == &Abs::RZ {
                    // Gatillos analógicos (0..255): progresión continua desde la zona muerta hasta 255
                    if scaled <= dz {
                        0
                    } else {
                        let effective_max = (255.0 - dz).max(1.0);
                        let norm = ((scaled - dz) / effective_max).clamp(0.0, 1.0);
                        (norm * 255.0).round() as i32
                    }
                } else if target == &Abs::HAT0X || target == &Abs::HAT0Y {
                    // Cruceta (D-Pad): -1, 0, 1
                    if scaled.abs() < 0.5 {
                        0
                    } else if scaled > 0.0 {
                        1
                    } else {
                        -1
                    }
                } else {
                    // Joysticks analógicos/magnéticos (-32768..32767):
                    // Re-escalado lineal continuo desde [dz .. 32767] -> [1 .. 32767]
                    let abs_v = scaled.abs();
                    if abs_v <= dz {
                        0
                    } else {
                        let effective_max = (32767.0 - dz).max(1.0);
                        let norm = ((abs_v - dz) / effective_max).clamp(0.0, 1.0);
                        let mag = (norm * 32767.0).round() as i32;
                        if scaled < 0.0 {
                            (-mag).clamp(-32768, 32767)
                        } else {
                            mag.clamp(-32768, 32767)
                        }
                    }
                };

                (*target, out)
            })
    }

    /// Map a source button to a target button
    pub fn map_button(&self, source: Key) -> Option<Key> {
        self.button_map.get(&source).copied()
    }

    /// Map a digital button (on older controllers) to an emulated Xbox axis value
    pub fn map_button_to_axis(&self, source: Key, pressed: bool) -> Option<(Abs, i32)> {
        self.button_axis_map
            .get(&source)
            .map(|&(target_abs, active_val)| {
                let val = if pressed { active_val } else { 0 };
                (target_abs, val)
            })
    }
}

/// Parse an axis name string to an Abs value
pub(crate) fn parse_abs(s: &str) -> Result<Abs, String> {
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
        "HAT2X" => Ok(Abs::HAT2X),
        "HAT2Y" => Ok(Abs::HAT2Y),
        "HAT3X" => Ok(Abs::HAT3X),
        "HAT3Y" => Ok(Abs::HAT3Y),
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
pub(crate) fn parse_key(s: &str) -> Result<Key, String> {
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
        "BTN_BASE2" => Ok(Key::BTN_BASE2),
        "BTN_BASE3" => Ok(Key::BTN_BASE3),
        "BTN_BASE4" => Ok(Key::BTN_BASE4),
        "BTN_BASE5" => Ok(Key::BTN_BASE5),
        "BTN_BASE6" => Ok(Key::BTN_BASE6),
        "BTN_DEAD" => Ok(Key::BTN_DEAD),
        "BTN_JOYSTICK" => Ok(Key::BTN_JOYSTICK),
        "BTN_DPAD_UP" => Ok(Key::BTN_DPAD_UP),
        "BTN_DPAD_DOWN" => Ok(Key::BTN_DPAD_DOWN),
        "BTN_DPAD_LEFT" => Ok(Key::BTN_DPAD_LEFT),
        "BTN_DPAD_RIGHT" => Ok(Key::BTN_DPAD_RIGHT),
        _ => Err(format!("Boton desconocido: {}", s)),
    }
}
