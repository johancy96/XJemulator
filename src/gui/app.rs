use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use eframe::egui;
use tracing::{error, info};

use super::udev_setup::{self, UdevStatus};
use crate::mapper::Mapper;
use crate::scanner::GamepadInfo;
use crate::virtual_device::VirtualXbox360;

// ─────────────────────────────────────────────
// SHARED INPUT STATE  (reader thread ↔ GUI)
// ─────────────────────────────────────────────

use crate::gui::backend::*;
use crate::gui::types::*;

// ─────────────────────────────────────────────
// APPLICATION STATE
// ─────────────────────────────────────────────

pub struct App {
    gamepads: Vec<GamepadInfo>,
    selected: Option<usize>,

    raw_capture: Arc<Mutex<RawCapture>>,
    reader_running: Arc<AtomicBool>,

    emulators: HashMap<String, Arc<AtomicBool>>,

    mapper: Mapper,
    active_profile: Option<String>,

    calib_step: CalibStep,
    calib_btns: Vec<BtnSlot>,
    calib_axes: Vec<AxisSlot>,
    profile_name: String,
    calib_msg: Option<String>,

    /// Axis values captured at the START of each calibration step (rest state).
    /// Axis detection uses delta-from-resting so constant offsets don't false-trigger.
    axis_resting: HashMap<String, i32>,
    /// Set of raw axis names already assigned    /// Axes currently in active range — excludes these from detection to prevent
    /// one physical axis being assigned to two Xbox targets in the same calibration.
    axes_used: HashSet<String>,

    /// Cooldown timer: after detecting, block new detection until this instant.
    cooldown_until: Option<std::time::Instant>,

    /// When Some, we detected an axis but are waiting for it to return to neutral
    /// AND for the minimum cooldown to expire before advancing to `next_step`.
    /// This prevents the RIGHT stick triggering while mapping the LEFT stick.
    calib_release_watch: Option<(String, usize)>, // (raw_axis_name, next_step_index)

    /// If Some, show profile editor for this file
    editing_profile: Option<(String, String)>, // (filename, content)

    saved_profiles: Vec<String>,
    udev_status: UdevStatus,
    status_msg: Option<String>,

    pub config: crate::config::AppConfig,
}

impl App {
    pub fn new() -> Self {
        let gamepads = crate::scanner::scan_gamepads();
        let saved_profiles = scan_profiles();
        let (mapper, active_profile) = load_best_profile(&saved_profiles);
        let config = crate::config::AppConfig::load();

        Self {
            gamepads,
            selected: None,
            raw_capture: Arc::new(Mutex::new(RawCapture::default())),
            reader_running: Arc::new(AtomicBool::new(false)),
            emulators: HashMap::new(),
            mapper,
            active_profile,
            calib_step: CalibStep::Idle,
            calib_btns: default_btn_slots(&config.lang),
            calib_axes: default_axis_slots(&config.lang),
            profile_name: "mi_mando".into(),
            calib_msg: None,
            axis_resting: HashMap::new(),
            axes_used: HashSet::new(),
            cooldown_until: None,
            calib_release_watch: None,
            editing_profile: None,
            saved_profiles,
            udev_status: UdevStatus::check(),
            status_msg: None,
            config,
        }
    }

    // ── Reader ────────────────────────────────

    fn start_reader(&mut self, path: String) {
        self.stop_reader();
        if let Ok(mut cap) = self.raw_capture.lock() {
            *cap = RawCapture::default();
        }
        let capture = self.raw_capture.clone();
        let running = self.reader_running.clone();
        running.store(true, std::sync::atomic::Ordering::SeqCst);
        std::thread::spawn(move || raw_reader_loop(path, capture, running));
        info!("Lector raw iniciado");
    }

    fn stop_reader(&mut self) {
        self.reader_running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        std::thread::sleep(std::time::Duration::from_millis(60));
    }

    fn select_gamepad(&mut self, idx: usize) {
        self.selected = Some(idx);
        if let Some(gp) = self.gamepads.get(idx) {
            // Only start reading if it's not currently being emulated
            if !self.emulators.contains_key(&gp.path) {
                self.start_reader(gp.path.clone());
            } else {
                self.stop_reader(); // Free the UI from old reads
            }
        }
    }

    // ── Emulator ──────────────────────────────

    fn start_emulator(&mut self, path: String) {
        if self.emulators.contains_key(&path) {
            return;
        }
        if self.emulators.len() >= 4 {
            self.status_msg = Some("Límite de 4 emuladores alcanzado".into());
            return;
        }

        // If this gamepad was the currently selected one, stop the raw_reader to release the lock!
        if let Some(idx) = self.selected {
            if let Some(gp) = self.gamepads.get(idx) {
                if gp.path == path {
                    self.stop_reader();
                }
            }
        }

        match VirtualXbox360::new() {
            Ok(vx) => {
                let running = Arc::new(AtomicBool::new(true));
                self.emulators.insert(path.clone(), running.clone());

                let mapper = self.mapper.clone();
                // Emulators get an isolated dummy capture so they don't break the UI calibration state
                let dummy_cap = Arc::new(Mutex::new(RawCapture::default()));

                let thread_path = path.clone();
                std::thread::spawn(move || {
                    emulation_loop(thread_path, dummy_cap, running, mapper, vx)
                });
                self.status_msg = Some(format!("✓ Emulador iniciado para {}", path));
                info!("Emulador activo: {}", path);
            }
            Err(e) => {
                error!("Error creando emulador para {}: {}", path, e);
                self.status_msg = Some(format!("Error: {}", e));
            }
        }
    }

    fn stop_emulator(&mut self, path: &str) {
        if let Some(running) = self.emulators.remove(path) {
            running.store(false, std::sync::atomic::Ordering::SeqCst);
            self.status_msg = Some(format!("Emulador detenido: {}", path));
            std::thread::sleep(std::time::Duration::from_millis(150)); // allow thread to exit

            // If the user's currently selected gamepad was this one, restart its raw_reader
            if let Some(idx) = self.selected {
                if self.gamepads.get(idx).map(|g| g.path.as_str()) == Some(path) {
                    self.start_reader(path.to_string());
                }
            }
        }
    }

    fn stop_all_emulators(&mut self) {
        for (_, running) in self.emulators.drain() {
            running.store(false, std::sync::atomic::Ordering::SeqCst);
        }
        self.status_msg = Some("Todos los emuladores detenidos".into());
        std::thread::sleep(std::time::Duration::from_millis(150));

        if let Some(idx) = self.selected {
            if let Some(gp) = self.gamepads.get(idx) {
                self.start_reader(gp.path.clone());
            }
        }
    }

    // ── Calibration helpers ───────────────────

    fn reset_calibration(&mut self) {
        self.calib_step = CalibStep::Idle;
        self.calib_btns = default_btn_slots(&self.config.lang);
        self.calib_axes = default_axis_slots(&self.config.lang);
        self.calib_msg = None;
        self.axis_resting = HashMap::new();
        self.axes_used = HashSet::new();
        self.cooldown_until = None;
        self.calib_release_watch = None;
        if let Ok(mut cap) = self.raw_capture.lock() {
            cap.key_queue.clear();
        }
    }

    /// Returns true if still in cooldown (block detection).
    fn in_cooldown(&self) -> bool {
        self.cooldown_until
            .map_or(false, |t| std::time::Instant::now() < t)
    }

    /// Remaining cooldown as seconds, for display
    fn cooldown_remaining_secs(&self) -> f32 {
        self.cooldown_until
            .map(|t| {
                t.saturating_duration_since(std::time::Instant::now())
                    .as_secs_f32()
            })
            .unwrap_or(0.0)
    }

    /// Start a 700ms cooldown for BUTTONS and flush the key queue.
    fn start_btn_cooldown(&mut self) {
        self.cooldown_until =
            Some(std::time::Instant::now() + std::time::Duration::from_millis(700));
        if let Ok(mut cap) = self.raw_capture.lock() {
            cap.key_queue.clear();
        }
    }

    /// Start a 1000ms minimum cooldown for AXES (the physical release adds extra time).
    fn start_axis_cooldown(&mut self) {
        self.cooldown_until =
            Some(std::time::Instant::now() + std::time::Duration::from_millis(1000));
    }

    /// Called when entering a new axis calibration step.
    /// Captures resting values so we detect DELTA from rest, not absolute values.
    fn capture_resting(&mut self) {
        if let Ok(cap) = self.raw_capture.lock() {
            self.axis_resting = cap.axis_values.clone();
        }
    }

    fn save_profile(&mut self) {
        let name = self.profile_name.trim().to_string();
        let fname = if name.ends_with(".toml") {
            name.clone()
        } else {
            format!("{}.toml", name)
        };
        let toml = generate_profile_toml(
            &self.calib_btns,
            &self.calib_axes,
            &name,
            &self.axis_resting,
        );

        match std::fs::write(&fname, &toml) {
            Ok(_) => {
                self.calib_msg = Some(format!("✓ Guardado: {}", fname));
                if let Ok(prof) = toml::from_str::<crate::mapper::MappingProfile>(&toml) {
                    if let Ok(m) = Mapper::from_profile(&prof) {
                        self.mapper = m;
                        self.active_profile = Some(fname.clone());
                    }
                }
                if !self.saved_profiles.contains(&fname) {
                    self.saved_profiles.push(fname.clone());
                    self.saved_profiles.sort();
                }
                self.status_msg = Some(format!("Perfil guardado: {}", fname));
                self.reset_calibration();
                if self.udev_status.all_ok() {
                    if let Some(idx) = self.selected {
                        if let Some(gp) = self.gamepads.get(idx).cloned() {
                            self.start_emulator(gp.path);
                        }
                    }
                }
            }
            Err(e) => {
                self.calib_msg = Some(format!("Error: {}", e));
            }
        }
    }
}

// ─────────────────────────────────────────────
// EGUI – Main frame
// ─────────────────────────────────────────────

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // ── Top bar ───────────────────────────
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(crate::i18n::t(&self.config.lang, "app_title"))
                        .strong()
                        .size(18.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    let lang_btn_text = if self.config.lang == crate::i18n::Lang::Es {
                        "🌐 EN"
                    } else {
                        "🌐 ES"
                    };
                    if ui.button(lang_btn_text).clicked() {
                        self.config.lang = if self.config.lang == crate::i18n::Lang::Es {
                            crate::i18n::Lang::En
                        } else {
                            crate::i18n::Lang::Es
                        };
                        self.config.save();
                    }
                    if !self.emulators.is_empty() {
                        if ui
                            .button(
                                egui::RichText::new(crate::i18n::t(
                                    &self.config.lang,
                                    "btn_stop_all",
                                ))
                                .color(egui::Color32::from_rgb(220, 80, 80)),
                            )
                            .clicked()
                        {
                            self.stop_all_emulators();
                        }
                    }
                });
            });
            ui.add_space(4.0);
        });

        // ── Status bar ────────────────────────
        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if let Some(ref msg) = self.status_msg {
                    ui.label(egui::RichText::new(msg).size(12.0).weak());
                }
                if !self.emulators.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.colored_label(
                            egui::Color32::from_rgb(80, 200, 80),
                            format!(
                                "● {} {}",
                                self.emulators.len(),
                                crate::i18n::t(&self.config.lang, "lbl_active_emulators")
                            ),
                        );
                    });
                }
            });
            ui.add_space(3.0);
        });

        // ── Main 3-column layout ──────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            let w = ui.available_width();
            egui::SidePanel::left("left_panel")
                .resizable(false)
                .exact_width(w * 0.23)
                .show_inside(ui, |ui| {
                    self.ui_left(ui);
                });

            egui::SidePanel::right("right_panel")
                .resizable(false)
                .exact_width(w * 0.28)
                .show_inside(ui, |ui| {
                    self.ui_right(ui);
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.ui_center(ui);
            });
        });
    }
}

// ─────────────────────────────────────────────
// LEFT – Devices & System
// ─────────────────────────────────────────────
impl App {
    fn ui_left(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_detected_pads"))
                .strong()
                .size(13.0),
        );
        ui.separator();

        if self.gamepads.is_empty() {
            ui.colored_label(
                egui::Color32::GRAY,
                crate::i18n::t(&self.config.lang, "lbl_no_pads"),
            );
        } else {
            let mut to_select: Option<usize> = None;
            let mut to_start: Option<String> = None;
            let mut to_stop: Option<String> = None;

            for (i, gp) in self.gamepads.iter().enumerate() {
                let active = self.selected == Some(i);
                let is_emulating = self.emulators.contains_key(&gp.path);
                ui.horizontal(|ui| {
                    if active {
                        ui.colored_label(egui::Color32::LIGHT_GREEN, "●");
                    } else {
                        ui.label(" ");
                    }
                    if ui.selectable_label(active, &gp.name).clicked() {
                        to_select = Some(i);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_emulating {
                            if ui
                                .button(
                                    egui::RichText::new("⏹")
                                        .color(egui::Color32::from_rgb(220, 80, 80)),
                                )
                                .on_hover_text(crate::i18n::t(&self.config.lang, "tooltip_stop"))
                                .clicked()
                            {
                                to_stop = Some(gp.path.clone());
                            }
                        } else {
                            if self.emulators.len() < 4
                                && self.udev_status.all_ok()
                                && self.active_profile.is_some()
                                && self.calib_step == CalibStep::Idle
                            {
                                if ui
                                    .button(
                                        egui::RichText::new("▶")
                                            .color(egui::Color32::from_rgb(80, 200, 80)),
                                    )
                                    .on_hover_text(crate::i18n::t(
                                        &self.config.lang,
                                        "tooltip_play",
                                    ))
                                    .clicked()
                                {
                                    to_start = Some(gp.path.clone());
                                }
                            }
                        }
                    });
                });
                ui.label(egui::RichText::new(&gp.path).size(10.0).weak());
            }
            if let Some(i) = to_select {
                self.select_gamepad(i);
            }
            if let Some(p) = to_start {
                self.start_emulator(p);
            }
            if let Some(p) = to_stop {
                self.stop_emulator(&p);
            }
        }

        ui.add_space(12.0);
        if ui
            .small_button(crate::i18n::t(&self.config.lang, "btn_refresh_pads"))
            .clicked()
        {
            self.gamepads = crate::scanner::scan_gamepads();
        }

        // udev status
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_sys_status"))
                .strong()
                .size(13.0),
        );
        ui.separator();

        if self.udev_status.all_ok() {
            ui.colored_label(
                egui::Color32::LIGHT_GREEN,
                crate::i18n::t(&self.config.lang, "udev_ok"),
            );
            ui.add_space(4.0);
            if ui
                .small_button(crate::i18n::t(&self.config.lang, "btn_uninstall_rules"))
                .clicked()
            {
                match udev_setup::try_uninstall_rules() {
                    Ok(_) => {
                        self.udev_status = UdevStatus::check();
                        self.status_msg = Some("Reglas udev desinstaladas".into());
                    }
                    Err(e) => {
                        self.status_msg = Some(format!("Error: {}", e));
                    }
                }
            }
        } else {
            ui.colored_label(
                egui::Color32::YELLOW,
                crate::i18n::t(&self.config.lang, "udev_warn"),
            );
            ui.label(
                egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_udev_req"))
                    .size(11.0)
                    .weak(),
            );
            ui.add_space(6.0);
            if ui
                .button(crate::i18n::t(&self.config.lang, "btn_install_rules"))
                .clicked()
            {
                match udev_setup::try_install_rules() {
                    Ok(_) => {
                        self.udev_status = UdevStatus::check();
                        self.status_msg = Some("✓ Reglas udev instaladas".into());
                    }
                    Err(e) => {
                        self.status_msg = Some(format!("Error: {}", e));
                    }
                }
            }
        }

        if let Some(ref ap) = self.active_profile.clone() {
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_active_profile"))
                    .size(11.0)
                    .weak(),
            );
            ui.label(egui::RichText::new(ap).size(11.0).strong());
        }
    }
}

// ─────────────────────────────────────────────
// RIGHT – Profiles & RAW monitor
// ─────────────────────────────────────────────
impl App {
    fn ui_right(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_saved_profiles"))
                .strong()
                .size(13.0),
        );
        ui.separator();

        // Profile list with load, view, delete
        let mut to_load: Option<String> = None;
        let mut to_view: Option<String> = None;
        let mut to_delete: Option<String> = None;

        egui::ScrollArea::vertical()
            .id_salt("saved_profiles_scroll")
            .max_height(200.0)
            .show(ui, |ui| {
                for p in &self.saved_profiles {
                    let active = self.active_profile.as_deref() == Some(p.as_str());
                    ui.horizontal(|ui| {
                        if active {
                            ui.colored_label(egui::Color32::from_rgb(80, 200, 100), "●");
                        } else {
                            ui.label(" ");
                        }
                        if ui.selectable_label(active, p).clicked() {
                            to_load = Some(p.clone());
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .small_button("🗑")
                                .on_hover_text(crate::i18n::t(&self.config.lang, "tooltip_delete"))
                                .clicked()
                            {
                                to_delete = Some(p.clone());
                            }
                            if ui
                                .small_button("👁")
                                .on_hover_text(crate::i18n::t(&self.config.lang, "tooltip_view"))
                                .clicked()
                            {
                                to_view = Some(p.clone());
                            }
                        });
                    });
                }
            });

        // Load profile
        if let Some(ref p) = to_load {
            if let Ok(content) = std::fs::read_to_string(p) {
                if let Ok(prof) = toml::from_str::<crate::mapper::MappingProfile>(&content) {
                    if let Ok(m) = Mapper::from_profile(&prof) {
                        self.mapper = m;
                        self.active_profile = Some(p.clone());
                        self.status_msg = Some(format!("Cargado: {}", p));
                    }
                }
            }
        }

        // View profile
        if let Some(ref p) = to_view {
            if let Ok(content) = std::fs::read_to_string(p) {
                self.editing_profile = Some((p.clone(), content));
            }
        }

        // Delete profile
        if let Some(ref p) = to_delete {
            if let Err(e) = std::fs::remove_file(p) {
                self.status_msg = Some(format!("Error eliminando: {}", e));
            } else {
                if self.active_profile.as_deref() == Some(p.as_str()) {
                    self.active_profile = None;
                    self.mapper = Mapper::identity();
                }
                if self.editing_profile.as_ref().map(|(n, _)| n) == Some(p) {
                    self.editing_profile = None;
                }
                self.saved_profiles = scan_profiles();
                self.status_msg = Some(format!("Eliminado: {}", p));
            }
        }

        ui.horizontal(|ui| {
            if ui
                .small_button(crate::i18n::t(&self.config.lang, "btn_refresh"))
                .clicked()
            {
                self.saved_profiles = scan_profiles();
            }
        });

        // Profile viewer/editor window
        if let Some((ref pname, ref mut pcontent)) = self.editing_profile {
            let pname = pname.clone();
            let mut open = true;
            egui::Window::new(format!("📄 {}", pname))
                .collapsible(false)
                .resizable(true)
                .default_size([500.0, 400.0])
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("profile_editor_scroll")
                        .show(ui, |ui| {
                            ui.text_edit_multiline(pcontent);
                        });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .button(crate::i18n::t(&self.config.lang, "btn_save_changes"))
                            .clicked()
                        {
                            if let Err(e) = std::fs::write(&pname, pcontent.as_str()) {
                                self.status_msg = Some(format!("Error: {}", e));
                            } else {
                                self.status_msg = Some(format!("✓ Guardado: {}", pname));
                                // Reload if it's the active profile
                                if self.active_profile.as_deref() == Some(&pname) {
                                    if let Ok(prof) =
                                        toml::from_str::<crate::mapper::MappingProfile>(pcontent)
                                    {
                                        if let Ok(m) = Mapper::from_profile(&prof) {
                                            self.mapper = m;
                                        }
                                    }
                                }
                            }
                        }
                    });
                });
            if !open {
                self.editing_profile = None;
            }
        }

        // RAW monitor (only while emulator is off for selected gamepad)
        let is_selected_emulated = self
            .selected
            .and_then(|idx| self.gamepads.get(idx))
            .map(|gp| self.emulators.contains_key(&gp.path))
            .unwrap_or(false);

        if !is_selected_emulated {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_raw_monitor"))
                    .strong()
                    .size(13.0),
            );
            ui.separator();
            ui.label(
                egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_raw_sub"))
                    .size(10.0)
                    .weak(),
            );
            ui.add_space(4.0);

            if let Ok(cap) = self.raw_capture.lock() {
                if cap.axis_values.is_empty() && cap.pressed_keys.is_empty() {
                    ui.label(
                        egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_move_pad"))
                            .weak(),
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("raw_monitor_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for key in &cap.pressed_keys {
                                ui.colored_label(egui::Color32::LIGHT_GREEN, format!("🔘 {}", key));
                            }
                            let mut axes: Vec<_> = cap.axis_values.iter().collect();
                            axes.sort_by_key(|(k, _)| (*k).clone());
                            for (axis, val) in axes {
                                let rest = self.axis_resting.get(axis).copied().unwrap_or(0);
                                let delta = val - rest;
                                let color = if delta.abs() >= calib_delta_threshold(axis) {
                                    egui::Color32::LIGHT_BLUE
                                } else {
                                    egui::Color32::from_rgb(90, 90, 100)
                                };
                                ui.colored_label(
                                    color,
                                    format!("{} = {:+}  (Δ{:+})", axis, val, delta),
                                );
                            }
                        });
                }
            }
        }
    }
}

// ─────────────────────────────────────────────
// CENTER – Calibrator Wizard
// ─────────────────────────────────────────────
impl App {
    fn ui_center(&mut self, ui: &mut egui::Ui) {
        match self.calib_step.clone() {
            CalibStep::Idle => self.ui_calib_idle(ui),
            CalibStep::Buttons(i) => self.ui_calib_button(ui, i),
            CalibStep::Axes(i) => self.ui_calib_axis(ui, i),
            CalibStep::Review => self.ui_calib_review(ui),
        }
    }

    fn ui_calib_idle(&mut self, ui: &mut egui::Ui) {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_calib_wiz"))
                    .strong()
                    .size(22.0),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_calib_desc"))
                    .weak()
                    .size(13.0),
            );
            ui.add_space(20.0);

            ui.label(crate::i18n::t(&self.config.lang, "lbl_profile_name"));
            ui.text_edit_singleline(&mut self.profile_name);
            ui.add_space(16.0);

            let is_selected_emulated = self
                .selected
                .and_then(|idx| self.gamepads.get(idx))
                .map(|gp| self.emulators.contains_key(&gp.path))
                .unwrap_or(false);

            let can_start = self.selected.is_some() && !is_selected_emulated;

            if is_selected_emulated {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    crate::i18n::t(&self.config.lang, "warning_busy"),
                );
            } else if self.selected.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    crate::i18n::t(&self.config.lang, "warning_select"),
                );
            }

            ui.add_space(8.0);

            if ui
                .add_enabled(
                    can_start,
                    egui::Button::new(
                        egui::RichText::new(crate::i18n::t(&self.config.lang, "btn_start_calib"))
                            .size(16.0),
                    )
                    .min_size(egui::vec2(200.0, 40.0)),
                )
                .clicked()
            {
                self.reset_calibration();
                // Clear key queue before starting
                if let Ok(mut cap) = self.raw_capture.lock() {
                    cap.key_queue.clear();
                }
                self.calib_step = CalibStep::Buttons(0);
            }

            // Summary of total steps
            ui.add_space(20.0);
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(22, 22, 32))
                .corner_radius(8.0)
                .inner_margin(14.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_calib_incl"))
                            .strong(),
                    );
                    ui.label(format!(
                        "  • {} {}",
                        default_btn_slots(&self.config.lang).len(),
                        crate::i18n::t(&self.config.lang, "lbl_calib_btns")
                    ));
                    ui.label(format!(
                        "  • {} {}",
                        default_axis_slots(&self.config.lang).len(),
                        crate::i18n::t(&self.config.lang, "lbl_calib_axes")
                    ));
                    ui.label(crate::i18n::t(&self.config.lang, "lbl_calib_skip"));
                });
        });
    }

    // ── Button step ──────────────────────────

    fn ui_calib_button(&mut self, ui: &mut egui::Ui, index: usize) {
        if index >= self.calib_btns.len() {
            // Done with buttons → start axes
            // Capture resting values before first axis step
            self.capture_resting();
            self.calib_step = CalibStep::Axes(0);
            return;
        }

        let total = self.calib_btns.len();
        let progress = index as f32 / total as f32;
        ui.add(
            egui::ProgressBar::new(progress)
                .desired_width(ui.available_width())
                .text(format!(
                    "{} {}/{}",
                    crate::i18n::t(&self.config.lang, "lbl_buttons"),
                    index + 1,
                    total
                )),
        );
        ui.add_space(10.0);

        egui::Frame::default()
            .fill(egui::Color32::from_rgb(28, 28, 40))
            .corner_radius(12.0)
            .inner_margin(egui::Margin::same(24))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(crate::i18n::t(&self.config.lang, "calib_btn_press"))
                            .size(13.0)
                            .weak(),
                    );
                    ui.add_space(8.0);
                    let label = &self.calib_btns[index].label;
                    let hint = &self.calib_btns[index].hint;

                    ui.label(
                        egui::RichText::new(label.clone())
                            .size(30.0)
                            .strong()
                            .color(egui::Color32::from_rgb(255, 210, 70)),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(hint.clone())
                            .size(12.0)
                            .weak()
                            .color(egui::Color32::from_rgb(160, 160, 185)),
                    );
                    ui.add_space(10.0);

                    // Live indicator
                    if let Ok(cap) = self.raw_capture.lock() {
                        if let Some(k) = cap.pressed_keys.iter().next() {
                            ui.colored_label(
                                egui::Color32::LIGHT_GREEN,
                                format!(
                                    "{} {}",
                                    crate::i18n::t(&self.config.lang, "calib_detecting"),
                                    k
                                ),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new(crate::i18n::t(
                                    &self.config.lang,
                                    "calib_waiting",
                                ))
                                .weak()
                                .size(12.0),
                            );
                        }
                    }
                });
            });

        ui.add_space(8.0);

        // Collapsible list of already-mapped buttons
        if index > 0 {
            egui::CollapsingHeader::new(
                egui::RichText::new(format!(
                    "{} ({} {})",
                    crate::i18n::t(&self.config.lang, "lbl_mapped"),
                    index,
                    crate::i18n::t(&self.config.lang, "lbl_buttons")
                ))
                .weak()
                .size(12.0),
            )
            .id_salt("btns_done")
            .show(ui, |ui| {
                egui::Grid::new("btns_grid").striped(true).show(ui, |ui| {
                    for i in 0..index {
                        let b = &self.calib_btns[i];
                        let src = b
                            .source
                            .as_deref()
                            .unwrap_or(crate::i18n::t(&self.config.lang, "lbl_skipped"));
                        ui.label(egui::RichText::new(&b.label).size(11.0));
                        let color = if b.source.is_some() {
                            egui::Color32::LIGHT_GREEN
                        } else {
                            egui::Color32::GRAY
                        };
                        ui.colored_label(color, src);
                        ui.end_row();
                    }
                });
            });
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui
                .button(format!(
                    "{} →",
                    crate::i18n::t(&self.config.lang, "btn_skip")
                ))
                .clicked()
            {
                // Leave source as None and advance
                self.calib_step = CalibStep::Buttons(index + 1);
            }
            if index > 0
                && ui
                    .button(format!(
                        "◀ {}",
                        crate::i18n::t(&self.config.lang, "btn_back")
                    ))
                    .clicked()
            {
                self.calib_btns[index - 1].source = None;
                // Clear any pending keys
                if let Ok(mut cap) = self.raw_capture.lock() {
                    cap.key_queue.clear();
                }
                self.calib_step = CalibStep::Buttons(index - 1);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(format!(
                        "✕ {}",
                        crate::i18n::t(&self.config.lang, "btn_cancel")
                    ))
                    .clicked()
                {
                    self.reset_calibration();
                }
            });
        });

        // ── Detection (gated by cooldown) ─────────────────────────────────
        let detected = if self.in_cooldown() {
            ui.add_space(4.0);
            ui.colored_label(
                egui::Color32::from_rgb(255, 200, 60),
                format!(
                    "⏳ {} {:.1}s…",
                    crate::i18n::t(&self.config.lang, "calib_wait"),
                    self.cooldown_remaining_secs()
                ),
            );
            None
        } else {
            if let Ok(mut cap) = self.raw_capture.lock() {
                cap.key_queue.pop_front()
            } else {
                None
            }
        };

        if let Some(raw_key) = detected {
            self.calib_btns[index].source = Some(raw_key.clone());
            self.status_msg = Some(format!("✓ {} → {}", self.calib_btns[index].label, raw_key));
            self.start_btn_cooldown();
            self.calib_step = CalibStep::Buttons(index + 1);
        }
    }

    // ── Axis step ─────────────────────────────

    fn ui_calib_axis(&mut self, ui: &mut egui::Ui, index: usize) {
        if index >= self.calib_axes.len() {
            self.calib_step = CalibStep::Review;
            return;
        }

        let total = self.calib_axes.len();
        let progress = index as f32 / total as f32;
        ui.add(
            egui::ProgressBar::new(progress)
                .desired_width(ui.available_width())
                .text(format!(
                    "{} {}/{}",
                    crate::i18n::t(&self.config.lang, "lbl_axes"),
                    index + 1,
                    total
                )),
        );
        ui.add_space(10.0);

        let target = self.calib_axes[index].xbox_axis;

        // Check if this xbox_axis was already resolved in a previous slot
        let already_mapped = self.calib_axes[..index]
            .iter()
            .any(|s| s.xbox_axis == target && s.source.is_some());

        egui::Frame::default()
            .fill(egui::Color32::from_rgb(28, 28, 40))
            .corner_radius(12.0)
            .inner_margin(egui::Margin::same(24))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    if already_mapped {
                        ui.label(
                            egui::RichText::new(crate::i18n::t(
                                &self.config.lang,
                                "lbl_verify_opt",
                            ))
                            .size(13.0)
                            .weak(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(crate::i18n::t(
                                &self.config.lang,
                                "lbl_move_joystick",
                            ))
                            .size(13.0)
                            .weak(),
                        );
                    }
                    ui.add_space(6.0);
                    let label = &self.calib_axes[index].label;
                    let dir_label = &self.calib_axes[index].direction_label;

                    ui.label(
                        egui::RichText::new(label.clone())
                            .size(26.0)
                            .strong()
                            .color(egui::Color32::from_rgb(90, 190, 255)),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(dir_label.clone())
                            .size(22.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "({}: {})",
                            crate::i18n::t(&self.config.lang, "lbl_maps_to"),
                            target
                        ))
                        .size(11.0)
                        .weak(),
                    );

                    ui.add_space(10.0);

                    // Live axis delta display
                    if let Ok(cap) = self.raw_capture.lock() {
                        let mut active: Vec<(&String, i32)> = cap
                            .axis_values
                            .iter()
                            .map(|(n, &v)| {
                                let rest = self.axis_resting.get(n).copied().unwrap_or(0);
                                (n, v - rest)
                            })
                            .filter(|(_, delta)| delta.abs() > 5)
                            .collect();
                        active.sort_by_key(|(_, d)| -d.abs());

                        if active.is_empty() {
                            ui.label(
                                egui::RichText::new(crate::i18n::t(
                                    &self.config.lang,
                                    "calib_wait_move",
                                ))
                                .weak()
                                .size(12.0),
                            );
                        } else {
                            for (name, delta) in active.iter().take(5) {
                                let thr = calib_delta_threshold(name);
                                let color = if delta.abs() >= thr {
                                    egui::Color32::LIGHT_BLUE
                                } else {
                                    egui::Color32::from_rgb(120, 120, 160)
                                };
                                ui.colored_label(
                                    color,
                                    format!(
                                        "{} Δ = {:+}  ({} {:+})",
                                        name,
                                        delta,
                                        crate::i18n::t(&self.config.lang, "calib_needs"),
                                        thr
                                    ),
                                );
                            }
                        }
                    }
                });
            });

        // Summary of previously mapped axes
        if index > 0 {
            ui.add_space(6.0);
            egui::CollapsingHeader::new(
                egui::RichText::new(format!(
                    "{} ({} {})",
                    crate::i18n::t(&self.config.lang, "lbl_mapped"),
                    index,
                    crate::i18n::t(&self.config.lang, "lbl_steps")
                ))
                .weak()
                .size(12.0),
            )
            .id_salt("axes_done")
            .show(ui, |ui| {
                egui::Grid::new("axes_grid").striped(true).show(ui, |ui| {
                    for i in 0..index {
                        let a = &self.calib_axes[i];
                        let src = a
                            .source
                            .as_deref()
                            .unwrap_or(crate::i18n::t(&self.config.lang, "lbl_skipped"));
                        let inv = if a.invert { " ↕" } else { "" };
                        ui.label(egui::RichText::new(&a.label).size(10.0));
                        ui.label(egui::RichText::new(&a.direction_label).size(10.0));
                        let color = if a.source.is_some() {
                            egui::Color32::LIGHT_BLUE
                        } else {
                            egui::Color32::GRAY
                        };
                        ui.colored_label(color, format!("{}{}", src, inv));
                        ui.end_row();
                    }
                });
            });
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("Omitir →").clicked() {
                self.capture_resting(); // Reset resting for next step
                self.calib_step = CalibStep::Axes(index + 1);
            }
            if index > 0 && ui.button("◀ Atrás").clicked() {
                // Going back: unmap previous slot, remove its axis from used set
                let prev_src = self.calib_axes[index - 1].source.take();
                if let Some(src) = prev_src {
                    self.axes_used.remove(&src);
                }
                self.capture_resting();
                self.calib_step = CalibStep::Axes(index - 1);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(format!(
                        "✕ {}",
                        crate::i18n::t(&self.config.lang, "btn_cancel")
                    ))
                    .clicked()
                {
                    self.reset_calibration();
                }
            });
        });

        // ── Two-phase release detection (Steam-Input style) ───────────────
        //
        // Phase 1 (release_watch is Some): Axis was detected. Show "suelta el control".
        //          Wait until the axis physically returns to neutral.
        //          Once settled, start_axis_cooldown() (1000ms buffer).
        // Phase 2 (release_watch is None, cooldown running): Axis released, buffer running.
        //          Show countdown. When cooldown ends, advance to next step.
        //
        // This prevents the RIGHT stick from being detected while moving the LEFT stick.

        if let Some((ref watched_axis, next_idx)) = self.calib_release_watch.clone() {
            // Track maximum delta while in phase 2 (release or holding)
            let mut delta = 0;
            if let Ok(cap) = self.raw_capture.lock() {
                let cur = cap.axis_values.get(watched_axis).copied().unwrap_or(0);
                let rest = self.axis_resting.get(watched_axis).copied().unwrap_or(0);
                delta = (cur - rest).abs();
                let curr_max = self.calib_axes[index].max_val.unwrap_or(0);
                if delta > curr_max {
                    self.calib_axes[index].max_val = Some(delta);
                }
            }

            // Settled if axis returns to below 30% of the peak delta observed
            let peak = self.calib_axes[index].max_val.unwrap_or(1);
            let settled_thr = (peak * 30 / 100).max(1);
            let settled = delta < settled_thr;

            if settled && !self.in_cooldown() {
                // Both conditions met: advance!
                self.calib_release_watch = None;
                self.capture_resting();
                self.calib_step = CalibStep::Axes(next_idx);
            } else if settled {
                // Released but buffer running
                ui.add_space(6.0);
                ui.colored_label(
                    egui::Color32::from_rgb(255, 200, 60),
                    format!(
                        "⏳ {} {:.1}s…",
                        crate::i18n::t(&self.config.lang, "calib_next_step"),
                        self.cooldown_remaining_secs()
                    ),
                );
            } else {
                // Still held — require physical release
                ui.add_space(6.0);
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(40, 35, 20))
                    .corner_radius(8.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 220, 60),
                                format!(
                                    "✅ {}",
                                    crate::i18n::t(&self.config.lang, "calib_detected")
                                ),
                            );
                            ui.label(
                                egui::RichText::new(crate::i18n::t(
                                    &self.config.lang,
                                    "calib_release",
                                ))
                                .size(13.0)
                                .weak(),
                            );
                        });
                    });
            }
            return;
        }

        // ── Normal detection (no active release watch) ────────────────────────────
        if !self.in_cooldown() {
            let detected = {
                if let Ok(cap) = self.raw_capture.lock() {
                    detect_axis_movement(&cap.axis_values, &self.axis_resting, &self.axes_used)
                } else {
                    None
                }
            };

            if let Some((raw_axis, is_positive)) = detected {
                let positive_expected = self.calib_axes[index].positive_expected;
                let invert = positive_expected != is_positive;

                if already_mapped {
                    if self.calib_axes[..index]
                        .iter()
                        .any(|s| s.xbox_axis == target && s.source.as_deref() == Some(&raw_axis))
                    {
                        self.status_msg = Some(format!(
                            "✓ {} verificado: {}",
                            self.calib_axes[index].label, raw_axis
                        ));
                    } else {
                        self.status_msg = Some(format!(
                            "⚠ Detectado {} (diferente al paso anterior, intenta otra vez)",
                            raw_axis
                        ));
                        return; // Prevent advancing until they move the SAME axis
                    }
                } else {
                    self.calib_axes[index].source = Some(raw_axis.clone());
                    self.calib_axes[index].invert = invert;
                    let inv_str = if invert { " (auto-invertido)" } else { "" };
                    self.status_msg = Some(format!(
                        "✓ {} detectado: {}{}",
                        self.calib_axes[index].label, raw_axis, inv_str
                    ));
                }

                // Initial max_val track at point of detection
                if let Ok(cap) = self.raw_capture.lock() {
                    let cur = cap.axis_values.get(&raw_axis).copied().unwrap_or(0);
                    let rest = self.axis_resting.get(&raw_axis).copied().unwrap_or(0);
                    self.calib_axes[index].max_val = Some((cur - rest).abs());
                }

                // Start release watch: axis must physically return before advancing
                self.calib_release_watch = Some((raw_axis, index + 1));
                self.start_axis_cooldown(); // 1000ms minimum buffer
            }
        } else {
            ui.add_space(4.0);
            ui.colored_label(
                egui::Color32::from_rgb(255, 200, 60),
                format!(
                    "⏳ {} {:.1}s…",
                    crate::i18n::t(&self.config.lang, "calib_wait"),
                    self.cooldown_remaining_secs()
                ),
            );
        }
    }

    // ── Review step ───────────────────────────

    fn ui_calib_review(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_calib_review"))
                .strong()
                .size(18.0),
        );
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_calib_review_desc"))
                .weak()
                .size(12.0),
        );
        ui.add_space(10.0);

        ui.columns(2, |cols| {
            cols[0].label(
                egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_buttons"))
                    .strong()
                    .size(13.0),
            );
            cols[0].separator();
            for b in &self.calib_btns {
                let src = b
                    .source
                    .as_deref()
                    .unwrap_or(crate::i18n::t(&self.config.lang, "lbl_skipped"));
                let color = if b.source.is_some() {
                    egui::Color32::LIGHT_GREEN
                } else {
                    egui::Color32::from_rgb(80, 80, 90)
                };
                cols[0].horizontal(|ui| {
                    ui.label(egui::RichText::new(&b.label).size(11.0));
                    ui.colored_label(color, src);
                });
            }

            cols[1].label(
                egui::RichText::new(crate::i18n::t(&self.config.lang, "lbl_axes"))
                    .strong()
                    .size(13.0),
            );
            cols[1].separator();
            // Deduplicated view: one row per xbox_axis
            let mut seen: HashSet<&str> = HashSet::new();
            for a in &self.calib_axes {
                if !seen.insert(a.xbox_axis) {
                    continue;
                }
                let src = a
                    .source
                    .as_deref()
                    .unwrap_or(crate::i18n::t(&self.config.lang, "lbl_skipped"));
                let color = if a.source.is_some() {
                    egui::Color32::LIGHT_BLUE
                } else {
                    egui::Color32::from_rgb(80, 80, 90)
                };
                let inv = if a.invert { " ↕inv" } else { "" };
                cols[1].horizontal(|ui| {
                    ui.label(egui::RichText::new(a.xbox_axis).size(11.0));
                    ui.colored_label(color, format!("← {}{}", src, inv));
                });
            }
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(6.0);

        if let Some(ref msg) = self.calib_msg {
            ui.colored_label(egui::Color32::LIGHT_GREEN, msg);
            ui.add_space(4.0);
        }

        ui.horizontal(|ui| {
            if ui
                .button(
                    egui::RichText::new(crate::i18n::t(&self.config.lang, "btn_save_active"))
                        .size(15.0),
                )
                .clicked()
            {
                self.save_profile();
            }
            if ui
                .button(format!(
                    "◀ {}",
                    crate::i18n::t(&self.config.lang, "btn_back")
                ))
                .clicked()
            {
                self.calib_step = CalibStep::Axes(self.calib_axes.len().saturating_sub(1));
            }
            if ui
                .button(format!(
                    "✕ {}",
                    crate::i18n::t(&self.config.lang, "btn_cancel")
                ))
                .clicked()
            {
                self.reset_calibration();
            }
        });
    }
}

// ─────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("XJEmulator")
            .with_inner_size([960.0, 600.0])
            .with_min_inner_size([720.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "XJEmulator",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}
