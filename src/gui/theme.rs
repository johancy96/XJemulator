use eframe::egui;
use eframe::egui::{Color32, Stroke, Visuals};

/// Paleta de colores Premium para XJEmulator
pub struct Theme;

impl Theme {
    // Colores de Fondo
    pub const BG_DEEP: Color32 = Color32::from_rgb(18, 18, 22);
    pub const BG_CARD: Color32 = Color32::from_rgb(30, 30, 38);
    pub const BG_SIDEBAR: Color32 = Color32::from_rgb(24, 24, 30);
    
    // Colores de Acento (Xbox Style)
    pub const ACCENT: Color32 = Color32::from_rgb(16, 124, 16); // Xbox Green
    
    // Colores de Estado
    pub const SUCCESS: Color32 = Color32::from_rgb(46, 204, 113);
    pub const ERROR: Color32 = Color32::from_rgb(231, 76, 60);
    pub const INFO: Color32 = Color32::from_rgb(52, 152, 219);
    
    // Colores de Texto
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 160, 180);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(100, 100, 120);

    /// Aplica el estilo visual premium a la aplicación
    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        
        // Personalización de colores base
        visuals.panel_fill = Self::BG_DEEP;
        visuals.window_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_SECONDARY);
        
        // Personalización de widgets activos
        visuals.selection.bg_fill = Self::ACCENT;
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
        
        // Estilo de botones
        visuals.widgets.inactive.bg_fill = Self::BG_SIDEBAR;
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 45, 55);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.active.bg_fill = Self::ACCENT;
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
        
        ctx.set_visuals(visuals);
        
        // Configuración de fuentes y espaciado
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = egui::Margin::same(15);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        ctx.set_style(style);
    }
}
