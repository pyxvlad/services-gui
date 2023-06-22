use egui::{Color32, Context, Window};

use crate::systemd;

#[derive(Default)]
pub struct PropertiesWindow {
    unit: Option<usize>,
    open: bool,
}

impl PropertiesWindow {
    pub fn draw<'a_, F>(&mut self, ctx: &Context, extractor: &F)
    where
        F: Fn(usize) -> Option<&'a_ systemd::UnitData>,
    {
        match self.unit {
            Some(index) => {
                let unit_opt = extractor(index);

                if let Some(unit) = unit_opt {
                    Window::new("Service Properties")
                        .resizable(true)
                        .pivot(egui::Align2::CENTER_CENTER)
                        .open(&mut self.open)
                        .show(ctx, |ui| {
                            ui.label(format!("Name: {}", unit.name));
                            ui.label(unit.description.as_str());
                            ui.label(unit.load_status.to_string());

                            ui.horizontal_wrapped(|ui| {
                                ui.label("Status:");
                                ui.colored_label(
                                    active_state_to_color(unit.active_status),
                                    unit.active_status.to_string(),
                                );
                            });

                            ui.label(unit.object_path.to_string());

                            ui.strong("populate this!");
                        });
                }
            }
            None => return,
        };
        if !self.open {
            self.unit = None
        }
    }

    pub fn open(&mut self, unit: usize) {
        self.unit = Some(unit);
        self.open = true;
    }
    pub fn close(&mut self) {
        self.unit = None;
        self.open = false;
    }
}

fn active_state_to_color(state: systemd::ActiveState) -> Color32 {
    match state {
        systemd::ActiveState::Active => Color32::GREEN,
        systemd::ActiveState::Failed => Color32::RED,
        systemd::ActiveState::Inactive => Color32::GRAY,
        systemd::ActiveState::Reloading => Color32::YELLOW,
        systemd::ActiveState::Activating => Color32::YELLOW,
        systemd::ActiveState::Deactivating => Color32::YELLOW,
    }
}
