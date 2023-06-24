use crate::error::Error;
use egui::{Context, Ui, Widget};
use egui_extras::Column;
use poll_promise::Promise;
use std::time::Duration;
use tokio::sync::mpsc::{error::TryRecvError, Receiver, Sender};

use systemd::journal::OpenOptions;
use tokio::task::JoinHandle;

pub struct JournalWindow {
    options: OpenOptions,
    entries: Vec<Entry>,
    target: Option<String>,
    filtered: Vec<usize>,
    receiver: Receiver<Entry>,
    work: JoinHandle<Result<(), Error>>,
    open: bool,
}

#[derive(Debug)]
pub struct Entry {
    unit: String,
    message: String,
}

impl Widget for Entry {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal_wrapped(|ui| {
            ui.label(self.unit);
            ui.label(self.message);
        })
        .response
    }
}

impl JournalWindow {
    pub fn new(options: OpenOptions) -> JournalWindow {
        let (sender, receiver) = tokio::sync::mpsc::channel(64);

        let moved_options = options.clone();
        let work = tokio::task::spawn_blocking(|| worker(sender, moved_options));
        Self {
            options,
            entries: Vec::default(),
            filtered: Vec::default(),
            target: None,
            receiver,
            work,
            open: false,
        }
    }

    pub fn open(&mut self, target: Option<String>) {
        self.target = target.clone();
        self.open = true;

        if let Some(target) = target {
            self.filtered = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.unit == target || entry.unit == "init.scope")
                .map(|(index, _)| index)
                .collect();
        } else {
            self.filtered = self
                .entries
                .iter()
                .enumerate()
                .map(|(index, _)| index)
                .collect();
        }
    }

    fn receive(&mut self) {
        loop {
            let receive = self.receiver.try_recv();
            match receive {
                Ok(entry) => {
                    self.entries.push(entry);
                }

                Err(err) => match err {
                    TryRecvError::Empty => return,
                    TryRecvError::Disconnected => {
                        panic!("This should have been dropped")
                    }
                },
            }
        }
    }
    pub fn update(&mut self, ctx: &Context) {
        self.receive();
        let mut open = self.open;
        egui::Window::new("Journal")
            .resizable(true)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(format!("Showing journal for {:?}", self.target));
                ui.label(format!("Having {} entries", self.entries.len()));

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.table(ui);
                    });
            });

        self.open = open;
    }

    fn table(&self, ui: &mut Ui) {
        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
        egui_extras::TableBuilder::new(ui)
            .column(Column::auto().at_least(64.0))
            .column(Column::remainder())
            .striped(true)
            .header(text_height, |mut header| {
                header.col(|ui| {
                    ui.label("service");
                });
                header.col(|ui| {
                    ui.label("message");
                });
            })
            .body(|b| {
                b.rows(text_height * 2.0, self.filtered.len(), |index, mut row| {
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.label(self.entries[self.filtered[index]].unit.as_str());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.label(self.entries[self.filtered[index]].message.as_str());
                    });
                })
            });
    }
}

fn worker(sender: Sender<Entry>, options: OpenOptions) -> Result<(), Error> {
    let mut reader = options.open()?;
    while !sender.is_closed() {
        while !sender.is_closed() {
            if reader.next()? == 0 {
                break;
            }

            let entry = reader.next_entry()?;
            if let Some(entry) = entry{
                println!("{{");
                entry.iter().for_each(|(k, v)| {
                    println!("\t\"{k}\": \"{v}\"");
                });
                println!("}}");
            }

            let unit = reader.get_data("_SYSTEMD_UNIT")?.and_then(|v| {
                v.value()
                    .map(String::from_utf8_lossy)
                    .map(|v| v.into_owned())
            });
            let message = reader.get_data("MESSAGE")?.and_then(|v| {
                v.value()
                    .map(String::from_utf8_lossy)
                    .map(|v| v.into_owned())
            });

            if let Some(unit) = unit {
                if let Some(message) = message {
                    let tmp = sender.clone();
                    Promise::spawn_async(async move { tmp.send(Entry { unit, message }).await })
                        .block_and_take()?;
                }
            }
        }
        reader.wait(Some(Duration::from_micros(1)))?;
    }
    Ok(())
}
