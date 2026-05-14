use eframe::egui;
use crate::gui::app::App;
use crate::gui::theme::Theme;
use crate::gui::widgets::controller_view::draw_xbox_controller;
use crate::gui::types::CalibStep;

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

        ui.add_space(30.0);
        render_navigation_footer(app, ui);
    });

    // Consola de Diagnóstico en tiempo real (DEBUG)
    render_debug_console(app, ui);
}

fn render_debug_console(app: &App, ui: &mut egui::Ui) {
    ui.add_space(20.0);
    egui::Frame::default()
        .fill(Theme::BG_DEEP)
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("⌨ CONSOLA DE DIAGNÓSTICO").weak().size(10.0));
            egui::ScrollArea::vertical().id_salt("debug_console").max_height(80.0).show(ui, |ui| {
                for log in &app.calib_logs {
                    ui.label(egui::RichText::new(log).monospace().size(10.0).color(Theme::INFO));
                }
            });
        });
}

fn render_button_step(app: &mut App, ui: &mut egui::Ui, index: usize) {
    let (label, hint, xbox_key) = {
        let slot = &app.calib_btns[index];
        (slot.label.clone(), slot.hint.clone(), slot.xbox_key)
    };
    
    ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "calib_step_buttons")).strong().color(Theme::ACCENT).size(14.0));
    ui.add_space(5.0);
    
    // Dibujar el mando resaltando el botón actual
    draw_xbox_controller(ui, Some(xbox_key), None);
    
    ui.add_space(15.0);
    ui.label(egui::RichText::new(label).size(28.0).strong().color(egui::Color32::WHITE));
    ui.label(egui::RichText::new(hint).weak().size(16.0));
    
    ui.add_space(20.0);
    app.handle_calibration_input();
}

fn render_axis_step(app: &mut App, ui: &mut egui::Ui, index: usize) {
    let axis = &app.calib_axes[index];
    
    ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "calib_step_axes")).strong().color(Theme::INFO).size(14.0));
    ui.add_space(5.0);
    
    ui.label(egui::RichText::new(&axis.label).size(28.0).strong().color(egui::Color32::WHITE));
    ui.label(egui::RichText::new(&axis.direction_label).size(20.0).color(Theme::ACCENT));
    ui.add_space(20.0);

    // Barra de Progreso de Confirmación (Estilo Steam)
    if app.waiting_for_release {
        ui.vertical_centered(|ui| {
            ui.add(egui::ProgressBar::new(1.0)
                .text("¡SUELTA EL JOYSTICK!")
                .animate(true)
                .desired_width(260.0));
            ui.label(egui::RichText::new("Esperando retorno al centro...").small().color(Theme::INFO));
        });
        ui.add_space(20.0);
    } else if let Some(since) = app.detection_since {
        let progress = (since.elapsed().as_secs_f32() / 0.3).min(1.0);
        ui.vertical_centered(|ui| {
            ui.add(egui::ProgressBar::new(progress)
                .text(format!("{}%", (progress * 100.0) as i32))
                .animate(true)
                .desired_width(200.0));
            ui.label(egui::RichText::new("Sostén para confirmar...").small().weak());
        });
        ui.add_space(20.0);
    }

    crate::gui::widgets::controller_view::draw_xbox_controller(
        ui,
        None,
        Some(axis.xbox_axis),
    );
    
    ui.add_space(20.0);
    app.handle_calibration_input();
}

fn render_navigation_footer(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add_space(ui.available_width() / 2.0 - 160.0);
        
        // Botón de Retroceder
        let can_go_back = match app.calib_step {
            CalibStep::Buttons(i) => i > 0,
            CalibStep::Axes(_) => true,
            CalibStep::Review => true,
            _ => false,
        };

        if ui.add_enabled(can_go_back, egui::Button::new("⬅ Retroceder").min_size(egui::vec2(100.0, 32.0))).clicked() {
            app.prev_calibration_step();
        }

        ui.add_space(8.0);

        // Botón de Omitir
        let can_skip = match app.calib_step {
            CalibStep::Buttons(_) => true,
            _ => false,
        };

        if ui.add_enabled(can_skip, egui::Button::new("⏭ Omitir").min_size(egui::vec2(100.0, 32.0))).clicked() {
            app.skip_calibration_step();
        }

        ui.add_space(8.0);

        if ui.button("✖ Cancelar").clicked() {
            app.reset_calibration();
        }
    });
}

fn render_review(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(30.0);
    ui.label(egui::RichText::new("✨ Mapeo Completado").strong().size(32.0).color(Theme::SUCCESS));
    ui.label(egui::RichText::new("Revisa que todos los botones respondan correctamente antes de guardar.").weak());
    
    ui.add_space(40.0);
    
    if ui.add(egui::Button::new("💾 GUARDAR PERFIL").min_size(egui::vec2(200.0, 50.0)).fill(Theme::ACCENT)).clicked() {
        app.save_profile();
    }
}

fn render_progress_bar(app: &mut App, ui: &mut egui::Ui) {
    let (current, total) = match app.calib_step {
        CalibStep::Buttons(i) => (i, app.calib_btns.len() + app.calib_axes.len()),
        CalibStep::Axes(i) => (app.calib_btns.len() + i, app.calib_btns.len() + app.calib_axes.len()),
        CalibStep::Review => (100, 100),
        _ => (0, 1),
    };

    let progress = current as f32 / total as f32;
    ui.add(egui::ProgressBar::new(progress)
        .show_percentage()
        .fill(Theme::ACCENT)
        .corner_radius(egui::CornerRadius::same(4)));
}
