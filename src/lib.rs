use dirs::home_dir;

use std::path::{Path, PathBuf};

pub mod args;
pub mod errors;
pub mod file_time_util;
pub mod fileaction;
pub mod parsedate;
pub mod plugins;

use anyhow::Result;

// use crate::parsedate;
use crate::args::ZapCli;
use crate::errors::ZapError;
use crate::file_time_util::FileTimeSpec;
use crate::fileaction::{Planner, execute_actions, open_in_editor};

fn get_config_dir() -> Result<PathBuf, ZapError> {
    let conf_dir: Option<PathBuf> = home_dir();
    conf_dir
        .ok_or(ZapError::ConfigDirNotFound)
        .map(|path| path.join(".config/zap"))
}

fn get_template_path(template_name: &str) -> Result<PathBuf, ZapError> {
    let config_dir = get_config_dir()?;
    let mut template_path = PathBuf::from(&config_dir);
    template_path.extend(["templates", template_name]);
    Ok(template_path)
}

pub fn set_file_times(
    path: &Path,
    times: &FileTimeSpec,
    symlink_only: bool,
) -> Result<(), ZapError> {
    match (times.atime, times.mtime) {
        (Some(atime), Some(mtime)) => {
            // Both: use the combined call for efficiency (only one syscall)
            file_time_util::set_both_times(path, atime, mtime, symlink_only)
        }
        (Some(atime), None) => file_time_util::set_access_time_only(path, atime, symlink_only),
        (None, Some(mtime)) => {
            file_time_util::set_modification_time_only(path, mtime, symlink_only)
        }
        (None, None) => Ok(()),
    }
}

/// zap: Create a file if it doesn't exist,
/// optionally populate it with text from a template.
/// If the file exists, its modification and access times are updated.
pub fn zap(cli: &ZapCli) -> Result<(), anyhow::Error> {
    let ZapCli {
        filenames,
        template,
        context,
        no_create,
        create_intermediate_dirs,
        adjust,
        date,
        timestamp,
        reference,
        symlink_only,
        ..
    } = cli;

    // Time calculation logic
    let explicit_times: Option<FileTimeSpec> = if let Some(date_str) = date {
        let parsed_date = parsedate::parse_d_format(date_str)?;
        Some(FileTimeSpec::from_datetime(parsed_date))
    } else if let Some(timestamp_str) = timestamp {
        let parsed_date = parsedate::parse_t_format(timestamp_str)?;
        Some(FileTimeSpec::from_datetime(parsed_date))
    } else if let Some(reference_path) = reference {
        let ref_path = Path::new(reference_path);
        if !ref_path.exists() {
            return Err(ZapError::ReferenceFileNotFound(reference_path.clone()).into());
        }
        let metadata = std::fs::metadata(ref_path)?;
        Some(FileTimeSpec::from_metadata(&metadata))
    } else {
        None
    };

    let (should_update_access, should_update_modification) = cli.should_update_times();

    // Create the planner
    let planner = Planner {
        no_create: *no_create,
        adjust: adjust.as_deref(),
        template: template.as_deref(),
        context: context.as_deref(),
        should_update_access,
        should_update_modification,
        create_intermediate_dirs: *create_intermediate_dirs,
        symlink_only: *symlink_only,
    };

    // Process each file
    for filename in filenames {
        let path = Path::new(filename);

        // Plan what actions to take
        let actions = planner.plan(path, explicit_times.as_ref())?;

        // Execute the actions
        execute_actions(actions, path, filename, *create_intermediate_dirs)?;
    }

    // Open editor if requested
    if cli.open {
        if let Err(e) = open_in_editor(&cli.filenames) {
            eprintln!("Warning: Could not open editor: {e}");
        }
    }

    Ok(())
}
