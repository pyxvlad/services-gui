use egui::Ui;
use egui_extras::Column;

use crate::systemd::UnitData;

use super::unitdata::active_state_to_color;

pub fn units_table(
    units: &Vec<UnitData>,
    ui: &mut Ui,
    open_index: &mut Option<usize>,
    journal_index: &mut Option<usize>,
) {
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
            let striped = ui.visuals().striped | true;
            egui_extras::TableBuilder::new(ui)
                .resizable(true)
                .striped(striped)
                .column(Column::auto().at_least(128.0))
                .column(Column::auto().at_least(256.0))
                .column(Column::remainder().at_least(64.0).at_most(64.0))
                .column(Column::remainder().at_least(64.0).at_most(64.0))
                .column(Column::remainder()) //.column(Column::auto().at_least(256.0))
                .header(text_height * 2.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Interact");
                        ui.set_max_width(64.0);
                    });
                    header.col(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.add_space(4.0);
                            ui.heading("name");
                        });
                    });

                    header.col(|ui| {
                        ui.vertical_centered_justified(|ui| {
                            ui.heading("active");
                        });
                    });
                    header.col(|ui| {
                        ui.vertical_centered_justified(|ui| {
                            ui.heading("state");
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.add_space(4.0);
                            ui.heading("description");
                        });
                    });
                })
                .body(|b| {
                    b.rows(text_height * 2.0, units.len(), |index, mut row| {
                        row.col(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                if ui.button("Properties").clicked() {
                                    *open_index = Some(index)
                                }
                                if ui.button("Journal").clicked() {
                                    *journal_index = Some(index)
                                }
                            });
                        });

                        row.col(|ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.horizontal(|ui| {
                                ui.add_space(4.0);
                                ui.label(&units[index].name);
                            });
                        });

                        row.col(|ui| {
                            ui.vertical_centered_justified(|ui| {
                                ui.colored_label(
                                    active_state_to_color(units[index].active_status),
                                    units[index].active_status.to_string(),
                                );
                            });
                        });
                        row.col(|ui| {
                            ui.vertical_centered_justified(|ui| {
                                ui.label(&units[index].load_status.to_string());
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(4.0);
                                ui.label(&units[index].description);
                            });
                        });
                    })
                });
        });
}
