use crate::systemd;
use crate::systemd::UnitData;
use crate::widgets::units_table::units_table;
use crate::widgets::PropertiesWindow;
use ::systemd::journal::OpenOptions;
use egui::Ui;
use poll_promise::Promise;

use super::journal::JournalWindow;

pub struct Services {
    units_promise: Promise<zbus::Result<Vec<UnitData>>>,
    properties: PropertiesWindow,
    journal: JournalWindow,
    con: zbus::Connection,
}

impl Services {
    pub fn new(con: zbus::Connection, options: OpenOptions) -> Self {
        Services {
            units_promise: Promise::spawn_async(systemd::list_units(con.clone())),
            properties: PropertiesWindow::with_connection(con.clone()),
            journal: JournalWindow::new(options),
            con,
        }
    }

    pub fn close_properties(&mut self) {
        self.properties.close();
    }

    fn refresh(&mut self) {
        self.units_promise = Promise::spawn_async(systemd::list_units(self.con.clone()));
    }

    pub fn draw(&mut self, ui: &mut Ui) {
        let mut unit_index = None;
        let mut journal_index = None;
        if ui.button("Refresh Units").clicked() {
            self.refresh();
        }
        if ui.button("View Journal for All").clicked() {
            self.journal.open(None)
        }
        if let Some(response) = self.units_promise.ready() {
            match response {
                Ok(units) => {
                    ui.heading(format!("services: {}", units.len()));
                    units_table(units, ui, &mut unit_index, &mut journal_index);

                    if let Some(index) = unit_index {
                        self.properties.open(index);
                    }

                    if let Some(index) = journal_index {
                        self.journal.open(Some(units[index].name.clone()))
                    }

                    self.properties.draw(ui.ctx(), &|i| units.get(i));
                }
                Err(err) => {
                    ui.heading(err.to_string());
                }
            }
        } else {
            ui.spinner();
        }
        self.journal.update(ui.ctx());
    }
}
