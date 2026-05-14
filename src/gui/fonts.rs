use eframe::egui;

pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 1. Cargar Phosphor (Iconos) - El archivo binario es válido
    fonts.font_data.insert(
        "Phosphor".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/Phosphor.ttf")).into(),
    );

    // Añadir Phosphor a Proporcional para que los glifos de iconos funcionen como fallback
    // egui usará su fuente predeterminada para el texto y Phosphor para los iconos
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
        .push("Phosphor".to_owned());

    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
        .push("Phosphor".to_owned());

    ctx.set_fonts(fonts);
}

// Constantes de Iconos Phosphor (mapeo manual de algunos útiles)
pub mod icons {
    pub const GAMEPAD: &str = "\u{e26e}";     // PH-GAME-CONTROLLER
    pub const TRASH: &str = "\u{e4a6}";       // PH-TRASH
    pub const PLAY: &str = "\u{e3d0}";        // PH-PLAY
    pub const STOP: &str = "\u{e46c}";        // PH-STOP
    pub const X: &str = "\u{e4f6}";           // PH-X
    pub const GLOBE: &str = "\u{e288}";       // PH-GLOBE
    pub const WARNING: &str = "\u{e4e0}";      // PH-WARNING
    pub const CHECK_CIRCLE: &str = "\u{e184}"; // PH-CHECK-CIRCLE
}
