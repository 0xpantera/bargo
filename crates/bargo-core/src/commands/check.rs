use color_eyre::Result;
use tracing::info;

use crate::{backends, commands::build_nargo_args, config::Config};

pub fn run(cfg: &Config) -> Result<()> {
    let args = build_nargo_args(cfg, &["check"])?;
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    if cfg.verbose {
        info!("Running: nargo {}", args.join(" "));
    }

    if cfg.dry_run {
        println!("Would run: nargo {}", args.join(" "));
        return Ok(());
    }

    backends::nargo::run(&arg_refs)
}
