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
    AdjustTimesOnly {
        times: FileTimeSpec,
    },
    UpdateTimesOnly {
        times: FileTimeSpec,
    },
    CreateWithTemplate {
        times: FileTimeSpec,
        template_name: String,
        context_str: Option<String>,
    },
    CreateEmpty {
        times: FileTimeSpec,
    },
    OverwriteWithTemplate {
        times: FileTimeSpec,
        template_name: String,
        context_str: Option<String>,
    },
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

        // Priority 1: If file exists and we have an adjustment, only adjust times
        if file_exists && self.adjust.is_some() {
            let metadata = std::fs::metadata(path)?;
            let adjustment_str = self.adjust.unwrap();
            let adjusted_times = adjust_file_times_from_metadata(&metadata, adjustment_str)?
                .with_flags(self.should_update_access, self.should_update_modification);
            return Ok(Action::AdjustTimesOnly {
                times: adjusted_times,
            });
        }

        // Priority 2: If file doesn't exist and no_create is true, skip
        if !file_exists && self.no_create {
            return Ok(Action::Skip {
                reason: "File doesn't exist and --no-create flag is set".to_string(),
            });
        }

        // Priority 3: If file exists and we have a template, we need to overwrite
        if file_exists && self.template.is_some() {
            return Ok(Action::OverwriteWithTemplate {
                times: file_times.clone(),
                template_name: self.template.unwrap().to_string(),
                context_str: self.context.map(|s| s.to_string()),
            });
        }

        // Priority 4: If file exists and no template, just update times
        if file_exists && self.template.is_none() {
            return Ok(Action::UpdateTimesOnly {
                times: file_times.clone(),
            });
        }

        // Priority 5: If file doesn't exist and we have a template, create with template
        if !file_exists && self.template.is_some() {
            return Ok(Action::CreateWithTemplate {
                times: file_times.clone(),
                template_name: self.template.unwrap().to_string(),
                context_str: self.context.map(|s| s.to_string()),
            });
        }

        // Priority 6: Default case - create empty file
        Ok(Action::CreateEmpty {
            times: file_times.clone(),
        })
    }
}

impl Action {
    pub fn execute(self, path: &Path, filename: &str) -> Result<(), anyhow::Error> {
        match self {
            Action::Skip { reason } => {
                // Could add logging here if needed
                println!("Skipping {}: {}", filename, reason);
                Ok(())
            }

            Action::AdjustTimesOnly { times } => {
                crate::set_file_times(path, &times)?;
                Ok(())
            }

            Action::UpdateTimesOnly { times } => {
                crate::set_file_times(path, &times)?;
                Ok(())
            }

            Action::CreateEmpty { times } => {
                Self::ensure_parent_directory_exists(path)?;
                let _file = std::fs::File::create(path)?;
                crate::set_file_times(path, &times)?;
                Ok(())
            }

            Action::CreateWithTemplate {
                times,
                template_name,
                context_str,
            } => {
                Self::ensure_parent_directory_exists(path)?;
                Self::create_file_with_template(
                    path,
                    &times,
                    &template_name,
                    context_str.as_deref(),
                )?;
                Ok(())
            }

            Action::OverwriteWithTemplate {
                times,
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
                    Self::create_file_with_template(
                        path,
                        &times,
                        &template_name,
                        context_str.as_deref(),
                    )?;
                }
                Ok(())
            }
        }
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

    fn create_file_with_template(
        path: &Path,
        times: &FileTimeSpec,
        template_name: &str,
        context_str: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        use crate::{get_config_dir, get_template_path, plugins::Plugins};
        use std::fs::File;
        use std::io::Write;
        use tera::{Context, Tera};

        let mut file = File::create(path)?;
        crate::set_file_times(path, times)?;

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
        file.write_all(rendered.as_bytes())?;

        Ok(())
    }
}
