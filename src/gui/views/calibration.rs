use eframe::egui;
use crate::gui::app::App;
use crate::gui::theme::Theme;
use crate::gui::widgets::controller_view::draw_xbox_controller;
use crate::gui::types::CalibStep;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(20.0);
    
    match app.calib_step {
        CalibStep::Buttons(i) => render_button_step(app, ui, i),
        CalibStep::Axes(i) => render_axis_step(app, ui, i),
        CalibStep::Review => render_review(app, ui),
        _ => {}
    }
}

fn render_button_step(app: &mut App, ui: &mut egui::Ui, index: usize) {
    let (label, hint, xbox_key) = {
        let slot = &app.calib_btns[index];
        (slot.label.clone(), slot.hint.clone(), slot.xbox_key)
    };
    
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "calib_step_buttons")).strong().color(Theme::ACCENT));
        ui.add_space(10.0);
        
        draw_xbox_controller(ui, Some(xbox_key), None);
        
        ui.add_space(20.0);
        ui.label(egui::RichText::new(label).size(32.0).strong());
        ui.label(egui::RichText::new(hint).weak());
        
        ui.add_space(30.0);
        
        app.handle_calibration_input();
    });
}

fn render_axis_step(app: &mut App, ui: &mut egui::Ui, index: usize) {
    let (label, dir_label, xbox_axis) = {
        let slot = &app.calib_axes[index];
        (slot.label.clone(), slot.direction_label.clone(), slot.xbox_axis)
    };
    
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "calib_step_axes")).strong().color(Theme::INFO));
        ui.add_space(10.0);
        
        draw_xbox_controller(ui, None, Some(xbox_axis));
        
        ui.add_space(20.0);
        ui.label(egui::RichText::new(label).size(32.0).strong());
        ui.label(egui::RichText::new(dir_label).size(24.0));
        
        ui.add_space(30.0);
        app.handle_calibration_input();
    });
}

fn render_review(app: &mut App, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "calib_review_title")).strong().size(24.0));
        ui.add_space(20.0);
        
        if ui.button(egui::RichText::new(crate::i18n::t(&app.config.lang, "btn_save_profile_action")).size(20.0)).clicked() {
            app.save_profile();
        }
        
        if ui.button(crate::i18n::t(&app.config.lang, "btn_cancel")).clicked() {
            app.reset_calibration();
        }
    });
}
