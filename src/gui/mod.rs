mod app;
pub(crate) mod backend;
pub(crate) mod types;

pub fn run_app() -> eframe::Result<()> {
    app::run()
}
