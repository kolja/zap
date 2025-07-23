use clap::Parser;
use clap::builder::ArgPredicate;

#[derive(Parser, Debug)]
#[clap(name = "zap", author, version, about = "touch, but with templates", long_about = None, arg_required_else_help(true))]
pub struct ZapCli {
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
        default_value_if("adjust", ArgPredicate::IsPresent, "true") // -c implied if -A is used
    )]
    pub no_create: bool,

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
    /// Determine which times should be updated based on the -a and -m flags.
    /// Following touch command behavior:
    /// - If neither -a nor -m or both -a and -m are specified: update both times
    /// - If only either -a or -m are specified: update only the respective times
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
