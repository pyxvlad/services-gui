use super::unitdata::{ActiveStateLabel, LoadStateLabel, UnitFilePresetLabel, UnitFileStateLabel};
use crate::systemd::{self, ActiveState, LoadState, UnitFilePreset, UnitFileState};
use egui::{Color32, Context, Label, Ui, Widget, Window};
use poll_promise::Promise;
use zbus_systemd::systemd1::{ManagerProxy, UnitProxy};
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

                            let result =
                                self.build_buttons(ui, unit.object_path.clone(), unit.name.clone());
                            if let Err(err) = result {
                                ui.colored_label(Color32::DEBUG_COLOR, format!("ERROR: {err}"));
                                println!("ERROR at {}:{}: {err}", file!(), line!());
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

    fn build_buttons(&self, ui: &mut Ui, path: OwnedObjectPath, name: String) -> zbus::Result<()> {
        if ui.button("Restart Unit").clicked() {
            self.restart(path.clone())?;
        }
        if self.can_start(path.clone())? {
            if ui.button("Start Unit").clicked() {
                self.start(path.clone())?;
            }
        } else if ui.button("Stop Unit").clicked() {
            self.stop(path.clone())?;
        }

        let ufs = self.unit_file_state(path)?;
        if ufs.can_enable() && ui.button("Enable").clicked() {
            self.enable_units(vec![name])?;
        } else if ufs.can_disable() && ui.button("Disable").clicked() {
            self.disable_units(vec![name])?;
        }

        Ok(())
    }

    fn can_start(&self, path: OwnedObjectPath) -> zbus::Result<bool> {
        let con = self.con.clone();
        Promise::spawn_async(async move {
            Ok(
                ActiveState::from(UnitProxy::new(&con, path).await?.active_state().await?)
                    .can_start(),
            )
        })
        .block_and_take()
    }

    fn start(&self, path: OwnedObjectPath) -> zbus::Result<OwnedObjectPath> {
        let con = self.con.clone();
        Promise::spawn_async(async move {
            UnitProxy::new(&con, path)
                .await?
                .start("replace".to_owned())
                .await
        })
        .block_and_take()
    }
    fn restart(&self, path: OwnedObjectPath) -> zbus::Result<OwnedObjectPath> {
        let con = self.con.clone();
        Promise::spawn_async(async move {
            UnitProxy::new(&con, path)
                .await?
                .restart("replace".to_owned())
                .await
        })
        .block_and_take()
    }
    fn stop(&self, path: OwnedObjectPath) -> zbus::Result<OwnedObjectPath> {
        let con = self.con.clone();
        Promise::spawn_async(async move {
            UnitProxy::new(&con, path)
                .await?
                .stop("replace".to_owned())
                .await
        })
        .block_and_take()
    }
    fn unit_file_state(&self, path: OwnedObjectPath) -> zbus::Result<UnitFileState> {
        let con = self.con.clone();
        Promise::spawn_async(async move {
            Ok(UnitFileState::from(
                UnitProxy::new(&con, path).await?.unit_file_state().await?,
            ))
        })
        .block_and_take()
    }
    fn enable_units(&self, units: Vec<String>) -> zbus::Result<()> {
        let con = self.con.clone();
        Promise::spawn_async(async move {
            ManagerProxy::new(&con)
                .await?
                .enable_unit_files(units, true, false)
                .await?;
            Ok(())
        })
        .block_and_take()
    }
    fn disable_units(&self, units: Vec<String>) -> zbus::Result<()> {
        let con = self.con.clone();
        Promise::spawn_async(async move {
            ManagerProxy::new(&con)
                .await?
                .disable_unit_files(units, false)
                .await?;
            Ok(())
        })
        .block_and_take()
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
