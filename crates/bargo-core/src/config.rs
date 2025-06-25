use crate::cli::Cli;

#[derive(Clone, Debug)]
pub struct Config {
    pub verbose: bool,
    pub dry_run: bool,
    pub pkg: Option<String>,
    pub quiet: bool,
}

impl From<&Cli> for Config {
    fn from(cli: &Cli) -> Self {
        Self {
            verbose: cli.verbose,
            dry_run: cli.dry_run,
            pkg: cli.pkg.clone(),
            quiet: cli.quiet,
        }
    }
}
