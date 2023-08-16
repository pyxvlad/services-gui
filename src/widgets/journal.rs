use crate::journal::{JournalReader, Priority};
use egui::{Color32, Context, Ui, Widget};
use egui_extras::Column;

use systemd::journal::OpenOptions;

pub struct JournalWindow {
    target: Option<String>,
    filtered: Vec<usize>,
    open: bool,
    reader: JournalReader,
}

impl JournalWindow {
    pub fn new(options: OpenOptions) -> JournalWindow {
        Self {
            reader: JournalReader::new(options),
            filtered: Vec::default(),
            target: None,
            open: false,
        }
    }

    pub fn open(&mut self, target: Option<String>) {
        self.target = target.clone();
        self.open = true;

        if let Some(target) = target {
            self.filtered = self.reader.filter(&mut |entry| entry.is_for_unit(&target));
        } else {
            self.filtered = self.reader.filter(&mut |_| true);
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        self.reader.receive();

        if let Some(last) = self.filtered.last().copied() {
            if let Some(target) = &self.target {
                self.reader.continue_filter(
                    &mut |entry| entry.is_for_unit(target),
                    &mut self.filtered,
                    last,
                );
            } else {
                self.reader
                    .continue_filter(&mut |_| true, &mut self.filtered, last);
                self.filtered = self.reader.filter(&mut |_| true);
            }
        }
        let mut open = self.open;
        egui::Window::new("Journal")
            .resizable(true)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(format!("Showing journal for {:?}", self.target));
                ui.label(format!("Having {} entries", self.filtered.len()));

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
            .column(Column::auto())
            .column(Column::auto().at_least(128.0))
            .column(Column::auto().at_least(64.0).resizable(true))
            .column(Column::remainder().resizable(true))
            .column(Column::auto().at_least(64.0))
            .striped(true)
            .header(text_height, |mut header| {
                header.col(|ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.label("priority");
                });
                header.col(|ui| {
                    ui.label("timestamp");
                });
                header.col(|ui| {
                    ui.label("service");
                });
                header.col(|ui| {
                    ui.label("message");
                });
                header.col(|ui| {
                    ui.label("source");
                });
            })
            .body(|b| {
                b.rows(text_height * 2.0, self.filtered.len(), |index, mut row| {
                    let entry = &self.reader[self.filtered[index]];
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.add(entry.priority());
                    });

                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.label(entry.timestamp().to_rfc2822());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.label(entry.unit());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.label(entry.message());
                    });
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.label(entry.kind());
                    });
                })
            });
    }
}

type PriorityLabel = Priority;
impl From<PriorityLabel> for Color32 {
    fn from(value: PriorityLabel) -> Self {
        match value {
            Priority::Emergency => Color32::DARK_RED,
            Priority::Alert => Color32::RED,
            Priority::Critical => Color32::LIGHT_RED,
            Priority::Error => Color32::GOLD,
            Priority::Warning => Color32::KHAKI,
            Priority::Notice => Color32::LIGHT_YELLOW,
            Priority::Info => Color32::GRAY,
            Priority::Debug => Color32::DEBUG_COLOR,
        }
    }
}

impl Widget for PriorityLabel {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.colored_label(Color32::from(self), self.to_string())
    }
}
