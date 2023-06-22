use egui::Ui;
use egui_extras::Column;

use crate::systemd::UnitData;

pub fn units_table(units: &Vec<UnitData>, ui: &mut Ui, open_index: &mut Option<usize>) {
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
            egui_extras::TableBuilder::new(ui)
                .resizable(true)
                .striped(true)
                .columns(Column::auto().at_least(64.0).resizable(true), 5)
                .header(text_height, |mut header| {
                    header.col(|ui| {
                        ui.heading("Interact");
                        ui.set_max_width(64.0);
                    });
                    header.col(|ui| {
                        ui.heading("name");
                    });
                    header.col(|ui| {
                        ui.heading("description");
                    });
                    header.col(|ui| {
                        ui.heading("active");
                    });
                    header.col(|ui| {
                        ui.heading("state");
                    });
                })
                .body(|b| {
                    b.rows(text_height * 2.0, units.len(), |index, mut row| {
                        row.col(|ui| {
                            if ui.button("Properties").clicked() {
                                *open_index = Some(index)
                            }
                        });

                        row.col(|ui| {
                            ui.label(&units[index].name);
                        });
                        row.col(|ui| {
                            ui.label(&units[index].description);
                        });
                        row.col(|ui| {
                            ui.label(&units[index].active_status.to_string());
                        });
                        row.col(|ui| {
                            ui.label(&units[index].load_status.to_string());
                        });
                    })
                });
        });
}
