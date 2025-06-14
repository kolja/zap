use filetime::{self, set_file_times as SetFileTimes};
use filetime::FileTime;
use std::fs::File;
use std::io::Write;
use std::env;
use std::process::Command;
use std::path::{Path, PathBuf};
use dirs::home_dir;

pub mod args;
pub mod plugins;
pub mod errors;

use dialoguer::Confirm;
use tera::{Context, Tera};

use crate::plugins::Plugins;
use crate::errors::ZapError;
use crate::args::ZapCli;

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

/// zap: Create a file if it doesn't exist,
/// optionally populate it with text from a template.
/// If the file exists, its modification and access times are updated.
pub fn zap(args: &ZapCli) -> Result<(), ZapError> {

    let filenames = &args.filenames;
    let template_name = args.template.as_deref();
    let context_str = args.context.as_deref();
    // let access_time = args.access_time;
    // let modification_time = args.modification_time;

    for filename in filenames {

        let path = Path::new(filename);
        let now = FileTime::now();

        if path.exists() {
            if template_name.is_none() { // If no template is provided, just update the file times
                return SetFileTimes(path, now, now).map_err(ZapError::SetTimesError);
            } else {
                let confirmation = Confirm::new()
                    .with_prompt(format!("File '{}' already exists. Do you want to overwrite it?", filename))
                    .default(false)
                    .interact()?;
                if !confirmation {
                    continue; // Skip to the next file if user doesn't agree with overwrite
                }
            }
        }

        let mut file = File::create(path)?;

        if let Some(tmpl_name) = template_name {
            let template_path_full = get_template_path(tmpl_name)?;

            if !template_path_full.exists() {
                return Err(ZapError::TemplateNotFound(template_path_full));
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
    let editor_executable = parts.next()
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
