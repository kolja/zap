use dirs::home_dir;
use filetime::{self, set_file_atime, set_file_mtime};
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
use crate::fileaction::{Planner, open_in_editor};

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

pub fn set_file_times(path: &Path, times: &FileTimeSpec) -> Result<(), ZapError> {
    match (times.atime, times.mtime) {
        (Some(atime), Some(mtime)) =>
        // Both: use the combined call for efficiency (only one syscall)
        {
            filetime::set_file_times(path, atime, mtime).map_err(ZapError::SetTimesError)
        }
        (Some(atime), None) => set_file_atime(path, atime).map_err(ZapError::SetTimesError),
        (None, Some(mtime)) => set_file_mtime(path, mtime).map_err(ZapError::SetTimesError),
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
        adjust,
        date,
        timestamp,
        reference,
        ..
    } = cli;

    // Time calculation logic
    let new_times: FileTimeSpec = if let Some(date_str) = date {
        let parsed_date = parsedate::parse_d_format(date_str)?;
        FileTimeSpec::from_datetime(parsed_date)
    } else if let Some(timestamp_str) = timestamp {
        let parsed_date = parsedate::parse_t_format(timestamp_str)?;
        FileTimeSpec::from_datetime(parsed_date)
    } else if let Some(reference_path) = reference {
        let ref_path = Path::new(reference_path);
        if !ref_path.exists() {
            return Err(ZapError::ReferenceFileNotFound(reference_path.clone()).into());
        }
        let metadata = std::fs::metadata(ref_path)?;
        FileTimeSpec::from_metadata(&metadata)
    } else {
        FileTimeSpec::now()
    };

    let (should_update_access, should_update_modification) = cli.should_update_times();
    let file_times = new_times.with_flags(should_update_access, should_update_modification);

    // Create the planner
    let planner = Planner {
        no_create: *no_create,
        adjust: adjust.as_deref(),
        template: template.as_deref(),
        context: context.as_deref(),
        should_update_access,
        should_update_modification,
    };

    // Process each file
    for filename in filenames {
        let path = Path::new(filename);

        // Plan what action to take
        let action = planner.plan(path, &file_times)?;

        // Execute the action
        action.execute(path, filename)?;
    }

    // Open editor if requested
    if cli.open {
        if let Err(e) = open_in_editor(&cli.filenames) {
            eprintln!("Warning: Could not open editor: {e}");
        }
    }

    Ok(())
}
