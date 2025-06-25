use color_eyre::Result;

use crate::config::Config;

/// Build argument list for nargo commands based on global config
pub fn build_nargo_args(cfg: &Config, base_args: &[&str]) -> Result<Vec<String>> {
    let mut args = base_args.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    if let Some(pkg) = &cfg.pkg {
        args.push("--package".to_string());
        args.push(pkg.clone());
    }

    Ok(args)
}
