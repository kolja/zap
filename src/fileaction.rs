use crate::errors::ZapError;
use crate::file_time_util::{FileTimeSpec, adjust_file_times_from_metadata};
use anyhow::Result;
use dialoguer::Confirm;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum FileOperation {
    Skip {
        reason: String,
    },
    CreateEmpty,
    CreateWithTemplate {
        template_name: String,
        context_str: Option<String>,
    },
    OverwriteWithTemplate {
        template_name: String,
        context_str: Option<String>,
    },
    NoFileOperation, // File exists, no file operations needed
}

#[derive(Debug, Clone)]
pub enum TimeOperation {
    SetTimes {
        times: FileTimeSpec,
    },
    AdjustTimes {
        adjustment_str: String,
        should_update_access: bool,
        should_update_modification: bool,
    },
    NoTimeOperation,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub file_op: FileOperation,
    pub time_ops: Vec<TimeOperation>,
}

pub struct Planner<'a> {
    pub no_create: bool,
    pub adjust: Option<&'a str>,
    pub template: Option<&'a str>,
    pub context: Option<&'a str>,
    pub should_update_access: bool,
    pub should_update_modification: bool,
}

impl<'a> Planner<'a> {
    pub fn plan(&self, path: &Path, file_times: &FileTimeSpec) -> Result<Action, ZapError> {
        let file_exists = path.exists();

        // Determine file operation
        let file_op = if !file_exists && self.no_create {
            FileOperation::Skip {
                reason: "File doesn't exist and --no-create flag is set".to_string(),
            }
        } else if !file_exists && self.template.is_some() {
            FileOperation::CreateWithTemplate {
                template_name: self.template.unwrap().to_string(),
                context_str: self.context.map(|s| s.to_string()),
            }
        } else if !file_exists {
            FileOperation::CreateEmpty
        } else if file_exists && self.template.is_some() {
            FileOperation::OverwriteWithTemplate {
                template_name: self.template.unwrap().to_string(),
                context_str: self.context.map(|s| s.to_string()),
            }
        } else {
            FileOperation::NoFileOperation
        };

        // Determine time operations
        let mut time_ops = Vec::new();

        // If we're skipping the file, don't do any time operations
        if matches!(file_op, FileOperation::Skip { .. }) {
            return Ok(Action { file_op, time_ops });
        }

        // Add time setting operation if we have explicit times or if file operations require it
        match &file_op {
            FileOperation::CreateEmpty
            | FileOperation::CreateWithTemplate { .. }
            | FileOperation::OverwriteWithTemplate { .. } => {
                // File creation/overwrite operations always need time setting
                time_ops.push(TimeOperation::SetTimes { times: *file_times });
            }
            FileOperation::NoFileOperation => {
                // File exists and no file operations - only set times if not adjusting
                if self.adjust.is_none() {
                    time_ops.push(TimeOperation::SetTimes { times: *file_times });
                }
            }
            FileOperation::Skip { .. } => {
                // Already handled above
            }
        }

        // Add adjustment operation if specified
        if let Some(adjustment_str) = self.adjust {
            time_ops.push(TimeOperation::AdjustTimes {
                adjustment_str: adjustment_str.to_string(),
                should_update_access: self.should_update_access,
                should_update_modification: self.should_update_modification,
            });
        }

        // If no time operations were added but we need them (file exists, no adjustment, no explicit times)
        if time_ops.is_empty() && matches!(file_op, FileOperation::NoFileOperation) {
            time_ops.push(TimeOperation::SetTimes { times: *file_times });
        }

        Ok(Action { file_op, time_ops })
    }
}

impl Action {
    pub fn execute(self, path: &Path, filename: &str) -> Result<(), anyhow::Error> {
        // Execute file operation first
        match &self.file_op {
            FileOperation::Skip { reason } => {
                println!("Skipping {filename}: {reason}");
                return Ok(());
            }
            FileOperation::CreateEmpty => {
                Self::ensure_parent_directory_exists(path)?;
                let _file = std::fs::File::create(path)?;
            }
            FileOperation::CreateWithTemplate {
                template_name,
                context_str,
            } => {
                Self::ensure_parent_directory_exists(path)?;
                Self::write_template_to_file(path, template_name, context_str.as_deref())?;
            }
            FileOperation::OverwriteWithTemplate {
                template_name,
                context_str,
            } => {
                let confirmation = Confirm::new()
                    .with_prompt(format!(
                        "File '{filename}' already exists. Do you want to overwrite it?",
                    ))
                    .default(false)
                    .interact()?;

                if confirmation {
                    Self::write_template_to_file(path, template_name, context_str.as_deref())?;
                } else {
                    // User declined overwrite, skip time operations too
                    return Ok(());
                }
            }
            FileOperation::NoFileOperation => {
                // Nothing to do for file
            }
        }

        // Execute time operations in sequence
        for time_op in self.time_ops {
            match time_op {
                TimeOperation::SetTimes { times } => {
                    crate::set_file_times(path, &times)?;
                }
                TimeOperation::AdjustTimes {
                    adjustment_str,
                    should_update_access,
                    should_update_modification,
                } => {
                    let metadata = std::fs::metadata(path)?;
                    let adjusted_times =
                        adjust_file_times_from_metadata(&metadata, &adjustment_str)?
                            .with_flags(should_update_access, should_update_modification);
                    crate::set_file_times(path, &adjusted_times)?;
                }
                TimeOperation::NoTimeOperation => {
                    // Nothing to do
                }
            }
        }

        Ok(())
    }

    fn ensure_parent_directory_exists(path: &Path) -> Result<(), anyhow::Error> {
        if let Some(parent) = path.parent() {
            if parent.components().next().is_some() && !parent.exists() {
                let confirmation = Confirm::new()
                    .with_prompt(format!(
                        "The directory {:?} doesn't exist. Create it?",
                        parent.display()
                    ))
                    .default(false)
                    .interact()?;

                if confirmation {
                    std::fs::create_dir_all(parent)?;
                }
            }
        }
        Ok(())
    }

    fn write_template_to_file(
        path: &Path,
        template_name: &str,
        context_str: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        use crate::{get_config_dir, get_template_path, plugins::Plugins};
        use std::fs::File;
        use std::io::Write;
        use tera::{Context, Tera};

        let template_path_full = get_template_path(template_name)?;
        if !template_path_full.exists() {
            return Err(ZapError::TemplateNotFound(template_path_full).into());
        }

        let mut tera = Tera::default();
        tera.add_template_file(&template_path_full, Some(template_name))?;

        let mut plugins = Plugins::new();
        let plugins_dir = get_config_dir()?.join("plugins");
        plugins.load_plugins_from_dir(&mut tera, &plugins_dir)?;

        let mut context = Context::new();
        if let Some(ctx) = context_str {
            for pair in ctx.split(',') {
                let mut parts = pair.splitn(2, '=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    context.insert(key.trim(), value.trim());
                }
            }
        }
        let rendered = tera.render(template_name, &context)?;

        let mut file = File::create(path)?;
        file.write_all(rendered.as_bytes())?;

        Ok(())
    }
}

pub fn open_in_editor(filepaths: &Vec<String>) -> Result<(), anyhow::Error> {
    use std::env;
    use std::process::Command;

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
                Err(ZapError::EditorExitedWithError(editor_env_var, status.code()).into())
            }
        }
        Err(e) => Err(ZapError::EditorSpawnFailed(editor_env_var, e).into()),
    }
}
