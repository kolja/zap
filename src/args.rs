use clap::builder::ArgPredicate;
use clap::{ArgAction, CommandFactory, Parser};
use std::env;

#[derive(Parser, Debug)]
#[clap(name = "zap", author, version, about = "touch, but with templates", long_about = None, arg_required_else_help(true))]
#[clap(disable_help_flag = true)] // We'll handle the help flag manually
pub struct ZapCli {
    /// Show help information
    #[clap(short = 'h', long = "help", action = ArgAction::Help)]
    pub help: Option<bool>,
    #[clap(value_parser, required = true, num_args = 1..)]
    pub filenames: Vec<String>,

    /// Optional template name to pre-populate the file.
    /// Templates are sourced from ~/.config/zap/<template_name>.
    #[clap(short = 'T', long, value_name = "TEMPLATE_NAME", verbatim_doc_comment)]
    pub template: Option<String>,

    /// Optional context to use when rendering the template.
    /// should contain key-value pairs in the format `foo=bar,baz=qux`.
    #[clap(short = 'C', long, value_name = "CONTEXT", verbatim_doc_comment)]
    pub context: Option<String>,

    /// always create intermediate directories if they do not exist
    /// (analogous to `mkdir -p`)
    #[clap(short = 'p', long, default_value = "false", verbatim_doc_comment)]
    pub create_intermediate_dirs: bool,

    /// Open the file with your $EDITOR
    #[clap(short = 'o', long)]
    pub open: bool,

    /// only update the access time
    #[clap(short = 'a')]
    pub access_time: bool,

    /// only update the modification time
    #[clap(short = 'm')]
    pub modification_time: bool,

    /// Don't create the file if it doesn't exist
    #[clap(
        short = 'c',
        long,
        default_value_if("adjust", ArgPredicate::IsPresent, "true"), // -c implied if -A is used
        default_value_if("symlink_only", ArgPredicate::IsPresent, "true") // -c implied if -h is used
    )]
    pub no_create: bool,

    /// If the file is a symbolic link, change the times of the link itself rather than the file that the link points to
    /// Note that this implies -c and thus will not create any new files
    #[clap(long = "symlink")]
    pub symlink_only: bool,

    /// pass date as human readable string (RFC3339)
    #[clap(
        short = 'd',
        long,
        value_name = "DATE",
        overrides_with_all = ["timestamp", "reference"],
        verbatim_doc_comment
    )]
    pub date: Option<String>,

    /// pass date as POSIX compliant timestamp: [[CC]YY]MMDDhhmm[.SS]
    #[clap(
        short = 't',
        long,
        value_name = "TIMESTAMP",
        overrides_with_all = ["date", "reference"],
        verbatim_doc_comment
    )]
    pub timestamp: Option<String>,

    /// Use access and modification times from the specified file
    #[clap(
        short = 'r',
        long,
        value_name = "REFERENCE",
        overrides_with_all = ["date", "timestamp"],
    )]
    pub reference: Option<String>,
    /// Adjust time [-][[hh]mm]SS
    /// the `-c` flag is implied
    #[clap(
        short = 'A',
        long,
        value_name = "ADJUST",
        verbatim_doc_comment,
        allow_hyphen_values = true
    )]
    pub adjust: Option<String>,
}

impl ZapCli {
    /// Process command line arguments and check for -h being used for symlink.
    /// If "-h" is passed without any other arguments, it's treated as help.
    /// Otherwise, it's treated as the symlink_only flag.
    pub fn process_h_flag() -> Self {
        let args: Vec<String> = env::args().collect();

        // Check if we have only "-h" without other arguments
        if args.len() == 2 && args[1] == "-h" {
            // Show help and exit
            let mut app = Self::command();
            app.print_help().unwrap();
            std::process::exit(0);
        }

        // Look for "-h" and replace it with "--symlink" for clap processing
        let processed_args: Vec<String> = args
            .into_iter()
            .map(|arg| {
                if arg == "-h" {
                    "--symlink".to_string()
                } else {
                    arg
                }
            })
            .collect();

        // Use the processed args
        Self::parse_from(processed_args)
    }

    /// Determine which times should be updated based on the -a and -m flags.
    /// Following touch command behavior:
    /// - If neither -a nor -m or both -a and -m are specified: update both times
    /// - If only either -a or -m are specified: update only the respective times
    /// Convenience method to check if symlink_only is set, and if so, ensure no_create is also set
    pub fn ensure_no_create_if_symlink(&mut self) {
        if self.symlink_only {
            self.no_create = true;
        }
    }

    pub fn should_update_times(&self) -> (bool, bool) {
        match (self.access_time, self.modification_time) {
            (false, false) => (true, true), // Neither specified: update both
            (true, false) => (true, false), // Only -a: update access time only
            (false, true) => (false, true), // Only -m: update modification time only
            (true, true) => (true, true),   // Both specified: update both
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_update_times_default_behavior() {
        // When neither -a nor -m is specified, both should be updated
        let cli = ZapCli {
            help: None,
            filenames: vec!["test.txt".to_string()],
            template: None,
            context: None,
            open: false,
            access_time: false,       // Default when flag not specified
            modification_time: false, // Default when flag not specified
            no_create: false,
            create_intermediate_dirs: false,
            date: None,
            timestamp: None,
            reference: None,
            adjust: None,
            symlink_only: false,
        };

        let (update_access, update_modification) = cli.should_update_times();
        assert!(
            update_access,
            "Should update access time when no flags specified"
        );
        assert!(
            update_modification,
            "Should update modification time when no flags specified"
        );
    }

    #[test]
    fn test_should_update_times_access_only() {
        // When only -a is specified, only access time should be updated
        let cli = ZapCli {
            help: None,
            filenames: vec!["test.txt".to_string()],
            template: None,
            context: None,
            open: false,
            access_time: true,        // -a flag specified
            modification_time: false, // -m flag not specified
            no_create: false,
            create_intermediate_dirs: false,
            date: None,
            timestamp: None,
            reference: None,
            adjust: None,
            symlink_only: false,
        };

        let (update_access, update_modification) = cli.should_update_times();
        assert!(update_access, "Should update access time when -a specified");
        assert!(
            !update_modification,
            "Should NOT update modification time when only -a specified"
        );
    }

    #[test]
    fn test_should_update_times_modification_only() {
        // When only -m is specified, only modification time should be updated
        let cli = ZapCli {
            help: None,
            filenames: vec!["test.txt".to_string()],
            template: None,
            context: None,
            open: false,
            access_time: false,      // -a flag not specified
            modification_time: true, // -m flag specified
            no_create: false,
            create_intermediate_dirs: false,
            date: None,
            timestamp: None,
            reference: None,
            adjust: None,
            symlink_only: false,
        };

        let (update_access, update_modification) = cli.should_update_times();
        assert!(
            !update_access,
            "Should NOT update access time when only -m specified"
        );
        assert!(
            update_modification,
            "Should update modification time when -m specified"
        );
    }

    #[test]
    fn test_should_update_times_both_flags() {
        // When both -a and -m are specified, both times should be updated
        let cli = ZapCli {
            help: None,
            filenames: vec!["test.txt".to_string()],
            template: None,
            context: None,
            open: false,
            access_time: true,       // -a flag specified
            modification_time: true, // -m flag specified
            no_create: false,
            create_intermediate_dirs: false,
            date: None,
            timestamp: None,
            reference: None,
            adjust: None,
            symlink_only: false,
        };

        let (update_access, update_modification) = cli.should_update_times();
        assert!(
            update_access,
            "Should update access time when both -a and -m specified"
        );
        assert!(
            update_modification,
            "Should update modification time when both -a and -m specified"
        );
    }
}
