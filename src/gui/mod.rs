mod app;
pub(crate) mod backend;
pub(crate) mod types;
pub(crate) mod theme;
pub(crate) mod widgets;
pub(crate) mod views;
pub(crate) mod tray;

pub fn run_app() -> eframe::Result<()> {
    app::run()
}
