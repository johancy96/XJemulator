use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Stroke};
use crate::gui::theme::Theme;

pub fn draw_xbox_controller(
    ui: &mut egui::Ui,
    highlight_key: Option<&str>,
    highlight_axis: Option<&str>,
) {
    let (response, painter) = ui.allocate_painter(
        egui::vec2(400.0, 250.0),
        egui::Sense::hover(),
    );

    let rect = response.rect;
    let center = rect.center();

    // Cuerpo principal del mando (Simplificado pero elegante)
    let main_body = Rect::from_center_size(center, egui::vec2(300.0, 160.0));
    painter.rect_filled(main_body, 40.0, Theme::BG_CARD);
    painter.rect_stroke(main_body, 40.0, Stroke::new(2.0, Theme::TEXT_DIM), egui::StrokeKind::Inside);

    // Empuñaduras (Grips)
    let left_grip = Rect::from_center_size(center + egui::vec2(-120.0, 40.0), egui::vec2(80.0, 120.0));
    painter.rect_filled(left_grip, 30.0, Theme::BG_CARD);
    painter.rect_stroke(left_grip, 30.0, Stroke::new(2.0, Theme::TEXT_DIM), egui::StrokeKind::Inside);

    let right_grip = Rect::from_center_size(center + egui::vec2(120.0, 40.0), egui::vec2(80.0, 120.0));
    painter.rect_filled(right_grip, 30.0, Theme::BG_CARD);
    painter.rect_stroke(right_grip, 30.0, Stroke::new(2.0, Theme::TEXT_DIM), egui::StrokeKind::Inside);

    // Botones ABXY
    draw_button(&painter, center + egui::vec2(80.0, -20.0), "BTN_B", highlight_key, Color32::from_rgb(231, 76, 60)); // B
    draw_button(&painter, center + egui::vec2(60.0, 0.0), "BTN_A", highlight_key, Theme::ACCENT); // A
    draw_button(&painter, center + egui::vec2(40.0, -20.0), "BTN_X", highlight_key, Color32::from_rgb(52, 152, 219)); // X
    draw_button(&painter, center + egui::vec2(60.0, -40.0), "BTN_Y", highlight_key, Color32::from_rgb(241, 196, 15)); // Y

    // Sticks
    draw_stick(&painter, center + egui::vec2(-80.0, -10.0), "ABS_X", highlight_axis); // Left Stick
    draw_stick(&painter, center + egui::vec2(40.0, 30.0), "ABS_RX", highlight_axis); // Right Stick

    // D-Pad
    draw_dpad(&painter, center + egui::vec2(-40.0, 30.0), "ABS_HAT0X", highlight_axis);
}

fn draw_button(painter: &egui::Painter, pos: Pos2, key: &str, highlight: Option<&str>, color: Color32) {
    let is_highlighted = highlight == Some(key);
    let final_color = if is_highlighted { color } else { Theme::BG_DEEP };
    let stroke = if is_highlighted { Stroke::new(3.0, Color32::WHITE) } else { Stroke::new(1.0, Theme::TEXT_DIM) };
    
    painter.circle(pos, 12.0, final_color, stroke);
}

fn draw_stick(painter: &egui::Painter, pos: Pos2, axis: &str, highlight: Option<&str>) {
    let is_highlighted = highlight == Some(axis);
    let final_color = if is_highlighted { Theme::ACCENT } else { Theme::BG_DEEP };
    let stroke = if is_highlighted { Stroke::new(3.0, Color32::WHITE) } else { Stroke::new(1.0, Theme::TEXT_DIM) };
    
    painter.circle(pos, 22.0, final_color, stroke);
    painter.circle_filled(pos, 18.0, Theme::BG_CARD);
}

fn draw_dpad(painter: &egui::Painter, pos: Pos2, axis: &str, highlight: Option<&str>) {
    let is_highlighted = highlight == Some(axis);
    let final_color = if is_highlighted { Theme::ACCENT } else { Theme::BG_DEEP };
    
    painter.rect_filled(Rect::from_center_size(pos, egui::vec2(30.0, 10.0)), 2.0, final_color);
    painter.rect_filled(Rect::from_center_size(pos, egui::vec2(10.0, 30.0)), 2.0, final_color);
}
