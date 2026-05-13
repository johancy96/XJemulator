use eframe::egui;
use crate::gui::app::App;
use crate::gui::theme::Theme;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(10.0);
    ui.label(egui::RichText::new(crate::i18n::t(&app.config.lang, "lbl_saved_profiles")).strong().size(20.0));
    ui.add_space(10.0);

    let mut action = None;

    egui::ScrollArea::vertical().id_salt("profiles_view_scroll").show(ui, |ui| {
        for p in &app.saved_profiles {
            let active = app.active_profile.as_deref() == Some(p.as_str());
            
            egui::Frame::default()
                .fill(Theme::BG_CARD)
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(12.0)
                .stroke(if active { egui::Stroke::new(1.0, Theme::ACCENT) } else { egui::Stroke::NONE })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(p).strong());
                            ui.label(egui::RichText::new("Perfil de mapeo TOML").size(10.0).weak());
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("👁").on_hover_text("Ver/Editar").clicked() {
                                action = Some(ProfileAction::Edit(p.clone()));
                            }
                            if active {
                                ui.colored_label(Theme::ACCENT, "ACTIVO");
                            } else {
                                if ui.button("Cargar").clicked() {
                                    action = Some(ProfileAction::Load(p.clone()));
                                }
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
            if let Ok(content) = std::fs::read_to_string(&p) {
                app.editing_profile = Some((p, content));
            }
        }
        None => {}
    }

    // Ventana flotante de edición si hay un perfil seleccionado
    if let Some((ref name, ref mut content)) = app.editing_profile {
        let mut save_clicked = false;
        let mut close_clicked = false;
        
        let mut open = true;
        egui::Window::new(format!("Editor: {}", name))
            .open(&mut open)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.add_space(4.0);
                egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY));
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new("💾 Guardar Cambios").color(Theme::SUCCESS)).clicked() {
                        save_clicked = true;
                    }
                    if ui.button("Cerrar").clicked() {
                        close_clicked = true;
                    }
                });
            });
        
        if save_clicked {
            let _ = std::fs::write(&name, content);
            app.status_msg = Some(format!("✓ Guardado: {}", name));
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
