use std::process;
use clap::Parser;

use zap::{
    args::ZapCli,
    zap,
    open_in_editor,
};

fn main() {
    let cli = ZapCli::parse();

    match zap(&cli) {
        Ok(()) => {
            if cli.open {
                if let Err(e) = open_in_editor(&cli.filenames) {
                    eprintln!("Warning: Could not open editor: {e}");
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
