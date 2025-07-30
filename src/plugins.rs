use libloading::{Library, Symbol};
use std::fs;
use std::path::Path;
use tera;

use crate::errors::PluginLoadError;

type PluginRegisterFn = unsafe extern "C" fn(tera: &mut tera::Tera);
const PLUGIN_ENTRY_POINT: &[u8] = b"register_tera_custom_functions";

pub struct Plugins {
    libs: Vec<Library>,
}

impl Default for Plugins {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugins {
    pub fn new() -> Self {
        Plugins { libs: Vec::new() }
    }

    pub fn load_plugin(
        &mut self,
        tera: &mut tera::Tera,
        plugin_path: &Path,
    ) -> Result<(), PluginLoadError> {
        unsafe {
            let lib = Library::new(plugin_path).map_err(|e| PluginLoadError::LibraryLoad {
                path: plugin_path.to_path_buf(),
                source: e,
            })?;

            self.libs.push(lib);
            let lib_ref = self.libs.last().unwrap(); // Safe as we just pushed

            // For error reporting, convert the entry point name to a String
            let entry_point_name_str = String::from_utf8_lossy(PLUGIN_ENTRY_POINT).into_owned();

            let register_fn: Symbol<PluginRegisterFn> =
                lib_ref.get(PLUGIN_ENTRY_POINT).map_err(|e| {
                    PluginLoadError::EntryPointNotFound {
                        plugin_path: plugin_path.to_path_buf(),
                        entry_point_name: entry_point_name_str,
                        source: e,
                    }
                })?;

            register_fn(tera);
        }
        Ok(())
    }

    pub fn load_plugins_from_dir(
        &mut self,
        tera: &mut tera::Tera,
        dir_path: &Path,
    ) -> Result<(), PluginLoadError> {

        // If the plugins directory doesn't exist, just return OK without loading any plugins
        if !dir_path.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir_path).map_err(|e| PluginLoadError::DirectoryRead {
            path: dir_path.to_path_buf(),
            source: e,
        })? {
            let entry = entry.map_err(|e| PluginLoadError::DirectoryRead {
                path: dir_path.to_path_buf(),
                source: e,
            })?;
            let path = entry.path();

            let ext = path.extension().and_then(std::ffi::OsStr::to_str);
            if !matches!(ext, Some("so") | Some("dylib") | Some("dll")) {
                continue;
            }

            self.load_plugin(tera, &path).map_err(|e| {
                eprintln!("Warning: Failed to load plugin {path:?}: {e}");
                e
            })?;
        }
        Ok(())
    }
}
