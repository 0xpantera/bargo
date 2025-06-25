use std::sync::Arc;

use crate::cli::Cli;
use crate::runner::{DryRunRunner, RealRunner, Runner};

#[derive(Clone, Debug)]
pub struct Config {
    pub verbose: bool,
    pub dry_run: bool,
    pub pkg: Option<String>,
    pub quiet: bool,
    pub runner: Arc<dyn Runner>,
}

/// Configuration specific to Cairo deploy operations
#[derive(Clone, Debug)]
pub struct CairoDeployConfig {
    pub class_hash: Option<String>,
    pub auto_declare: bool,
    pub no_declare: bool,
}

impl CairoDeployConfig {
    pub fn new(class_hash: Option<String>, auto_declare: bool, no_declare: bool) -> Self {
        Self {
            class_hash,
            auto_declare,
            no_declare,
        }
    }

    /// Returns true if auto-declare should be performed
    pub fn should_auto_declare(&self) -> bool {
        self.auto_declare && !self.no_declare
    }
}

impl From<&Cli> for Config {
    fn from(cli: &Cli) -> Self {
        let runner: Arc<dyn Runner> = if cli.dry_run {
            Arc::new(DryRunRunner::new())
        } else {
            Arc::new(RealRunner::new())
        };

        Self {
            verbose: cli.verbose,
            dry_run: cli.dry_run,
            pkg: cli.pkg.clone(),
            quiet: cli.quiet,
            runner,
        }
    }
}
