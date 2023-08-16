use egui::Ui;
use poll_promise::Promise;
use systemd::journal::OpenOptions;

use crate::error::Error;

use super::Services;

pub struct Overview {
    system_bus: zbus::Connection,
    session_bus: zbus::Connection,

    system_services: Services,
    user_services: Services,

    tab: String,
}

impl Overview {
    pub fn connect() -> Result<Overview, Error> {
        let system_bus = Promise::spawn_async(zbus::Connection::system()).block_and_take()?;
        let session_bus = Promise::spawn_async(zbus::Connection::session()).block_and_take()?;

        Ok(Overview {
            system_bus: system_bus.clone(),
            session_bus: session_bus.clone(),
            system_services: Services::new(system_bus, OpenOptions::default().system(true).clone()),
            user_services: Services::new(
                session_bus,
                OpenOptions::default().current_user(true).clone(),
            ),
            tab: "system".to_string(),
        })
    }

    pub fn draw(&mut self, ui: &mut Ui) {
        let system_tab = "system".to_string();
        let user_tab = "user".to_string();

        let sys_services_radio = ui.radio_value(&mut self.tab, system_tab, "System Services");
        let user_serices_radio = ui.radio_value(&mut self.tab, user_tab, "User Services");

        if sys_services_radio.clicked() || user_serices_radio.clicked() {
            self.system_services.close_properties();
            self.user_services.close_properties();
        }

        ui.heading(&self.tab);

        match self.tab.as_str() {
            "system" => self.system_services.draw(ui),
            "user" => self.user_services.draw(ui),
            _ => todo!(),
        }
    }
}
