mod app;
pub(crate) mod backend;
pub(crate) mod types;
mod udev_setup;

pub fn run_app() -> eframe::Result<()> {
    app::run()
}
