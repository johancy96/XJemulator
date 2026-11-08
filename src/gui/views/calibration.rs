use crate::gui::app::App;
use crate::gui::theme::Theme;
use crate::gui::types::CalibStep;
use crate::gui::widgets::controller_view::draw_xbox_controller;
use eframe::egui;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(10.0);

    // Indicador de Progreso Superior
    render_progress_bar(app, ui);
    ui.add_space(15.0);

    // Contenedor principal para centrar todo el contenido
    ui.vertical_centered(|ui| {
        match app.calib_step {
            CalibStep::Buttons(i) => render_button_step(app, ui, i),
            CalibStep::Axes(i) => render_axis_step(app, ui, i),
            CalibStep::Review => render_review(app, ui),
            _ => {}
        }

        ui.add_space(25.0);
        render_navigation_footer(app, ui);
    });

    // Consola de Diagnóstico en tiempo real
    render_debug_console(app, ui);
}

fn render_debug_console(app: &App, ui: &mut egui::Ui) {
    ui.add_space(15.0);
    egui::Frame::default()
        .fill(Theme::BG_DEEP)
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_diag_console"))
                    .weak()
                    .size(10.0),
            );
            egui::ScrollArea::vertical()
                .id_salt("debug_console")
                .max_height(70.0)
                .show(ui, |ui| {
                    for log in &app.calib_logs {
                        ui.label(
                            egui::RichText::new(log)
                                .monospace()
                                .size(10.0)
                                .color(Theme::INFO),
                        );
                    }
                });
        });
}

fn render_button_step(app: &mut App, ui: &mut egui::Ui, index: usize) {
    let (label, hint, xbox_key) = {
        let slot = &app.calib_btns[index];
        (slot.label.clone(), slot.hint.clone(), slot.xbox_key)
    };

    ui.label(
        egui::RichText::new(crate::i18n::t(&app.config.lang, "calib_step_buttons"))
            .strong()
            .color(Theme::ACCENT)
            .size(14.0),
    );
    ui.add_space(5.0);

    draw_xbox_controller(ui, Some(xbox_key), None);

    ui.add_space(12.0);
    ui.label(
        egui::RichText::new(label)
            .size(28.0)
            .strong()
            .color(egui::Color32::WHITE),
    );
    ui.label(egui::RichText::new(hint).weak().size(16.0));

    ui.add_space(15.0);
    app.handle_calibration_input();
}

fn render_axis_step(app: &mut App, ui: &mut egui::Ui, index: usize) {
    let (label, dir_label, xbox_axis) = {
        let axis = &app.calib_axes[index];
        (
            axis.label.clone(),
            axis.direction_label.clone(),
            axis.xbox_axis,
        )
    };

    ui.label(
        egui::RichText::new(crate::i18n::t(&app.config.lang, "calib_step_axes"))
            .strong()
            .color(Theme::INFO)
            .size(14.0),
    );
    ui.add_space(5.0);

    ui.label(
        egui::RichText::new(&label)
            .size(28.0)
            .strong()
            .color(egui::Color32::WHITE),
    );
    ui.label(egui::RichText::new(&dir_label).size(20.0).color(Theme::ACCENT));
    ui.add_space(15.0);

    // La franja verde de movimiento SOLO aparece durante CalibStep::Axes cuando
    // el mando ya estuvo en reposo (!waiting_for_release) y el usuario está moviendo el eje.
    if app.waiting_for_release {
        if app.needs_user_release_notice {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(crate::i18n::t(&app.config.lang, "calib_release"))
                        .size(14.0)
                        .color(Theme::INFO),
                );
            });
            ui.add_space(15.0);
        }
    } else if app.current_travel_ratio > 0.0 {
        let ratio = app.current_travel_ratio.clamp(0.0, 1.0);
        let pct = (ratio * 100.0).round() as i32;
        ui.vertical_centered(|ui| {
            ui.add(
                egui::ProgressBar::new(ratio)
                    .text(format!("{}%", pct))
                    .fill(if ratio >= 0.85 {
                        Theme::SUCCESS
                    } else {
                        Theme::ACCENT
                    })
                    .animate(true)
                    .desired_width(240.0),
            );
            ui.label(
                egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_hold_confirm"))
                    .small()
                    .weak(),
            );
        });
        ui.add_space(15.0);
    }

    draw_xbox_controller(ui, None, Some(xbox_axis));

    ui.add_space(15.0);
    app.handle_calibration_input();
}

fn render_navigation_footer(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() / 2.0 - 160.0).max(0.0));

        let can_go_back = match app.calib_step {
            CalibStep::Buttons(i) => i > 0,
            CalibStep::Axes(_) => true,
            CalibStep::Review => true,
            _ => false,
        };

        if ui
            .add_enabled(
                can_go_back,
                egui::Button::new(crate::i18n::t(&app.config.lang, "btn_back_step"))
                    .min_size(egui::vec2(100.0, 32.0)),
            )
            .clicked()
        {
            app.prev_calibration_step();
        }

        ui.add_space(8.0);

        let can_skip = matches!(app.calib_step, CalibStep::Buttons(_) | CalibStep::Axes(_));

        if ui
            .add_enabled(
                can_skip,
                egui::Button::new(crate::i18n::t(&app.config.lang, "btn_skip_step"))
                    .min_size(egui::vec2(100.0, 32.0)),
            )
            .clicked()
        {
            app.skip_calibration_step();
        }

        ui.add_space(8.0);

        if ui
            .button(crate::i18n::t(&app.config.lang, "btn_cancel_step"))
            .clicked()
        {
            app.reset_calibration();
        }
    });
}

fn render_review(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(20.0);
    ui.label(
        egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_mapping_done"))
            .strong()
            .size(32.0)
            .color(Theme::SUCCESS),
    );
    ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_mapping_done_sub")).weak());

    ui.add_space(25.0);

    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() / 2.0 - 160.0).max(0.0));
        ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_profile_name")).strong());
        ui.add(
            egui::TextEdit::singleline(&mut app.profile_name)
                .desired_width(200.0)
                .margin(egui::vec2(8.0, 6.0)),
        );
    });

    ui.add_space(25.0);

    if ui
        .add(
            egui::Button::new(crate::i18n::t(&app.config.lang, "btn_save_profile_action"))
                .min_size(egui::vec2(200.0, 50.0))
                .fill(Theme::ACCENT),
        )
        .clicked()
    {
        app.save_profile();
    }
}

fn render_progress_bar(app: &mut App, ui: &mut egui::Ui) {
    let total = (app.calib_btns.len() + app.calib_axes.len()).max(1);
    let (current, max_steps) = match app.calib_step {
        CalibStep::Buttons(i) => (i, total),
        CalibStep::Axes(i) => (app.calib_btns.len() + i, total),
        CalibStep::Review => (total, total),
        _ => (0, total),
    };

    let progress = current as f32 / max_steps as f32;
    ui.add(
        egui::ProgressBar::new(progress)
            .show_percentage()
            .fill(Theme::ACCENT)
            .corner_radius(egui::CornerRadius::same(4)),
    );
}
