use std::fmt::Display;
use zvariant::OwnedObjectPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    Stub,
    Loaded,
    NotFound,
    BadSetting,
    Error,
    Merged,
    Masked,
}

impl From<&str> for LoadState {
    fn from(value: &str) -> Self {
        match value {
            "stub" => LoadState::Stub,
            "loaded" => LoadState::Loaded,
            "not-found" => LoadState::NotFound,
            "bad-setting" => LoadState::BadSetting,
            "error" => LoadState::Error,
            "merged" => LoadState::Merged,
            "masked" => LoadState::Masked,
            _ => todo!("{value}"),
        }
    }
}

impl From<String> for LoadState {
    fn from(value: String) -> Self {
        Self::from(value.as_ref())
    }
}

impl Display for LoadState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stub => write!(f, "stub"),
            Self::Loaded => write!(f, "loaded"),
            Self::NotFound => write!(f, "not-found"),
            Self::BadSetting => write!(f, "bad-setting"),
            Self::Error => write!(f, "error"),
            Self::Merged => write!(f, "merged"),
            Self::Masked => write!(f, "masked"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ActiveState {
    Active,
    Reloading,
    Inactive,
    Failed,
    Activating,
    Deactivating,
}

impl From<&str> for ActiveState {
    fn from(value: &str) -> Self {
        match value {
            "active" => Self::Active,
            "reloading" => Self::Reloading,
            "inactive" => Self::Inactive,
            "failed" => Self::Failed,
            "activating" => Self::Activating,
            "deactivating" => Self::Deactivating,
            _ => todo!(),
        }
    }
}

impl From<String> for ActiveState {
    fn from(value: String) -> Self {
        Self::from(value.as_ref())
    }
}

impl Display for ActiveState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Reloading => write!(f, "reloading"),
            Self::Inactive => write!(f, "inactive"),
            Self::Failed => write!(f, "failed"),
            Self::Activating => write!(f, "activating"),
            Self::Deactivating => write!(f, "deactivating"),
        }
    }
}

impl ActiveState {
    pub fn can_start(self) -> bool {
        matches!(self, Self::Inactive | Self::Failed)
    }
}

#[derive(Clone, Copy)]
pub enum UnitFileState {
    Enabled,
    Disabled,
    Static,
}

impl From<&str> for UnitFileState {
    fn from(value: &str) -> Self {
        match value {
            "enabled" => Self::Enabled,
            "disabled" => Self::Disabled,
            "static" => Self::Static,
            _ => todo!("this: {value}"),
        }
    }
}

impl From<String> for UnitFileState {
    fn from(value: String) -> Self {
        Self::from(value.as_ref())
    }
}

impl UnitFileState {
    pub fn can_enable(self) -> bool {
        matches!(self, Self::Disabled)
    }
    pub fn can_disable(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

impl Display for UnitFileState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => f.write_str("disabled"),
            Self::Enabled => f.write_str("enabled"),
            Self::Static => f.write_str("static"),
        }
    }
}

pub struct UnitFilePreset(UnitFileState);
impl From<&str> for UnitFilePreset {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}
impl From<String> for UnitFilePreset {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}
impl Display for UnitFilePreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct UnitData {
    pub name: String,
    pub description: String,
    pub load_status: LoadState,
    pub active_status: ActiveState,
    pub object_path: OwnedObjectPath,
}

impl
    From<&(
        String,
        String,
        String,
        String,
        String,
        String,
        OwnedObjectPath,
        u32,
        String,
        OwnedObjectPath,
    )> for UnitData
{
    /// NOTE: read org.freedesktop.systemd1(5), the ListUnits() part
    fn from(
        value: &(
            String,
            String,
            String,
            String,
            String,
            String,
            OwnedObjectPath,
            u32,
            String,
            OwnedObjectPath,
        ),
    ) -> Self {
        UnitData {
            name: value.0.clone(),
            description: value.1.clone(),
            load_status: value.2.as_str().into(),
            active_status: value.3.as_str().into(),
            object_path: value.6.clone(),
        }
    }
}

pub async fn list_units(con: zbus::Connection) -> zbus::Result<Vec<UnitData>> {
    let manager = zbus_systemd::systemd1::ManagerProxy::new(&con).await?;
    let mut units = manager
        .list_units()
        .await?
        .iter()
        .map(UnitData::from)
        .filter(|u| u.name.ends_with(".service"))
        .collect::<Vec<UnitData>>();
    units.sort_by_key(|u| u.name.clone());

    Ok(units)
}
