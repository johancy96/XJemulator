use eframe::egui;
use crate::gui::theme::Theme;

pub fn draw_device_card(
    ui: &mut egui::Ui,
    name: &str,
    path: &str,
    selected: bool,
    emulating: bool,
) -> (bool, bool, bool) { // (clicked, start_clicked, stop_clicked)
    let mut clicked = false;
    let mut start_clicked = false;
    let mut stop_clicked = false;

    let bg_color = if selected {
        egui::Color32::from_rgb(40, 40, 50)
    } else {
        Theme::BG_CARD
    };

    let stroke = if selected {
        egui::Stroke::new(2.0, Theme::ACCENT)
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_gray(50))
    };

    egui::Frame::default()
        .fill(bg_color)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Icono e Información
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if emulating {
                            ui.colored_label(Theme::SUCCESS, crate::gui::fonts::icons::CHECK_CIRCLE);
                        } else {
                            ui.colored_label(Theme::TEXT_DIM, crate::gui::fonts::icons::GAMEPAD);
                        }
                        let resp = ui.selectable_label(false, egui::RichText::new(name).strong().size(14.0));
                        if resp.clicked() {
                            clicked = true;
                        }
                    });
                    ui.label(egui::RichText::new(path).size(10.0).color(Theme::TEXT_DIM));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if emulating {
                        if ui.button(egui::RichText::new(crate::gui::fonts::icons::STOP).color(Theme::ERROR)).clicked() {
                            stop_clicked = true;
                        }
                    } else if selected {
                        if ui.button(egui::RichText::new(crate::gui::fonts::icons::PLAY).color(Theme::SUCCESS)).clicked() {
                            start_clicked = true;
                        }
                    }
                });
            });
        });

    (clicked, start_clicked, stop_clicked)
}
