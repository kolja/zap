use std::process;

use zap::{args::ZapCli, zap};

fn main() {
    let mut cli = ZapCli::process_h_flag();

    cli.ensure_no_create_if_symlink();

    if let Err(e) = zap(&cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
