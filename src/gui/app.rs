use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use eframe::egui;
use tracing::error;

use crate::mapper::Mapper;
use crate::scanner::{scan_gamepads, GamepadInfo};
use crate::virtual_device::VirtualXbox360;
use crate::gui::backend::*;
use crate::gui::types::*;
use crate::gui::theme::Theme;
use crate::gui::views;

pub struct App {
    pub current_view: AppView,
    pub gamepads: Vec<GamepadInfo>,
    pub selected: Option<usize>,

    pub raw_capture: Arc<Mutex<RawCapture>>,
    pub reader_running: Arc<AtomicBool>,

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
    pub tray: Option<crate::gui::tray::TrayManager>,
    pub tray_rx: crossbeam_channel::Receiver<crate::gui::tray::TrayMsg>,
    pub quitting: bool,
}

impl App {
    pub fn new() -> Self {
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
            uinput_ok: std::fs::OpenOptions::new().write(true).open("/dev/uinput").is_ok(),
            profiles_cache: Vec::new(),
            tray: None,
            tray_rx: crossbeam_channel::unbounded().1,
            quitting: false,
        };
        
        let (tx, rx) = crossbeam_channel::unbounded();
        app.tray = Some(crate::gui::tray::TrayManager::new(&config.lang, tx));
        app.tray_rx = rx;
        
        app.refresh_profiles_cache();
        app
    }

    pub fn refresh_profiles_cache(&mut self) {
        self.profiles_cache.clear();
        for path in &self.saved_profiles {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(prof) = toml::from_str::<crate::mapper::MappingProfile>(&content) {
                    self.profiles_cache.push(prof);
                }
            }
        }
    }

    // --- Lógica de Dispositivos ---

    pub fn select_gamepad(&mut self, idx: usize) {
        if self.selected == Some(idx) { return; }
        self.selected = Some(idx);
        
        if let Some(gp) = self.gamepads.get(idx).cloned() {
            // Auto-detección: Buscar perfil que coincida con VID:PID
            self.try_auto_load_profile(gp.vendor_id, gp.product_id);

            if !self.emulators.contains_key(&gp.path) {
                self.start_reader(gp.path);
            } else {
                self.stop_reader();
            }
        }
    }

    fn try_auto_load_profile(&mut self, vid: u16, pid: u16) {
        // Uso de cache: 0 accesos al disco en el hot-path
        for prof in &self.profiles_cache {
            if prof.vendor_id == Some(vid) && prof.product_id == Some(pid) {
                if let Ok(m) = Mapper::from_profile(prof) {
                    self.mapper = m;
                    self.active_profile = Some(format!("{}.toml", prof.name));
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
        let running = self.reader_running.clone();
        running.store(true, Ordering::SeqCst);
        std::thread::spawn(move || raw_reader_loop(path, capture, running));
    }

    pub fn stop_reader(&mut self) {
        self.reader_running.store(false, Ordering::SeqCst);
    }

    // --- Lógica de Emulación ---

    pub fn start_emulator(&mut self, path: String) {
        if self.emulators.contains_key(&path) { return; }
        if self.emulators.len() >= 4 {
            self.status_msg = Some("Máximo 4 controladores".into());
            return;
        }

        self.stop_reader(); // Liberar el dispositivo para el emulador

        match VirtualXbox360::new() {
            Ok(vx) => {
                let running = Arc::new(AtomicBool::new(true));
                self.emulators.insert(path.clone(), running.clone());

                let mapper = self.mapper.clone();
                let dummy_cap = Arc::new(Mutex::new(RawCapture::default()));

                let thread_path = path.clone();
                std::thread::spawn(move || {
                    emulation_loop(thread_path, dummy_cap, running, mapper, vx)
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
            if let Some(idx) = self.selected {
                if self.gamepads[idx].path == path {
                    self.start_reader(path.to_string());
                }
            }
        }
    }

    // --- Lógica de Perfiles ---

    pub fn load_profile_from_path(&mut self, name: &str) {
        // Sanitización: Evitar que el path contenga ".." para prevenir Path Traversal
        if name.contains("..") || name.contains('/') {
            error!("Intento de acceso a ruta no permitida: {}", name);
            return;
        }

        let path = crate::paths::AppPaths::profile_path(name);
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(prof) = toml::from_str::<crate::mapper::MappingProfile>(&content) {
                if let Ok(m) = Mapper::from_profile(&prof) {
                    self.mapper = m;
                    self.active_profile = Some(name.to_string());
                    self.status_msg = Some(format!("Perfil cargado: {}", name));
                }
            }
        }
    }

    pub fn delete_profile(&mut self, name: &str) {
        if name.contains("..") || name.contains('/') { return; }
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
        if let Ok(mut cap) = self.raw_capture.lock() {
            cap.key_queue.clear();
        }
    }

    pub fn handle_calibration_input(&mut self) {
        if self.in_cooldown() { return; }

        match self.calib_step {
            CalibStep::Buttons(idx) => {
                let detected = if let Ok(mut cap) = self.raw_capture.lock() {
                    cap.key_queue.pop_front()
                } else { None };

                if let Some(key) = detected {
                    // Convertir a nombre solo para el perfil TOML final
                    let key_name = format!("{:?}", key);
                    self.calib_btns[idx].source = Some(key_name);
                    self.start_btn_cooldown();
                    self.calib_step = CalibStep::Buttons(idx + 1);
                    if idx + 1 >= self.calib_btns.len() {
                        self.capture_resting();
                        self.calib_step = CalibStep::Axes(0);
                    }
                }
            }
            CalibStep::Axes(idx) => {
                let detected = if let Ok(cap) = self.raw_capture.lock() {
                    detect_axis_movement(&cap.axis_values, &self.axis_resting, &self.axes_used)
                } else { None };

                if let Some((axis, pos)) = detected {
                    let axis_name = format!("{:?}", axis);
                    self.calib_axes[idx].source = Some(axis_name);
                    self.calib_axes[idx].invert = self.calib_axes[idx].positive_expected != pos;
                    self.axes_used.insert(axis);
                    self.start_axis_cooldown();
                    self.calib_step = CalibStep::Axes(idx + 1);
                    if idx + 1 >= self.calib_axes.len() {
                        self.calib_step = CalibStep::Review;
                    }
                }
            }
            _ => {}
        }
    }

    fn in_cooldown(&self) -> bool {
        self.cooldown_until.map_or(false, |t| std::time::Instant::now() < t)
    }

    fn start_btn_cooldown(&mut self) {
        self.cooldown_until = Some(std::time::Instant::now() + std::time::Duration::from_millis(500));
    }

    fn start_axis_cooldown(&mut self) {
        self.cooldown_until = Some(std::time::Instant::now() + std::time::Duration::from_millis(800));
    }

    fn capture_resting(&mut self) {
        if let Ok(cap) = self.raw_capture.lock() {
            self.axis_resting = cap.axis_values.clone();
        }
    }

    pub fn save_profile(&mut self) {
        let mut vid = None;
        let mut pid = None;
        if let Some(idx) = self.selected {
            vid = Some(self.gamepads[idx].vendor_id);
            pid = Some(self.gamepads[idx].product_id);
        }

        let toml = generate_profile_toml_with_ids(
            &self.calib_btns, 
            &self.calib_axes, 
            &self.profile_name, 
            &self.axis_resting,
            vid,
            pid
        );
        let path = crate::paths::AppPaths::profile_path(&self.profile_name);
        if std::fs::write(&path, toml).is_ok() {
            let filename = format!("{}.toml", self.profile_name);
            self.saved_profiles = scan_profiles();
            self.refresh_profiles_cache();
            self.load_profile_from_path(&filename);
            self.reset_calibration();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Procesar mensajes del System Tray
        while let Ok(msg) = self.tray_rx.try_recv() {
            match msg {
                crate::gui::tray::TrayMsg::ShowWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                crate::gui::tray::TrayMsg::Quit => {
                    self.quitting = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // Lógica de "Ocultar al Tray" en lugar de cerrar
        if ctx.input(|i| i.viewport().close_requested()) && !self.quitting {
            if self.tray.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }

        Theme::apply(ctx);

        // GPU Smart Sleep: Solo repintar si la ventana está activa y hay actividad
        let is_active = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if is_active && (self.calib_step != CalibStep::Idle || !self.emulators.is_empty()) {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        egui::TopBottomPanel::top("header").frame(egui::Frame::NONE.fill(Theme::BG_DEEP).inner_margin(10.0)).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🎮 XJEMULATOR").strong().size(20.0).color(Theme::ACCENT));
                ui.add_space(20.0);
                
                ui.selectable_value(&mut self.current_view, AppView::Dashboard, crate::i18n::t(&self.config.lang, "nav_dashboard"));
                ui.selectable_value(&mut self.current_view, AppView::Profiles, crate::i18n::t(&self.config.lang, "nav_profiles"));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🌐").clicked() {
                        self.config.lang = match self.config.lang {
                            crate::i18n::Lang::Es => crate::i18n::Lang::En,
                            crate::i18n::Lang::En => crate::i18n::Lang::Es,
                        };
                        // Sincronizar idioma con el tray
                        if let Some(t) = &self.tray {
                            t.update_lang(self.config.lang);
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

        egui::TopBottomPanel::bottom("footer").frame(egui::Frame::NONE.fill(Theme::BG_DEEP).inner_margin(5.0)).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(msg) = &self.status_msg {
                    ui.label(egui::RichText::new(msg).size(12.0).weak());
                }
            });
        });
    }
}

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("XJEmulator")
            .with_app_id("xjemulator")
            .with_inner_size([1000.0, 650.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native("XJEmulator", options, Box::new(|_cc| Ok(Box::new(App::new()))))
}
