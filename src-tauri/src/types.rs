use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use serde::ser::Serializer;
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    Message(String),
    Io {
        context: &'static str,
        source: std::io::Error,
    },
    Json {
        context: &'static str,
        source: serde_json::Error,
    },
    Dll {
        context: &'static str,
        source: libloading::Error,
    },
}

impl AppError {
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    pub fn io(context: &'static str, source: std::io::Error) -> Self {
        Self::Io { context, source }
    }

    pub fn json(context: &'static str, source: serde_json::Error) -> Self {
        Self::Json { context, source }
    }

    pub fn dll(context: &'static str, source: libloading::Error) -> Self {
        Self::Dll { context, source }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => write!(f, "{message}"),
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::Json { context, source } => write!(f, "{context}: {source}"),
            Self::Dll { context, source } => write!(f, "{context}: {source}"),
        }
    }
}

impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            Self::Dll { source, .. } => Some(source),
            Self::Message(_) => None,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        Self::Message(value.to_string())
    }
}

pub type CommandResult<T> = Result<T, AppError>;
