use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use eframe::egui;
use tracing::error;

use crate::gui::backend::*;
use crate::gui::theme::Theme;
use crate::gui::types::*;
use crate::gui::views;
use crate::mapper::Mapper;
use crate::scanner::{scan_gamepads, GamepadInfo};
use crate::virtual_device::VirtualXbox360;

pub struct App {
    pub current_view: AppView,
    pub gamepads: Vec<GamepadInfo>,
    pub selected: Option<usize>,

    pub raw_capture: Arc<Mutex<RawCapture>>,
    pub reader_running: Arc<AtomicBool>,
    pub reader_handle: Option<std::thread::JoinHandle<()>>,
    pub egui_ctx: Option<egui::Context>,

    pub emulators: HashMap<String, Arc<AtomicBool>>,

    pub mapper: Mapper,
    pub active_profile: Option<String>,

    pub calib_step: CalibStep,
    pub calib_btns: Vec<BtnSlot>,
    pub calib_axes: Vec<AxisSlot>,
    pub profile_name: String,

    pub axis_resting: HashMap<evdevil::event::Abs, i32>,
    pub axes_used: HashSet<evdevil::event::Abs>,
    pub cooldown_until: Option<std::time::Instant>,

    pub editing_profile: Option<(String, String)>,
    pub saved_profiles: Vec<String>,
    pub status_msg: Option<String>,

    pub config: crate::config::AppConfig,
    pub uinput_ok: bool,
    pub profiles_cache: Vec<crate::mapper::MappingProfile>,
    pub quitting: bool,
    pub last_step_time: std::time::Instant,
    pub calib_logs: Vec<String>,
    pub axis_min: HashMap<evdevil::event::Abs, i32>,
    pub axis_max: HashMap<evdevil::event::Abs, i32>,
    pub current_detect: Option<(evdevil::event::Abs, bool)>,
    pub detection_since: Option<std::time::Instant>,
    pub waiting_for_release: bool,
    pub stability_timer: Option<std::time::Instant>,
}

impl App {
    pub fn new(egui_ctx: Option<egui::Context>) -> Self {
        let gamepads = scan_gamepads();
        let saved_profiles = scan_profiles();
        let (mapper, active_profile) = load_best_profile(&saved_profiles);
        let config = crate::config::AppConfig::load();

        let mut app = Self {
            current_view: AppView::Dashboard,
            gamepads,
            selected: None,
            raw_capture: Arc::new(Mutex::new(RawCapture::default())),
            reader_running: Arc::new(AtomicBool::new(false)),
            reader_handle: None,
            egui_ctx,
            emulators: HashMap::new(),
            mapper,
            active_profile,
            calib_step: CalibStep::Idle,
            calib_btns: default_btn_slots(&config.lang),
            calib_axes: default_axis_slots(&config.lang),
            profile_name: "nuevo_perfil".into(),
            axis_resting: HashMap::new(),
            axes_used: HashSet::new(),
            cooldown_until: None,
            editing_profile: None,
            saved_profiles,
            status_msg: None,
            config: config.clone(),
            uinput_ok: false,
            profiles_cache: Vec::new(),
            quitting: false,
            last_step_time: std::time::Instant::now(),
            calib_logs: Vec::new(),
            axis_min: HashMap::new(),
            axis_max: HashMap::new(),
            current_detect: None,
            detection_since: None,
            waiting_for_release: false,
            stability_timer: None,
        };

        app.check_uinput_permission();
        app.refresh_profiles_cache();
        app
    }

    pub fn check_uinput_permission(&mut self) {
        self.uinput_ok = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/uinput")
            .is_ok();
    }

    pub fn refresh_profiles_cache(&mut self) {
        self.profiles_cache.clear();
        for name in &self.saved_profiles {
            let full_path = crate::paths::AppPaths::profile_path(name);
            if let Ok(content) = std::fs::read_to_string(full_path) {
                if let Ok(prof) = toml::from_str::<crate::mapper::MappingProfile>(&content) {
                    self.profiles_cache.push(prof);
                }
            }
        }
    }

    // --- Lógica de Dispositivos ---

    pub fn select_gamepad(&mut self, idx: usize) {
        if self.selected == Some(idx) {
            return;
        }
        self.selected = Some(idx);

        if let Some(gp) = self.gamepads.get(idx).cloned() {
            self.try_auto_load_profile(gp.vendor_id, gp.product_id);
        }
        self.reset_calibration();
    }

    fn try_auto_load_profile(&mut self, vid: u16, pid: u16) {
        for prof in &self.profiles_cache {
            if prof.vendor_id == Some(vid) && prof.product_id == Some(pid) {
                if let Ok(m) = Mapper::from_profile(prof) {
                    let safe_name = crate::paths::AppPaths::sanitize_profile_name(&prof.name);
                    self.mapper = m;
                    self.active_profile = Some(format!("{}.toml", safe_name));
                    self.status_msg = Some(format!("✨ Perfil auto-detectado: {}", prof.name));
                    return;
                }
            }
        }
    }

    pub fn start_reader(&mut self, path: String) {
        self.stop_reader();
        if let Ok(mut cap) = self.raw_capture.lock() {
            *cap = RawCapture::default();
        }
        let capture = self.raw_capture.clone();
        let running = Arc::new(AtomicBool::new(true));
        self.reader_running = running.clone();
        let ctx = self.egui_ctx.clone();
        self.reader_handle = Some(std::thread::spawn(move || {
            raw_reader_loop(path, capture, running, ctx)
        }));
    }

    pub fn stop_reader(&mut self) {
        self.reader_running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
    }

    // --- Lógica de Emulación ---

    pub fn start_emulator(&mut self, path: String) {
        if self.emulators.contains_key(&path) {
            return;
        }
        if self.emulators.len() >= 4 {
            self.status_msg = Some("Máximo 4 controladores".into());
            return;
        }

        // Liberar de forma síncrona el lector antes de abrir el emulador (evita conflicto EVIOCGRAB)
        self.stop_reader();

        match VirtualXbox360::new() {
            Ok(vx) => {
                let running = Arc::new(AtomicBool::new(true));
                self.emulators.insert(path.clone(), running.clone());

                let mapper = self.mapper.clone();
                let capture = self.raw_capture.clone();

                let thread_path = path.clone();
                std::thread::spawn(move || {
                    emulation_loop(thread_path, capture, running, mapper, vx)
                });
                self.status_msg = Some(format!("✓ Emulando: {}", path));
            }
            Err(e) => {
                error!("Error emulador: {}", e);
                self.status_msg = Some(format!("Error: {}", e));
            }
        }
    }

    pub fn stop_emulator(&mut self, path: &str) {
        if let Some(running) = self.emulators.remove(path) {
            running.store(false, Ordering::SeqCst);
        }
    }

    pub fn stop_all_activity(&mut self) {
        self.stop_reader();
        for (_, running) in self.emulators.drain() {
            running.store(false, Ordering::SeqCst);
        }
    }

    // --- Lógica de Perfiles ---

    pub fn load_profile_from_path(&mut self, name: &str) {
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            error!("Intento de acceso a ruta no permitida: {}", name);
            return;
        }

        let path = crate::paths::AppPaths::profile_path(name);
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(prof) = toml::from_str::<crate::mapper::MappingProfile>(&content) {
                if let Ok(m) = Mapper::from_profile(&prof) {
                    let filename = path
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_else(|| name.to_string());
                    self.mapper = m;
                    self.active_profile = Some(filename.clone());
                    self.status_msg = Some(format!("Perfil cargado: {}", filename));
                }
            }
        }
    }

    pub fn delete_profile(&mut self, name: &str) {
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            return;
        }
        let path = crate::paths::AppPaths::profile_path(name);
        let _ = std::fs::remove_file(path);
        self.saved_profiles = scan_profiles();
        self.refresh_profiles_cache();
        if self.active_profile.as_deref() == Some(name) {
            self.active_profile = None;
            self.mapper = Mapper::identity();
        }
    }

    // --- Calibración ---

    pub fn reset_calibration(&mut self) {
        self.calib_step = CalibStep::Idle;
        self.calib_btns = default_btn_slots(&self.config.lang);
        self.calib_axes = default_axis_slots(&self.config.lang);
        self.axes_used.clear();
        self.axis_resting.clear();
        self.axis_min.clear();
        self.axis_max.clear();
        self.calib_logs.clear();
        self.waiting_for_release = false;
        self.stability_timer = None;
        self.current_detect = None;
        self.detection_since = None;
        self.stop_reader();

        if let Ok(mut cap) = self.raw_capture.lock() {
            cap.key_queue.clear();
            cap.axis_values.clear();
            cap.pressed_keys.clear();
        }
    }

    /// Gestor de transiciones determinista
    fn advance_to_step(&mut self, next: CalibStep) {
        self.calib_step = next;
        self.last_step_time = std::time::Instant::now();
        self.waiting_for_release = true; // Forzar centro por seguridad en cada cambio
        self.stability_timer = None;
        self.current_detect = None;
        self.detection_since = None;

        if let Ok(mut cap) = self.raw_capture.lock() {
            cap.key_queue.clear();
        }
        self.calib_logs.clear();
    }

    pub fn start_assisted_mapping(&mut self) {
        self.reset_calibration();
        if let Some(idx) = self.selected {
            if let Some(gp) = self.gamepads.get(idx).cloned() {
                self.profile_name = crate::paths::AppPaths::sanitize_profile_name(&gp.name);
                self.start_reader(gp.path);
            }
        }
        self.advance_to_step(CalibStep::Buttons(0));
    }

    pub fn handle_calibration_input(&mut self) {
        if self.in_cooldown() {
            return;
        }

        match self.calib_step {
            CalibStep::Buttons(idx) => {
                let key = if let Ok(mut cap) = self.raw_capture.lock() {
                    cap.key_queue.pop_front()
                } else {
                    None
                };

                if let Some(key) = key {
                    let key_name = format!("{:?}", key);
                    self.calib_btns[idx].source = Some(key_name);
                    self.start_btn_cooldown();

                    let next_idx = idx + 1;
                    if next_idx >= self.calib_btns.len() {
                        self.advance_to_step(CalibStep::Axes(0));
                    } else {
                        self.advance_to_step(CalibStep::Buttons(next_idx));
                    }
                }
            }
            CalibStep::Axes(idx) => {
                let ranges = self
                    .selected
                    .and_then(|i| self.gamepads.get(i))
                    .map(|g| &g.axis_ranges);
                let empty_ranges = HashMap::new();
                let ranges_ref = ranges.unwrap_or(&empty_ranges);
                let current_target = self.calib_axes[idx].xbox_axis;

                let detected = {
                    let lock = self.raw_capture.lock();
                    let Ok(cap) = lock else { return };

                    if self.waiting_for_release {
                        if crate::gui::backend::is_all_centered(&cap.axis_values, ranges_ref) {
                            let now = std::time::Instant::now();
                            let start = self.stability_timer.get_or_insert(now);
                            if start.elapsed() >= std::time::Duration::from_millis(250) {
                                Some((None, cap.axis_values.clone()))
                            } else {
                                None
                            }
                        } else {
                            self.stability_timer = None;
                            None
                        }
                    } else {
                        let mut exclude = HashSet::new();
                        for prev in &self.calib_axes {
                            if let (Some(src), tgt) = (&prev.source, prev.xbox_axis) {
                                if tgt != current_target {
                                    if let Ok(abs_code) = crate::mapper::parse_abs(src) {
                                        exclude.insert(abs_code);
                                    }
                                }
                            }
                        }

                        let det = crate::gui::backend::detect_axis_movement(
                            &cap.axis_values,
                            &self.axis_resting,
                            &exclude,
                            ranges_ref,
                        );

                        det.map(|d| (Some(d), cap.axis_values.clone()))
                    }
                };

                if let Some((det, values)) = detected {
                    if let Some(d) = det {
                        let (axis, pos) = d;
                        let is_hat = format!("{:?}", axis).contains("HAT");
                        let hold_req = if is_hat { 40 } else { 250 };

                        if self.current_detect == Some(d) {
                            if let Some(since) = self.detection_since {
                                if !is_hat {
                                    let rest = self.axis_resting.get(&axis).copied().unwrap_or(0);
                                    let cur_val = values.get(&axis).copied().unwrap_or(rest);
                                    let delta = (cur_val - rest).abs();
                                    let old_max = self.calib_axes[idx].max_val.unwrap_or(0);
                                    if delta > old_max {
                                        self.calib_axes[idx].max_val = Some(delta);
                                    }
                                }

                                if since.elapsed().as_millis() >= hold_req as u128 {
                                    let axis_name = format!("{:?}", axis);
                                    self.calib_axes[idx].source = Some(axis_name);
                                    self.calib_axes[idx].invert =
                                        self.calib_axes[idx].positive_expected != pos;
                                    self.axes_used.insert(axis);
                                    self.advance_to_step(if idx + 1 >= self.calib_axes.len() {
                                        CalibStep::Review
                                    } else {
                                        CalibStep::Axes(idx + 1)
                                    });
                                }
                            } else {
                                self.detection_since = Some(std::time::Instant::now());
                            }
                        } else {
                            self.current_detect = Some(d);
                            self.detection_since = Some(std::time::Instant::now());
                        }
                    } else {
                        for (k, v) in values {
                            self.axis_resting.insert(k, v);
                        }
                        self.waiting_for_release = false;
                        self.stability_timer = None;
                        self.calib_logs.push(
                            crate::i18n::t(&self.config.lang, "lbl_axis_synced").to_string(),
                        );
                    }
                } else {
                    self.current_detect = None;
                    self.detection_since = None;
                }
            }
            _ => {}
        }
    }

    fn in_cooldown(&self) -> bool {
        self.cooldown_until
            .map_or(false, |t| std::time::Instant::now() < t)
    }

    fn start_btn_cooldown(&mut self) {
        self.cooldown_until =
            Some(std::time::Instant::now() + std::time::Duration::from_millis(250));
    }

    pub fn skip_calibration_step(&mut self) {
        match self.calib_step {
            CalibStep::Buttons(idx) => {
                self.calib_btns[idx].source = None;
                let next_idx = idx + 1;
                if next_idx >= self.calib_btns.len() {
                    self.capture_resting();
                    self.advance_to_step(CalibStep::Axes(0));
                } else {
                    self.advance_to_step(CalibStep::Buttons(next_idx));
                }
            }
            CalibStep::Axes(idx) => {
                self.calib_axes[idx].source = None;
                self.calib_axes[idx].max_val = None;
                let next_idx = idx + 1;
                if next_idx >= self.calib_axes.len() {
                    self.advance_to_step(CalibStep::Review);
                } else {
                    self.advance_to_step(CalibStep::Axes(next_idx));
                }
            }
            _ => {}
        }
        self.start_btn_cooldown();
    }

    fn capture_resting(&mut self) {
        if let Ok(cap) = self.raw_capture.lock() {
            self.axis_resting = cap.axis_values.clone();
        }
    }

    pub fn prev_calibration_step(&mut self) {
        self.current_detect = None;
        self.detection_since = None;
        self.waiting_for_release = false;

        match self.calib_step {
            CalibStep::Buttons(idx) if idx > 0 => {
                self.calib_step = CalibStep::Buttons(idx - 1);
                self.calib_btns[idx - 1].source = None;
            }
            CalibStep::Axes(idx) => {
                if idx == 0 {
                    let last_btn = self.calib_btns.len() - 1;
                    self.calib_step = CalibStep::Buttons(last_btn);
                    if let Some(b) = self.calib_btns.last_mut() {
                        b.source = None;
                    }
                } else {
                    self.calib_step = CalibStep::Axes(idx - 1);
                    self.calib_axes[idx - 1].source = None;
                    self.calib_axes[idx - 1].max_val = None;

                    self.axes_used.clear();
                    for i in 0..(idx - 1) {
                        if let Some(ref src) = self.calib_axes[i].source {
                            if let Ok(abs_code) = crate::mapper::parse_abs(src) {
                                self.axes_used.insert(abs_code);
                            }
                        }
                    }
                }
            }
            CalibStep::Review => {
                let last_axis = self.calib_axes.len() - 1;
                self.calib_step = CalibStep::Axes(last_axis);
                if let Some(a) = self.calib_axes.last_mut() {
                    a.source = None;
                }
            }
            _ => {}
        }
        self.last_step_time = std::time::Instant::now();
        self.calib_logs.clear();
        self.start_btn_cooldown();
    }

    pub fn save_profile(&mut self) {
        let mut vid = None;
        let mut pid = None;
        if let Some(idx) = self.selected {
            if let Some(gp) = self.gamepads.get(idx) {
                vid = Some(gp.vendor_id);
                pid = Some(gp.product_id);
            }
        }

        let safe_name = crate::paths::AppPaths::sanitize_profile_name(&self.profile_name);
        self.profile_name = safe_name.clone();

        let toml = generate_profile_toml_with_ids(
            &self.calib_btns,
            &self.calib_axes,
            &safe_name,
            &self.axis_resting,
            vid,
            pid,
        );
        let path = crate::paths::AppPaths::profile_path(&safe_name);
        if std::fs::write(&path, toml).is_ok() {
            let filename = format!("{}.toml", safe_name);
            self.saved_profiles = scan_profiles();
            self.refresh_profiles_cache();
            self.load_profile_from_path(&filename);
            self.reset_calibration();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.egui_ctx.is_none() {
            self.egui_ctx = Some(ctx.clone());
        }

        if ctx.input(|i| i.time % 2.0 < 0.02) {
            self.check_uinput_permission();
        }

        self.emulators
            .retain(|_, running| running.load(Ordering::SeqCst));

        Theme::apply(ctx);

        let is_focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if is_focused && (self.calib_step != CalibStep::Idle || !self.emulators.is_empty()) {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::NONE.fill(Theme::BG_DEEP).inner_margin(10.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} XJEMULATOR",
                            crate::gui::fonts::icons::GAMEPAD
                        ))
                        .strong()
                        .size(20.0)
                        .color(Theme::ACCENT),
                    );
                    ui.add_space(20.0);

                    ui.selectable_value(
                        &mut self.current_view,
                        AppView::Dashboard,
                        crate::i18n::t(&self.config.lang, "nav_dashboard"),
                    );
                    ui.selectable_value(
                        &mut self.current_view,
                        AppView::Profiles,
                        crate::i18n::t(&self.config.lang, "nav_profiles"),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(crate::gui::fonts::icons::X).clicked() {
                            self.quitting = true;
                            self.stop_all_activity();
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        ui.add_space(8.0);

                        if ui.button(crate::gui::fonts::icons::GLOBE).clicked() {
                            self.config.lang = match self.config.lang {
                                crate::i18n::Lang::Es => crate::i18n::Lang::En,
                                crate::i18n::Lang::En => crate::i18n::Lang::Es,
                            };
                            self.config.save();
                            if self.calib_step == CalibStep::Idle {
                                self.calib_btns = default_btn_slots(&self.config.lang);
                                self.calib_axes = default_axis_slots(&self.config.lang);
                            }
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.calib_step != CalibStep::Idle {
                views::calibration::render(self, ui);
            } else {
                match self.current_view {
                    AppView::Dashboard => views::dashboard::render(self, ui),
                    AppView::Profiles => views::profiles::render(self, ui),
                }
            }
        });

        egui::TopBottomPanel::bottom("footer")
            .frame(egui::Frame::NONE.fill(Theme::BG_DEEP).inner_margin(5.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(msg) = &self.status_msg {
                        ui.label(egui::RichText::new(msg).size(12.0).weak());
                    }
                });
            });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_all_activity();
    }
}

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(format!("XJEmulator v{}", env!("CARGO_PKG_VERSION")))
            .with_app_id("xjemulator")
            .with_inner_size([1000.0, 650.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "XJEmulator",
        options,
        Box::new(|cc| {
            crate::gui::fonts::setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(App::new(Some(cc.egui_ctx.clone()))))
        }),
    )
}
