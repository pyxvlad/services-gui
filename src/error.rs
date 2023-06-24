use std::fmt::Display;
use tokio::sync::mpsc::error::SendError;

use tokio::task::JoinError;

use crate::widgets::journal::Entry;

#[derive(Debug)]
pub enum Error {
    Systemd(systemd::Error),
    Zbus(zbus::Error),
    MPSC(SendError<crate::widgets::journal::Entry>),
    Join(JoinError),
    Custom(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Systemd(err) => write!(f, "systemd: {err}"),
            Self::Zbus(err) => write!(f, "zbus: {err}"),
            Self::Custom(err) => write!(f, "custom: {err}"),
            Self::MPSC(err) => write!(f, "tokio mpsc: {err}"),
            Self::Join(err) => write!(f, "join: {err}"),
        }
    }
}

impl From<systemd::Error> for Error {
    fn from(value: systemd::Error) -> Self {
        Self::Systemd(value)
    }
}

impl From<zbus::Error> for Error {
    fn from(value: zbus::Error) -> Self {
        Self::Zbus(value)
    }
}

impl From<SendError<Entry>> for Error {
    fn from(value: SendError<Entry>) -> Self {
        Self::MPSC(value)
    }
}

impl From<JoinError> for Error {
    fn from(value: JoinError) -> Self {
        Self::Join(value)
    }
}

impl std::error::Error for Error {}
