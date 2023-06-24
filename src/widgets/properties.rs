use super::unitdata::{ActiveStateLabel, LoadStateLabel, UnitFilePresetLabel, UnitFileStateLabel};
use crate::systemd::{self, ActiveState, LoadState, UnitFilePreset, UnitFileState};
use egui::{Context, Label, Ui, Widget, Window};
use poll_promise::Promise;
use zvariant::OwnedObjectPath;

#[derive(Clone)]
pub struct PropertiesWindow {
    unit: Option<usize>,
    open: bool,

    con: zbus::Connection,
}

impl PropertiesWindow {
    pub fn with_connection(con: zbus::Connection) -> PropertiesWindow {
        Self {
            unit: None,
            open: false,
            con,
        }
    }

    pub fn draw<'a_, F>(&mut self, ctx: &Context, extractor: &F)
    where
        F: Fn(usize) -> Option<&'a_ systemd::UnitData>,
    {
        match self.unit {
            Some(index) => {
                let unit_opt = extractor(index);

                let mut open = self.open;
                if let Some(unit) = unit_opt {
                    Window::new("Service Properties")
                        .resizable(true)
                        .pivot(egui::Align2::CENTER_CENTER)
                        .open(&mut open)
                        .show(ctx, |ui| {
                            ui.label(format!("Name: {}", unit.name));
                            ui.label(unit.description.as_str());
                            ui.label(unit.load_status.to_string());

                            match Promise::spawn_async(build_ui(
                                self.con.clone(),
                                unit.object_path.clone(),
                            ))
                            .block_and_take()
                            {
                                Ok(mut widgets) => {
                                    // It may seem strange, but here's the explanation: I am
                                    // pushing in a specific order into this vector and then I am
                                    // `pop()`ing the elements, which results into a reverse
                                    // order... So I am reversing it first, so it is `pop()`ed in
                                    // the correct order...
                                    widgets.reverse();
                                    while let Some(widget) = widgets.pop() {
                                        ui.add(widget);
                                    }
                                }
                                Err(err) => {
                                    ui.heading(err.to_string());
                                }
                            }
                        });
                }
                self.open = open;
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

enum PropertiesWidget {
    Label(Label),
    ActiveState(ActiveStateLabel),
    LoadState(LoadStateLabel),
    UnitFileState(UnitFileStateLabel),
    UnitFilePreset(UnitFilePresetLabel),
}

impl Widget for PropertiesWidget {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        match self {
            Self::Label(w) => w.ui(ui),
            Self::ActiveState(w) => w.ui(ui),
            Self::LoadState(w) => w.ui(ui),
            Self::UnitFileState(w) => w.ui(ui),
            Self::UnitFilePreset(w) => w.ui(ui),
        }
    }
}

impl From<Label> for PropertiesWidget {
    fn from(value: Label) -> Self {
        Self::Label(value)
    }
}

impl From<ActiveState> for PropertiesWidget {
    fn from(value: ActiveState) -> Self {
        Self::ActiveState(value)
    }
}

impl From<LoadState> for PropertiesWidget {
    fn from(value: LoadState) -> Self {
        Self::LoadState(value)
    }
}

impl From<UnitFileState> for PropertiesWidget {
    fn from(value: UnitFileState) -> Self {
        Self::UnitFileState(value)
    }
}

impl From<UnitFilePreset> for PropertiesWidget {
    fn from(value: UnitFilePreset) -> Self {
        Self::UnitFilePreset(value)
    }
}

async fn build_ui(
    con: zbus::Connection,
    path: OwnedObjectPath,
) -> zbus::Result<Vec<PropertiesWidget>> {
    let proxy = zbus_systemd::systemd1::UnitProxy::new(&con, path).await?;
    let mut vec: Vec<PropertiesWidget> = Vec::new();
    vec.push(ActiveState::from(proxy.active_state().await?).into());
    let load_state = LoadState::from(proxy.load_state().await?);
    vec.push(load_state.into());
    if load_state == LoadState::NotFound {
        return Ok(vec);
    }
    vec.push(UnitFileState::from(proxy.unit_file_state().await?).into());
    vec.push(systemd::UnitFilePreset::from(proxy.unit_file_preset().await?).into());
    Ok(vec)
}
