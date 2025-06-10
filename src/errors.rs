
use std::path::PathBuf;
use std::{io, fmt};
use std::error::Error as StdError;
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
        entry_point_name: String, // Using String as entry point name can vary
        #[source]
        source: libloading::Error,
    },

    #[error("Plugin path contains invalid UTF-8: {0:?}")]
    InvalidPath(PathBuf),
}

#[derive(Error, Debug)]
pub enum ZapError {
    Io(#[from] io::Error),
    Tera(#[from] tera::Error),
    ConfigDirNotFound,
    TemplateNotFound(PathBuf),
    SetTimesError(io::Error),
    Dialoguer(#[from] dialoguer::Error),
    EditorNotSet,
    EditorCommandParseError(String),
    EditorSpawnFailed(String, io::Error),
    EditorExitedWithError(String, Option<i32>),
    PluginSystem(#[from] PluginLoadError),
}

fn format_tera_error_kind(kind: &tera::ErrorKind, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match kind {
        tera::ErrorKind::Msg(s) => f.write_str(s),
        _ => write!(f, "{:?}", kind),
    }
}

impl fmt::Display for ZapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZapError::Io(err) => write!(f, "I/O error: {}", err),
            ZapError::Tera(err) => {
                f.write_str("Tera templating Error: ")?;
                format_tera_error_kind(&err.kind, f)?;

                if let Some(source) = err.source() {
                    if let Some(tera_source_error) = source.downcast_ref::<tera::Error>() {
                        f.write_str("\ncaused by:\n")?;
                        format_tera_error_kind(&tera_source_error.kind, f)?;
                    } else {
                        write!(f, "\ncaused by:\n{}", source)?;
                    }
                }
                Ok(())
            }
            ZapError::ConfigDirNotFound => write!(f, "Could not find user config directory"),
            ZapError::TemplateNotFound(path) => write!(f, "Template file not found: {:?}", path),
            ZapError::SetTimesError(err) => write!(f, "Failed to set file times: {}", err),
            ZapError::Dialoguer(err) => write!(f, "Dialoguer error: {}", err),
            ZapError::EditorNotSet => write!(f, "EDITOR environment variable not set"),
            ZapError::EditorCommandParseError(cmd) => {
                write!(f, "EDITOR command '{}' could not be parsed (is it empty?)", cmd)
            }
            ZapError::EditorSpawnFailed(cmd, err) => {
                write!(f, "Failed to spawn editor '{}': {}", cmd, err)
            }
            ZapError::EditorExitedWithError(cmd, status) => {
                write!(f, "Editor '{}' exited with non-zero status: {:?}", cmd, status)
            }
            ZapError::PluginSystem(err) => write!(f, "Plugin system error: {}", err),
        }
    }
}
