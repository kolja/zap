
use std::path::PathBuf;
use std::io;
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
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Tera templating error: {0}")]
    Tera(#[from] tera::Error),
    #[error("Could not find user config directory")]
    ConfigDirNotFound,
    #[error("Template file not found: {0}")]
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
}
