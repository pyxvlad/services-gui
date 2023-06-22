use std::fmt::Display;
use zvariant::OwnedObjectPath;

#[derive(Debug, Clone, Copy)]
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
    fn can_start(self) -> bool {
        match self {
            Self::Inactive | Self::Failed => true,
            _ => false,
        }
    }

    fn can_stop(self) -> bool {
        match self {
            Self::Active => true,
            _ => false,
        }
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

pub async fn list_system_units() -> zbus::Result<Vec<UnitData>> {
    let conn = zbus::Connection::system().await?;

    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn).await?;
    let units = manager
        .list_units()
        .await?
        .iter()
        .map(|data| UnitData::from(data))
        .filter(|u| u.name.ends_with(".service"))
        .collect::<Vec<UnitData>>();

    Ok(units)
}

pub async fn list_user_units() -> zbus::Result<Vec<UnitData>> {
    let conn = zbus::Connection::session().await?;

    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn).await?;
    let units = manager
        .list_units()
        .await?
        .iter()
        .map(|data| UnitData::from(data))
        .filter(|u| u.name.ends_with(".service"))
        .collect::<Vec<UnitData>>();

    Ok(units)
}

pub async fn get_unit(unit_name: String) -> zbus::Result<()> {
    let conn = zbus::Connection::session().await?;
    let manager = zbus_systemd::systemd1::ManagerProxy::new(&conn).await?;

    let obj = manager.get_unit(unit_name).await?;
    let proxy = zbus_systemd::systemd1::UnitProxy::new(&conn, obj).await?;

    proxy.start("".to_string());

    Ok(())
}
