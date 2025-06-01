use clap::Parser;
use std::process;
use zap::{zap, open_in_editor, ZapCli};

fn main() {
    let cli = ZapCli::parse();

    match zap(&cli.filenames, cli.template.as_deref(), cli.context.as_deref()) {
        Ok(()) => {
            if cli.open {
                if let Err(e) = open_in_editor(&cli.filenames) {
                    eprintln!("Warning: Could not open editor: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
