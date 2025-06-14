
use clap::Parser;

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
    #[clap(short = 'a')]
    pub access_time: bool,

    /// only update the modification time
    #[clap(short = 'm')]
    pub modification_time: bool,
}
