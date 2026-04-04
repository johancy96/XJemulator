mod config;
pub mod error;
mod gui;
mod i18n;
mod mapper;
pub mod reader;
mod scanner;
mod virtual_device;
mod xbox_descriptor;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_new("info").unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    if let Err(e) = gui::run_app() {
        eprintln!("Error: {}", e);
    }
}
