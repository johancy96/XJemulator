use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Stroke};
use crate::gui::theme::Theme;

pub fn draw_xbox_controller(
    ui: &mut egui::Ui,
    highlight_key: Option<&str>,
    highlight_axis: Option<&str>,
) {
    let (response, painter) = ui.allocate_painter(
        egui::vec2(400.0, 260.0),
        egui::Sense::hover(),
    );

    let rect = response.rect;
    let center = rect.center();

    // Diseño Minimalista: Solo botones y sticks en su distribución Xbox
    // Hemos eliminado la silueta del mando para mayor claridad visual

    // --- Botón Guía (Xbox) ---
    draw_guide_button(&painter, center + egui::vec2(0.0, -45.0), highlight_key == Some("BTN_MODE"));

    // --- Botones de Control Central (Back, Start) ---
    draw_small_btn(&painter, center + egui::vec2(-35.0, -25.0), "BTN_SELECT", highlight_key); // View (Back)
    draw_small_btn(&painter, center + egui::vec2(35.0, -25.0), "BTN_START", highlight_key);  // Menu (Start)

    // --- Stick Izquierdo (Arriba-Izquierda) ---
    draw_stick(&painter, center + egui::vec2(-75.0, -25.0), "ABS_X", highlight_axis, highlight_key == Some("BTN_THUMBL"));

    // --- Stick Derecho (Abajo-Derecha) ---
    draw_stick(&painter, center + egui::vec2(55.0, 35.0), "ABS_RX", highlight_axis, highlight_key == Some("BTN_THUMBR"));

    // --- D-Pad ---
    draw_dpad(&painter, center + egui::vec2(-45.0, 35.0), "ABS_HAT0X", highlight_axis);

    // --- Botones ABXY ---
    let abxy_center = center + egui::vec2(85.0, -25.0);
    draw_button(&painter, abxy_center + egui::vec2(0.0, 22.0), "BTN_A", highlight_key, Color32::from_rgb(39, 174, 96)); // A
    draw_button(&painter, abxy_center + egui::vec2(22.0, 0.0), "BTN_B", highlight_key, Color32::from_rgb(192, 57, 43)); // B
    draw_button(&painter, abxy_center + egui::vec2(-22.0, 0.0), "BTN_X", highlight_key, Color32::from_rgb(41, 128, 185)); // X
    draw_button(&painter, abxy_center + egui::vec2(0.0, -22.0), "BTN_Y", highlight_key, Color32::from_rgb(241, 196, 15)); // Y

    // --- Bumpers y Gatillos (Resumen visual superior) ---
    draw_bumper(&painter, center + egui::vec2(-70.0, -90.0), "BTN_TL", highlight_key);
    draw_bumper(&painter, center + egui::vec2(70.0, -90.0), "BTN_TR", highlight_key);
    draw_trigger(&painter, center + egui::vec2(-70.0, -115.0), "ABS_Z", highlight_axis);
    draw_trigger(&painter, center + egui::vec2(70.0, -115.0), "ABS_RZ", highlight_axis);
}

fn draw_guide_button(painter: &egui::Painter, pos: Pos2, active: bool) {
    let color = if active { Color32::from_rgb(16, 124, 16) } else { Color32::from_gray(40) };
    painter.circle(pos, 16.0, color, Stroke::new(2.0, Color32::WHITE));
    painter.text(pos, egui::Align2::CENTER_CENTER, "X", egui::FontId::proportional(14.0), Color32::WHITE);
}

fn draw_small_btn(painter: &egui::Painter, pos: Pos2, key: &str, highlight: Option<&str>) {
    let active = highlight == Some(key);
    let color = if active { Theme::ACCENT } else { Color32::from_gray(50) };
    painter.circle(pos, 8.0, color, Stroke::new(1.0, Color32::WHITE));
}

fn draw_button(painter: &egui::Painter, pos: Pos2, key: &str, highlight: Option<&str>, color: Color32) {
    let active = highlight == Some(key);
    let fill = if active { color } else { Color32::from_gray(35) };
    painter.circle(pos, 11.0, fill, Stroke::new(1.5, Color32::WHITE));
}

fn draw_stick(painter: &egui::Painter, pos: Pos2, axis: &str, highlight_axis: Option<&str>, highlight_btn: bool) {
    let active = highlight_axis == Some(axis) || highlight_axis == Some("ABS_Y") || highlight_axis == Some("ABS_RY") || highlight_btn;
    let color = if active { Theme::ACCENT } else { Color32::from_gray(45) };
    painter.circle(pos, 22.0, color, Stroke::new(2.0, Color32::WHITE));
}

fn draw_dpad(painter: &egui::Painter, pos: Pos2, axis: &str, highlight: Option<&str>) {
    let active = highlight == Some(axis) || highlight == Some("ABS_HAT0Y");
    let color = if active { Theme::ACCENT } else { Color32::from_gray(45) };
    painter.rect_filled(Rect::from_center_size(pos, egui::vec2(32.0, 10.0)), 2.0, color);
    painter.rect_filled(Rect::from_center_size(pos, egui::vec2(10.0, 32.0)), 2.0, color);
}

fn draw_bumper(painter: &egui::Painter, pos: Pos2, key: &str, highlight: Option<&str>) {
    let active = highlight == Some(key);
    let color = if active { Theme::ACCENT } else { Color32::from_gray(35) };
    let rect = Rect::from_center_size(pos, egui::vec2(60.0, 16.0));
    painter.rect_filled(rect, 4.0, color);
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::WHITE), egui::StrokeKind::Inside);
}

fn draw_trigger(painter: &egui::Painter, pos: Pos2, axis: &str, highlight: Option<&str>) {
    let active = highlight == Some(axis);
    let color = if active { Theme::ACCENT } else { Color32::from_gray(30) };
    let rect = Rect::from_center_size(pos, egui::vec2(50.0, 25.0));
    painter.rect_filled(rect, 6.0, color);
    painter.rect_stroke(rect, 6.0, Stroke::new(1.0, Color32::WHITE), egui::StrokeKind::Inside);
}
