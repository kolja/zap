use dirs::home_dir;
use filetime::{self, set_file_atime, set_file_mtime};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod args;
pub mod errors;
pub mod file_time_util;
pub mod parsedate;
pub mod plugins;

use anyhow::Result;
use dialoguer::Confirm;
use tera::{Context, Tera};

// use crate::parsedate;
use crate::args::ZapCli;
use crate::errors::ZapError;
use crate::file_time_util::{FileTimeSpec, adjust_file_times_from_metadata};
use crate::plugins::Plugins;

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
    let template_name: Option<&str> = template.as_deref();
    let context_str: Option<&str> = context.as_deref();

    let new_times: FileTimeSpec;

    // After parsing, at most one of these will be `Some`.
    if let Some(date_str) = date {
        let parsed_date = parsedate::parse_d_format(date_str)?;
        new_times = FileTimeSpec::from_datetime(parsed_date);
    } else if let Some(timestamp_str) = timestamp {
        let parsed_date = parsedate::parse_t_format(timestamp_str)?;
        new_times = FileTimeSpec::from_datetime(parsed_date);
    } else if let Some(reference_path) = reference {
        let ref_path = Path::new(reference_path);
        if !ref_path.exists() {
            return Err(ZapError::ReferenceFileNotFound(reference_path.clone()).into());
        }
        let metadata = std::fs::metadata(ref_path)?;
        new_times = FileTimeSpec::from_metadata(&metadata);
    } else {
        new_times = FileTimeSpec::now();
    }

    let (should_update_access, should_update_modification) = cli.should_update_times();
    let file_times = new_times.with_flags(should_update_access, should_update_modification);

    for filename in filenames {
        let path = Path::new(&filename);

        if path.exists() && adjust.is_some() {
            let metadata = std::fs::metadata(path)?;
            let adjustment_str = adjust.as_deref().unwrap(); // we know it's Some here

            let adjusted_times = adjust_file_times_from_metadata(&metadata, adjustment_str)?
                .with_flags(should_update_access, should_update_modification);

            set_file_times(path, &adjusted_times)?;
            continue; // Skip file creation: with the -A flag, we only adjust times
        }

        if path.exists() && template_name.is_some() {
            let confirmation = Confirm::new()
                .with_prompt(format!(
                    "File '{filename}' already exists. Do you want to overwrite it?",
                ))
                .default(false)
                .interact()?;
            if !confirmation {
                continue; // Skip to the next file if user doesn't agree with overwrite
            }
        }

        // should we create intermediate directories?
        if let Some(parent) = path.parent()
            && parent.components().next().is_some()
            && !parent.exists()
            && !no_create
        {
            let confirmation = Confirm::new()
                .with_prompt(format!(
                    "The directory {:?} doesn't exist. Create it?",
                    parent.display()
                ))
                .default(false)
                .interact()?;

            if !confirmation {
                continue; // Skip to the next file
            }
            std::fs::create_dir_all(parent)?;
        }

        if !path.exists() && *no_create {
            continue; // Skip file creation if the file does not exist and no_create is true
        }

        // If file exists and we have no template, just update times without recreating
        if path.exists() && template_name.is_none() {
            set_file_times(path, &file_times)?;
            continue;
        }

        // Create or recreate the file (for new files or when using templates)
        let mut file = File::create(path)?;
        set_file_times(path, &file_times)?;

        if let Some(tmpl_name) = template_name {
            let template_path_full = get_template_path(tmpl_name)?;

            if !template_path_full.exists() {
                return Err(ZapError::TemplateNotFound(template_path_full).into());
            }

            let mut tera = Tera::default();

            tera.add_template_file(&template_path_full, Some(tmpl_name))?;

            let mut plugins = Plugins::new();
            let plugins_dir = get_config_dir()?.join("plugins");
            plugins.load_plugins_from_dir(&mut tera, &plugins_dir)?;

            let mut context = Context::new();
            if let Some(ctx) = &context_str {
                for pair in ctx.split(',') {
                    let mut parts = pair.splitn(2, '=');
                    if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                        context.insert(key.trim(), value.trim());
                    }
                }
            }
            let rendered = tera.render(tmpl_name, &context)?;
            file.write_all(rendered.as_bytes())?;
        }
    }
    Ok(())
}

pub fn open_in_editor(filepaths: &Vec<String>) -> Result<(), ZapError> {
    let editor_env_var = env::var("EDITOR").map_err(|_| ZapError::EditorNotSet)?;

    let mut parts = editor_env_var.split_whitespace();
    let editor_executable = parts
        .next()
        .ok_or_else(|| ZapError::EditorCommandParseError(editor_env_var.clone()))?;

    let mut cmd = Command::new(editor_executable);
    cmd.args(parts);
    cmd.args(filepaths);

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(ZapError::EditorExitedWithError(
                    editor_env_var,
                    status.code(),
                ))
            }
        }
        Err(e) => Err(ZapError::EditorSpawnFailed(editor_env_var, e)),
    }
}
