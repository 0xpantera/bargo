use color_eyre::Result;

use crate::{commands::common::run_nargo_command, config::Config};

pub fn run(cfg: &Config) -> Result<()> {
    run_nargo_command(cfg, &["check"])
}
