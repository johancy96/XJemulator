use crate::gui::app::App;
use crate::gui::theme::Theme;
use eframe::egui;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(10.0);
    ui.label(
        egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_saved_profiles"))
            .strong()
            .size(20.0),
    );
    ui.add_space(10.0);

    let mut action = None;

    egui::ScrollArea::vertical()
        .id_salt("profiles_view_scroll")
        .show(ui, |ui| {
            for p in &app.saved_profiles {
                let active = app.active_profile.as_deref() == Some(p.as_str());

                egui::Frame::default()
                    .fill(Theme::BG_CARD)
                    .corner_radius(egui::CornerRadius::same(12))
                    .inner_margin(12.0)
                    .stroke(if active {
                        egui::Stroke::new(1.0_f32, Theme::ACCENT)
                    } else {
                        egui::Stroke::NONE
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(p).strong());
                                ui.label(
                                    egui::RichText::new(crate::i18n::t(
                                        &app.config.lang,
                                        "lbl_toml_subtitle",
                                    ))
                                    .size(10.0)
                                    .weak(),
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .button("👁")
                                    .on_hover_text(crate::i18n::t(&app.config.lang, "tooltip_view"))
                                    .clicked()
                                {
                                    action = Some(ProfileAction::Edit(p.clone()));
                                }
                                if active {
                                    ui.colored_label(
                                        Theme::ACCENT,
                                        crate::i18n::t(&app.config.lang, "lbl_active_badge"),
                                    );
                                } else if ui
                                    .button(crate::i18n::t(&app.config.lang, "btn_load_profile"))
                                    .clicked()
                                {
                                    action = Some(ProfileAction::Load(p.clone()));
                                }
                            });
                        });
                    });
                ui.add_space(8.0);
            }
        });

    match action {
        Some(ProfileAction::Load(p)) => app.load_profile_from_path(&p),
        Some(ProfileAction::Edit(p)) => {
            let full_path = crate::paths::AppPaths::profile_path(&p);
            if let Ok(content) = std::fs::read_to_string(full_path) {
                app.editing_profile = Some((p, content));
            }
        }
        None => {}
    }

    if let Some((ref name, ref mut content)) = app.editing_profile.clone() {
        let mut edited_content = content.clone();
        let mut save_clicked = false;
        let mut close_clicked = false;
        let mut open = true;

        egui::Window::new(format!("Editor: {}", name))
            .open(&mut open)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut edited_content)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY),
                        );
                    });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            egui::RichText::new(crate::i18n::t(
                                &app.config.lang,
                                "btn_save_changes",
                            ))
                            .color(Theme::SUCCESS),
                        )
                        .clicked()
                    {
                        save_clicked = true;
                    }
                    if ui
                        .button(crate::i18n::t(&app.config.lang, "btn_close"))
                        .clicked()
                    {
                        close_clicked = true;
                    }
                });
            });

        if let Some((_, ref mut orig_content)) = app.editing_profile {
            *orig_content = edited_content.clone();
        }

        if save_clicked {
            let full_path = crate::paths::AppPaths::profile_path(name);
            if std::fs::write(&full_path, &edited_content).is_ok() {
                app.refresh_profiles_cache();
                if app.active_profile.as_deref() == Some(name.as_str()) {
                    app.load_profile_from_path(name);
                }
                app.status_msg = Some(format!("✓ Guardado: {}", name));
            }
        }
        if close_clicked || !open {
            app.editing_profile = None;
        }
    }
}

enum ProfileAction {
    Load(String),
    Edit(String),
}
