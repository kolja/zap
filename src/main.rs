use clap::Parser;
use std::process;

use zap::{args::ZapCli, zap};

fn main() {
    let cli = ZapCli::parse();

    if let Err(e) = zap(&cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
