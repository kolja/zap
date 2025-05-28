use filetime::{self, set_file_times as SetFileTimes};
use filetime::FileTime;
use std::fs::File;
use std::env;
use std::process::Command;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use dirs::home_dir;

use clap::Parser;
use tera::{Context, Tera};
use thiserror::Error;

#[derive(Parser, Debug)]
#[clap(name = "zap", author, version, about = "touch, but with templates", long_about = None)]
pub struct ZapCli {
    #[clap(value_parser)]
    pub filename: String,

    /// Optional template name to pre-populate the file.\n
    /// Templates are sourced from ~/.config/zap/<template_name>.
    #[clap(short = 't', long, value_name = "TEMPLATE_NAME")]
    pub template: Option<String>,

    /// Optional context to use when rendering the template.
    #[clap(short = 'c', long, value_name = "CONTEXT")]
    pub context: Option<String>,

    /// Open the file with your $EDITOR
    #[clap(short = 'o', long)]
    pub open: bool,
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

    #[error("EDITOR environment variable not set")]
    EditorNotSet,
    #[error("EDITOR command '{0}' could not be parsed (is it empty?)")]
    EditorCommandParseError(String),
    #[error("Failed to spawn editor '{0}': {1}")]
    EditorSpawnFailed(String, io::Error),
    #[error("Editor '{0}' exited with non-zero status: {1:?}")]
    EditorExitedWithError(String, Option<i32>),
}

fn get_config_dir() -> Result<PathBuf, ZapError> {
    let conf_dir: Option<PathBuf> = home_dir();
    conf_dir
        .ok_or(ZapError::ConfigDirNotFound)
        .map(|path| path.join(".config/zap"))
}

fn get_template_path(template_name: &str) -> Result<PathBuf, ZapError> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join(template_name))
}

/// zap: Create a file if it doesn't exist,
/// optionally populate it with text from a template.
/// If the file exists, its modification and access times are updated.
pub fn zap(filename_str: &str, template_name: Option<&str>, context_str: Option<&str>) -> Result<(), ZapError> {
    let path = Path::new(filename_str);

    // no template provided and the file exists: Just update the timestamp
    if path.exists() {
        let now = FileTime::now();
        if template_name.is_none() {
            return SetFileTimes(path, now, now)
                .map_err(ZapError::SetTimesError);
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
    Ok(())
}

pub fn open_in_editor(filepath: &str) -> Result<(), ZapError> {
    let editor_env_var = env::var("EDITOR").map_err(|_| ZapError::EditorNotSet)?;

    let mut parts = editor_env_var.split_whitespace();
    let editor_executable = parts.next()
        .ok_or_else(|| ZapError::EditorCommandParseError(editor_env_var.clone()))?;

    let mut cmd = Command::new(editor_executable);
    cmd.args(parts);
    cmd.arg(filepath);

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
