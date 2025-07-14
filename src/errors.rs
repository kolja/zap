use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginLoadError {
    #[error("Plugin directory not found or is not a directory: {0:?}")]
    DirectoryNotFound(PathBuf),

    #[error("Failed to read plugin directory {path:?}: {source}")]
    DirectoryRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to load plugin library from {path:?}: {source}")]
    LibraryLoad {
        path: PathBuf,
        #[source]
        source: libloading::Error,
    },

    #[error("Entry point '{entry_point_name}' not found in plugin {plugin_path:?}: {source}")]
    EntryPointNotFound {
        plugin_path: PathBuf,
        entry_point_name: String,
        #[source]
        source: libloading::Error,
    },

    #[error("Plugin path contains invalid UTF-8: {0:?}")]
    InvalidPath(PathBuf),
}

// Custom wrapper for Tera errors
#[derive(Debug)]
pub struct TeraError(pub tera::Error);

impl fmt::Display for TeraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Tera templating Error: ")?;
        format_tera_error_kind(&self.0.kind, f)?;

        if let Some(source) = self.0.source() {
            if let Some(tera_source_error) = source.downcast_ref::<tera::Error>() {
                f.write_str("\ncaused by:\n")?;
                format_tera_error_kind(&tera_source_error.kind, f)?;
            } else {
                write!(f, "\ncaused by:\n{source}")?;
            }
        }
        Ok(())
    }
}

impl StdError for TeraError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.0)
    }
}

impl From<tera::Error> for TeraError {
    fn from(err: tera::Error) -> Self {
        TeraError(err)
    }
}

fn format_tera_error_kind(kind: &tera::ErrorKind, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match kind {
        tera::ErrorKind::Msg(s) => f.write_str(s),
        _ => write!(f, "{kind:?}"),
    }
}

#[derive(Error, Debug)]
pub enum ZapError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error(transparent)]
    Tera(#[from] TeraError),

    #[error("Could not find user config directory")]
    ConfigDirNotFound,

    #[error("Template file not found: {0:?}")]
    TemplateNotFound(PathBuf),

    #[error("Failed to set file times: {0}")]
    SetTimesError(io::Error),

    #[error("Dialoguer error: {0}")]
    Dialoguer(#[from] dialoguer::Error),

    #[error("EDITOR environment variable not set")]
    EditorNotSet,

    #[error("EDITOR command '{0}' could not be parsed (is it empty?)")]
    EditorCommandParseError(String),

    #[error("Failed to spawn editor '{0}': {1}")]
    EditorSpawnFailed(String, io::Error),

    #[error("Editor '{0}' exited with non-zero status: {1:?}")]
    EditorExitedWithError(String, Option<i32>),

    #[error("Plugin system error: {0}")]
    PluginSystem(#[from] PluginLoadError),

    // For -d format specifically, wrapping chrono's error or a generic message.
    #[error("Invalid RFC3339 date-time string '{input}': {source}")]
    DateTimeRfc3339 {
        input: String,
        #[source]
        source: chrono::ParseError,
    },

    #[error("Invalid RFC3339 date-time string '{input}': {reason}")]
    DateTimeRfc3339Generic { input: String, reason: String },

    // For -t format, which has more custom parsing logic.
    #[error("Invalid custom date-time string '{input}' for -t format: {reason}")]
    DateTimeTCustom { input: String, reason: String },

    #[error("Integer parsing failed for a date-time component: {0}")]
    DateTimeParseInt(#[from] std::num::ParseIntError),

    #[error(
        "Date-time component '{component}' with value '{value}' is out of valid range: {details}"
    )]
    DateTimeComponentRange {
        component: String,
        value: String,
        details: String,
    },

    #[error(
        "Specified local time '{naive_datetime}' is ambiguous or non-existent due to DST or other calendar transition"
    )]
    AmbiguousOrInvalidLocalTime { naive_datetime: String },

    // Time adjustment errors
    #[error("Time adjustment would cause overflow")]
    TimeAdjustmentOverflow,

    #[error("Time adjustment would cause underflow")]
    TimeAdjustmentUnderflow,

    #[error("Failed to parse time adjustment: {0}")]
    TimeAdjustmentParse(String),

    #[error("Failed to convert between time representations")]
    TimeConversionError,
}

// Provide a direct conversion from tera::Error to ZapError for convenience
impl From<tera::Error> for ZapError {
    fn from(err: tera::Error) -> Self {
        ZapError::Tera(TeraError::from(err))
    }
}
