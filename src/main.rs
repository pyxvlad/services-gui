#![warn(clippy::all, rust_2018_idioms)]
mod app;
pub mod message;
mod systemd;
mod widgets;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;

    let handle = rt.spawn(async { systemd::list_system_units().await });

    let mut units: Vec<systemd::UnitData> = rt.block_on(handle)??;

    units.sort_by_key(|ud| ud.name.clone());
    let handle = rt.spawn(async { systemd::list_user_units().await });

    let mut user_units: Vec<systemd::UnitData> = rt.block_on(handle)??;

    user_units.sort_by_key(|ud| ud.name.clone());

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| {
            println!("===> units = {}", units.len());
            Box::new(app::TemplateApp::new(cc, units, user_units))
        }),
    )?;

    Ok(())
}
