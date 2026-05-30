use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "leuko",
    about = "Friendly white blood cell for your package manager"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // Install {
    //     packages: Vec<String>
    // },
    Audit { packages: Vec<String> },
}
