use crate::systemd::{self, ActiveState, LoadState, UnitFilePreset, UnitFileState};
use egui::{Color32, Widget};

pub fn active_state_to_color(state: ActiveState) -> Color32 {
    match state {
        systemd::ActiveState::Active => Color32::GREEN,
        systemd::ActiveState::Failed => Color32::RED,
        systemd::ActiveState::Inactive => Color32::GRAY,
        systemd::ActiveState::Reloading => Color32::YELLOW,
        systemd::ActiveState::Activating => Color32::YELLOW,
        systemd::ActiveState::Deactivating => Color32::YELLOW,
    }
}

pub fn load_state_to_color(state: LoadState) -> Color32 {
    match state {
        LoadState::Loaded => Color32::GRAY,
        LoadState::NotFound => Color32::RED,
        _ => todo!(),
    }
}

pub type ActiveStateLabel = ActiveState;

impl Widget for ActiveStateLabel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal_wrapped(|ui| {
            ui.label("Active:");
            ui.colored_label(active_state_to_color(self), self.to_string());
            match self {
                ActiveState::Activating | ActiveState::Reloading | ActiveState::Deactivating => {
                    ui.spinner();
                }
                _ => (),
            };
        })
        .response
    }
}

pub type LoadStateLabel = LoadState;

impl Widget for LoadStateLabel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal_wrapped(|ui| {
            ui.label("Loaded:");
            ui.colored_label(load_state_to_color(self), self.to_string());
        })
        .response
    }
}

pub type UnitFileStateLabel = UnitFileState;
impl Widget for UnitFileStateLabel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal_wrapped(|ui| {
            ui.label("State:");
            ui.label(self.to_string());
        })
        .response
    }
}

pub type UnitFilePresetLabel = UnitFilePreset;
impl Widget for UnitFilePresetLabel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal_wrapped(|ui| {
            ui.label("Preset:");
            ui.label(self.to_string());
        })
        .response
    }
}
