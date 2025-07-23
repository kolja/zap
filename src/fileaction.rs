use crate::errors::ZapError;
use crate::file_time_util::{FileTimeSpec, adjust_file_times_from_metadata};
use anyhow::Result;
use dialoguer::Confirm;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum Action {
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
    SetTimes {
        times: FileTimeSpec,
    },
    AdjustTimes {
        adjustment_str: String,
        should_update_access: bool,
        should_update_modification: bool,
    },
}

pub struct Planner<'a> {
    pub no_create: bool,
    pub adjust: Option<&'a str>,
    pub template: Option<&'a str>,
    pub context: Option<&'a str>,
    pub should_update_access: bool,
    pub should_update_modification: bool,
    pub create_intermediate_dirs: bool,
}

impl<'a> Planner<'a> {
    pub fn plan(
        &self,
        path: &Path,
        explicit_times: Option<&FileTimeSpec>,
    ) -> Result<Vec<Action>, ZapError> {
        let file_exists = path.exists();
        let mut actions = Vec::new();

        // Step 1: Handle file operations
        if !file_exists && self.no_create {
            actions.push(Action::Skip {
                reason: "File doesn't exist and --no-create flag is set".to_string(),
            });
            return Ok(actions);
        } else if !file_exists && self.template.is_some() {
            actions.push(Action::CreateWithTemplate {
                template_name: self.template.unwrap().to_string(),
                context_str: self.context.map(|s| s.to_string()),
            });
        } else if !file_exists {
            actions.push(Action::CreateEmpty);
        } else if file_exists && self.template.is_some() {
            actions.push(Action::OverwriteWithTemplate {
                template_name: self.template.unwrap().to_string(),
                context_str: self.context.map(|s| s.to_string()),
            });
        }

        // Step 2: Handle time setting
        match (explicit_times, self.adjust.is_some()) {
            (Some(times), _) => {
                // Explicit times provided - always set them (with flags applied)
                let flagged_times =
                    times.with_flags(self.should_update_access, self.should_update_modification);
                actions.push(Action::SetTimes {
                    times: flagged_times,
                });
            }
            (None, false) => {
                // No explicit times and no adjustment - set to current time (regular touch)
                let current_times = FileTimeSpec::now()
                    .with_flags(self.should_update_access, self.should_update_modification);
                actions.push(Action::SetTimes {
                    times: current_times,
                });
            }
            (None, true) => {
                // No explicit times but adjustment requested - don't set times, just adjust existing
            }
        }

        // Step 3: Handle time adjustment
        if let Some(adjustment_str) = self.adjust {
            actions.push(Action::AdjustTimes {
                adjustment_str: adjustment_str.to_string(),
                should_update_access: self.should_update_access,
                should_update_modification: self.should_update_modification,
            });
        }

        Ok(actions)
    }
}

impl Action {
    pub fn execute(
        self,
        path: &Path,
        filename: &str,
        create_intermediate_dirs: bool,
    ) -> Result<(), anyhow::Error> {
        match self {
            Action::Skip { reason } => {
                println!("Skipping {filename}: {reason}");
            }
            Action::CreateEmpty => {
                Self::ensure_parent_directory_exists(path, create_intermediate_dirs)?;
                let _file = std::fs::File::create(path)?;
            }
            Action::CreateWithTemplate {
                template_name,
                context_str,
            } => {
                Self::ensure_parent_directory_exists(path, create_intermediate_dirs)?;
                Self::write_template_to_file(path, &template_name, context_str.as_deref())?;
            }
            Action::OverwriteWithTemplate {
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
                    Self::write_template_to_file(path, &template_name, context_str.as_deref())?;
                } else {
                    // User declined overwrite - this will interrupt the action sequence
                    return Err(ZapError::UserDeclinedOverwrite.into());
                }
            }
            Action::SetTimes { times } => {
                crate::set_file_times(path, &times)?;
            }
            Action::AdjustTimes {
                adjustment_str,
                should_update_access,
                should_update_modification,
            } => {
                let metadata = std::fs::metadata(path)?;
                let adjusted_times = adjust_file_times_from_metadata(&metadata, &adjustment_str)?
                    .with_flags(should_update_access, should_update_modification);
                crate::set_file_times(path, &adjusted_times)?;
            }
        }
        Ok(())
    }

    fn ensure_parent_directory_exists(
        path: &Path,
        create_intermediate_dirs: bool,
    ) -> Result<(), anyhow::Error> {
        if let Some(parent) = path.parent() {
            if parent.components().next().is_some() && !parent.exists() {
                if create_intermediate_dirs {
                    std::fs::create_dir_all(parent)?;
                } else {
                    let confirmation = Confirm::new()
                        .with_prompt(format!(
                            "The directory {:?} doesn't exist. Create it?",
                            parent.display()
                        ))
                        .default(false)
                        .interact()?;
                    if confirmation {
                        std::fs::create_dir_all(parent)?;
                    } else {
                        return Err(ZapError::UserDeclinedDirCreation.into());
                    }
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

pub fn execute_actions(
    actions: Vec<Action>,
    path: &Path,
    filename: &str,
    create_intermediate_dirs: bool,
) -> Result<(), anyhow::Error> {
    for action in actions {
        action.execute(path, filename, create_intermediate_dirs)?;
    }
    Ok(())
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
