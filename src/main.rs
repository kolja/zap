use clap::Parser;
use std::process;
use zap::{zap, open_in_editor, ZapCli};

fn main() {
    let cli = ZapCli::parse();

    match zap(&cli.filename, cli.template.as_deref(), cli.context.as_deref()) {
        Ok(()) => {
            if cli.open {
                println!("about to open {}", &cli.filename);
                if let Err(e) = open_in_editor(&cli.filename) {
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
