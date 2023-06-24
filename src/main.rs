#![warn(clippy::all, rust_2018_idioms)]
#![feature(iter_collect_into)]

use widgets::system_overview::Overview;

mod app;
pub mod error;
pub mod message;
mod systemd;
mod widgets;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let overview = Overview::connect()?;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(app::TemplateApp::new(cc, overview))),
    )?;

    Ok(())
}
