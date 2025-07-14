use clap::Parser;
use clap::builder::ArgPredicate;

#[derive(Parser, Debug)]
#[clap(name = "zap", author, version, about = "touch, but with templates", long_about = None)]
pub struct ZapCli {
    #[clap(value_parser)]
    pub filenames: Vec<String>,

    /// Optional template name to pre-populate the file.
    /// Templates are sourced from ~/.config/zap/<template_name>.
    #[clap(short = 'T', long, value_name = "TEMPLATE_NAME", verbatim_doc_comment)]
    pub template: Option<String>,

    /// Optional context to use when rendering the template.
    /// should contain key-value pairs in the format `foo=bar,baz=qux`.
    #[clap(short = 'C', long, value_name = "CONTEXT", verbatim_doc_comment)]
    pub context: Option<String>,

    /// Open the file with your $EDITOR
    #[clap(short = 'o', long)]
    pub open: bool,

    /// only update the access time
    #[clap(short = 'a', default_value_t = true)]
    pub access_time: bool,

    /// only update the modification time
    #[clap(short = 'm', default_value_t = true)]
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
        overrides_with = "timestamp",
        verbatim_doc_comment
    )]
    pub date: Option<String>,

    /// pass date as POSIX compliant timestamp: [[CC]YY]MMDDhhmm[.SS]
    #[clap(
        short = 't',
        long,
        value_name = "TIMESTAMP",
        overrides_with = "date",
        verbatim_doc_comment
    )]
    pub timestamp: Option<String>,

    /// Adjust time [-][[hh]mm]SS
    /// the `-c` flag is implied
    #[clap(short = 'A', long, value_name = "ADJUST", verbatim_doc_comment, allow_hyphen_values = true)]
    pub adjust: Option<String>,
}
