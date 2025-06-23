pub mod check;
pub mod build;
pub mod prove;
pub mod verify;
pub mod verifier;
pub mod clean;
pub mod rebuild;
pub mod doctor;

pub mod cairo {
    pub mod gen_cmd;
    pub mod data;
    pub mod declare;
    pub mod deploy;
    pub mod verify_onchain;
}

pub mod evm {
    pub mod gen_cmd;
    pub mod deploy;
    pub mod calldata;
    pub mod verify_onchain;
}

use crate::cli::Cli;
use color_eyre::Result;

pub(crate) fn build_nargo_args(cli: &Cli, base_args: &[&str]) -> Result<Vec<String>> {
    let mut args = base_args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    if let Some(pkg) = &cli.pkg {
        args.push("--package".to_string());
        args.push(pkg.clone());
    }
    Ok(args)
}
