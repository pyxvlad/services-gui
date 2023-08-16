
use crate::error::Error;
use chrono::{DateTime, Local};
use poll_promise::Promise;
use std::collections::btree_map::Iter;
use std::default::Default;
use std::fmt::Display;
use std::ops::Index;
use std::slice::SliceIndex;
use std::time::{Duration, SystemTime};
use systemd::journal::OpenOptions;
use tokio::sync::mpsc::{error::TryRecvError, Receiver, Sender};
use tokio::task::JoinHandle;

/// Use the simple formatting for full names or the alternative one for
/// syslog-like names.
#[derive(Debug, Clone, Copy)]
pub enum Priority {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Info
    }
}

impl From<&str> for Priority {
    fn from(value: &str) -> Self {
        match value {
            "0" => Self::Emergency,
            "1" => Self::Alert,
            "2" => Self::Critical,
            "3" => Self::Error,
            "4" => Self::Warning,
            "5" => Self::Notice,
            "6" => Self::Info,
            "7" => Self::Debug,
            _ => panic!("invalid priority {value}"),
        }
    }
}

impl From<String> for Priority {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                Self::Emergency => f.write_str("emerg"),
                Self::Alert => f.write_str("alert"),
                Self::Critical => f.write_str("crit"),
                Self::Error => f.write_str("err"),
                Self::Warning => f.write_str("warning"),
                Self::Notice => f.write_str("notice"),
                Self::Info => f.write_str("info"),
                Self::Debug => f.write_str("debug"),
            }
        } else {
            match self {
                Self::Emergency => f.write_str("emergency"),
                Self::Alert => f.write_str("alert"),
                Self::Critical => f.write_str("critical"),
                Self::Error => f.write_str("error"),
                Self::Warning => f.write_str("warning"),
                Self::Notice => f.write_str("notice"),
                Self::Info => f.write_str("info"),
                Self::Debug => f.write_str("debug"),
            }
        }
    }
}

#[derive(Debug)]
pub struct EntryCommon {
    /// field _SYSTEMD_SLICE=
    pub slice: String,
    /// field _SYSTEMD_UNIT=
    pub systemd_unit: String,
    /// field _COMM=
    pub comm: String,
    /// field SYSLOG_IDENTIFIER=
    pub syslog_id: String,
    /// When this entry was received by the journal.
    pub timestamp: SystemTime,
    /// field MESSAGE=
    pub message: String,
    /// field UNIT= for systemd entries
    pub unit: String,
    /// field PRIORITY=, see levels on syslog(3)
    pub priority: Priority,
}

impl EntryCommon {
    fn with_timestamp(timestamp: SystemTime) -> Self {
        Self {
            timestamp,
            slice: Default::default(),
            systemd_unit: Default::default(),
            comm: Default::default(),
            syslog_id: Default::default(),
            message: Default::default(),
            unit: Default::default(),
            priority: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum Entry {
    System(EntryCommon),
    Service(EntryCommon),
}

impl Entry {
    pub fn is_for_unit(&self, s: &str) -> bool {
        match self {
            Self::System(entry) => entry.unit == s,
            Self::Service(entry) => entry.systemd_unit == s,
        }
    }

    pub fn unit(&self) -> &str {
        match self {
            Self::System(e) => e.unit.as_str(),
            Self::Service(e) => e.systemd_unit.as_str(),
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::System(e) => e.message.as_str(),
            Self::Service(e) => e.message.as_str(),
        }
    }

    pub fn priority(&self) -> Priority {
        match self {
            Self::System(e) => e.priority,
            Self::Service(e) => e.priority,
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            Self::Service(_) => "service",
            Self::System(_) => "system",
        }
    }

    pub fn timestamp(&self) -> DateTime<Local> {
        match self {
            Self::Service(e) => e.timestamp,
            Self::System(e) => e.timestamp,
        }
        .into()
    }
}

pub struct JournalReader {
    entries: Vec<Entry>,
    receiver: Receiver<Entry>,
    work: JoinHandle<Result<(), Error>>,
}

impl JournalReader {
    pub fn new(options: OpenOptions) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(64);
        let work = tokio::task::spawn_blocking(|| worker(sender, options));

        Self {
            entries: Vec::default(),
            receiver,
            work,
        }
    }
    pub fn receive(&mut self) {
        loop {
            let receive = self.receiver.try_recv();
            match receive {
                Ok(entry) => {
                    self.entries.push(entry);
                }

                Err(err) => match err {
                    TryRecvError::Empty => return,
                    TryRecvError::Disconnected => {
                        panic!("This should have been dropped")
                    }
                },
            }
        }
    }

    pub fn continue_filter<P, C>(&self, predicate: &mut P, container: &mut C, last: usize)
    where
        P: FnMut(&Entry) -> bool,
        C: std::iter::Extend<usize>,
    {
        container.extend(self.entries.iter().enumerate().skip(last + 1).filter_map(
            |(index, entry)| {
                if predicate(entry) {
                    Some(index)
                } else {
                    None
                }
            },
        ));
    }

    pub fn filter<P>(&self, predicate: &mut P) -> Vec<usize>
    where
        P: FnMut(&Entry) -> bool,
    {
        self.entries
            .iter()
            .enumerate()
            .filter_map(
                |(index, entry)| {
                    if predicate(entry) {
                        Some(index)
                    } else {
                        None
                    }
                },
            )
            .collect()
    }
}

impl<I: SliceIndex<[Entry]>> Index<I> for JournalReader {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.entries.index(index)
    }
}

fn parse_journal_kv(timestamp: SystemTime, fields_iter: Iter<'_, String, String>) -> Entry {
    let mut common = EntryCommon::with_timestamp(timestamp);
    fields_iter.for_each(|(k, v)| match k.as_ref() {
        "_SYSTEMD_SLICE" => common.slice = v.clone(),
        "_SYSTEMD_UNIT" => common.systemd_unit = v.clone(),
        "SYSLOG_IDENTIFIER" => common.syslog_id = v.clone(),
        "MESSAGE" => common.message = v.clone(),
        "UNIT" => common.unit = v.clone(),
        "PRIORITY" => common.priority = v.clone().into(),
        _ => {
            if cfg!(feature = "panic-for-unimplementd") {
                panic!("implement parsing for {k}")
            }
        }
    });

    if common.syslog_id == "systemd" {
        Entry::System(common)
    } else {
        Entry::Service(common)
    }
}

fn worker(sender: Sender<Entry>, options: OpenOptions) -> Result<(), Error> {
    let mut reader = options.open()?;
    while !sender.is_closed() {
        while let Some(entry) = reader.next_entry()? {
            let timestamp = reader.timestamp()?;
            let entry = parse_journal_kv(timestamp, entry.iter());
            let sender = sender.clone();

            Promise::spawn_async(async move { sender.send(entry).await }).block_and_take()?;
        }
        reader.wait(Some(Duration::from_micros(1)))?;
    }
    Ok(())
}
