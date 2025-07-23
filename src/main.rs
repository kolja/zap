use std::process;

use zap::{args::ZapCli, zap};

fn main() {
    // Use our custom parsing that handles the -h flag
    let mut cli = ZapCli::process_h_flag();

    // Ensure no_create is set if symlink_only is set
    cli.ensure_no_create_if_symlink();

    if let Err(e) = zap(&cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
