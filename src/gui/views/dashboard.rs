use eframe::egui;
use crate::gui::app::App;
use crate::gui::theme::Theme;
use crate::gui::widgets::device_card::draw_device_card;
use crate::gui::types::CalibStep;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    let w = ui.available_width();
    
    // Alerta de Permisos si fallan
    if !app.uinput_ok {
        render_permissions_warning(app, ui);
    }

    ui.add_space(5.0);
    
    egui::SidePanel::left("dashboard_devices")
        .resizable(false)
        .exact_width(w * 0.3)
        .frame(egui::Frame::NONE.fill(Theme::BG_SIDEBAR).inner_margin(12.0))
        .show_inside(ui, |ui| {
            render_device_list(app, ui);
        });

    egui::SidePanel::right("dashboard_profiles")
        .resizable(false)
        .exact_width(w * 0.3)
        .frame(egui::Frame::NONE.fill(Theme::BG_SIDEBAR).inner_margin(12.0))
        .show_inside(ui, |ui| {
            render_profile_section(app, ui);
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.inner_margin(20.0))
        .show_inside(ui, |ui| {
            render_main_content(app, ui);
        });
}

fn render_device_list(app: &mut App, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_detected_pads")).strong().size(16.0));
    ui.add_space(8.0);

    if app.gamepads.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.label(crate::i18n::t(&app.config.lang, "lbl_no_pads"));
        });
    } else {
        let mut action = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, gp) in app.gamepads.iter().enumerate() {
                let selected = app.selected == Some(i);
                let emulating = app.emulators.contains_key(&gp.path);
                
                let (clicked, start, stop) = draw_device_card(ui, &gp.name, &gp.path, selected, emulating);
                
                if clicked { action = Some(DeviceAction::Select(i)); }
                if start { action = Some(DeviceAction::Start(gp.path.clone())); }
                if stop { action = Some(DeviceAction::Stop(gp.path.clone())); }
                ui.add_space(8.0);
            }
        });

        match action {
            Some(DeviceAction::Select(i)) => app.select_gamepad(i),
            Some(DeviceAction::Start(p)) => app.start_emulator(p),
            Some(DeviceAction::Stop(p)) => app.stop_emulator(&p),
            None => {}
        }
    }

    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        if ui.button(crate::i18n::t(&app.config.lang, "btn_refresh_pads")).clicked() {
            app.gamepads = crate::scanner::scan_gamepads();
        }
    });
}

fn render_profile_section(app: &mut App, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_saved_profiles")).strong().size(16.0));
    ui.add_space(8.0);

    egui::ScrollArea::vertical().id_salt("profiles_scroll").max_height(300.0).show(ui, |ui| {
        let mut to_load = None;
        let mut to_delete = None;
        
        for p in &app.saved_profiles {
            let active = app.active_profile.as_deref() == Some(p.as_str());
            
            egui::Frame::default()
                .fill(if active { Theme::BG_DEEP } else { egui::Color32::TRANSPARENT })
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(active, p).clicked() {
                            to_load = Some(p.clone());
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑").clicked() { to_delete = Some(p.clone()); }
                        });
                    });
                });
            ui.add_space(4.0);
        }
        
        if let Some(p) = to_load { app.load_profile_from_path(&p); }
        if let Some(p) = to_delete { app.delete_profile(&p); }
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(10.0);
    
    render_raw_monitor(app, ui);
}

fn render_raw_monitor(app: &mut App, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_raw_monitor")).strong());
    ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_raw_sub")).size(10.0).weak());
    ui.add_space(4.0);

    if let Ok(cap) = app.raw_capture.lock() {
        if cap.axis_values.is_empty() && cap.pressed_keys.is_empty() {
            ui.label(crate::i18n::t(&app.config.lang, "lbl_move_pad"));
        } else {
            egui::ScrollArea::vertical().id_salt("raw_monitor").max_height(150.0).show(ui, |ui| {
                for &key in &cap.pressed_keys {
                    let name = format!("{:?}", key);
                    ui.colored_label(Theme::ACCENT, format!("[ {} ]", name));
                }
                
                if !cap.axis_values.is_empty() {
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    for (&axis, &val) in &cap.axis_values {
                        if val.abs() > 100 { // Umbral de visualización
                            let name = format!("{:?}", axis);
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", name));
                                ui.colored_label(Theme::INFO, val.to_string());
                            });
                        }
                    }
                }
            });
        }
    }
}

fn render_permissions_warning(_app: &mut App, ui: &mut egui::Ui) {
    egui::Frame::default()
        .fill(Theme::ERROR.gamma_multiply(0.2))
        .stroke(egui::Stroke::new(1.0, Theme::ERROR))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠️").size(20.0));
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Permisos udev no detectados").strong());
                    ui.label(egui::RichText::new("La emulación fallará sin permisos de escritura en /dev/uinput.").size(11.0));
                });
            });
        });
    ui.add_space(10.0);
}

enum DeviceAction {
    Select(usize),
    Start(String),
    Stop(String),
}

fn render_main_content(app: &mut App, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        if let Some(idx) = app.selected {
            let gp = &app.gamepads[idx];
            ui.label(egui::RichText::new(&gp.name).size(24.0).strong());
            ui.label(egui::RichText::new(&gp.path).weak());
            ui.add_space(30.0);
            
            if app.emulators.contains_key(&gp.path) {
                ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_emulating_active")).color(Theme::SUCCESS).strong());
            } else {
                if ui.button(egui::RichText::new(crate::i18n::t(&app.config.lang, "btn_start_calib")).size(18.0)).clicked() {
                    app.reset_calibration();
                    app.calib_step = CalibStep::Buttons(0);
                }
            }
        } else {
            ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_select_to_start")).weak());
        }
    });
}
