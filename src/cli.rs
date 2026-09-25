use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "leuko",
    about = "Friendly white blood cell for your package manager",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run an audit
    Audit(AuditArgs),
}

#[derive(Args)]
pub struct AuditArgs {
    #[command(subcommand)]
    pub command: Option<AuditCommands>,
}

#[derive(Subcommand)]
pub enum AuditCommands {
    /// Audit specific packages
    Pkgs {
        /// Package names to audit
        #[arg(required = true, num_args = 1..)]
        packages: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
