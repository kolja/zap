use clap::Parser;
use std::process;
use zap::{zap, ZapCli};

fn main() {
    let cli = ZapCli::parse();

    if let Err(e) = zap(&cli.filename, cli.template.as_deref(), cli.context.as_deref()) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    // println!("zapped {}", cli.filename);
}
