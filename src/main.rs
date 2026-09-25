use clap::Parser;
use leuko::cli::{AuditCommands, Cli, Commands};
use leuko::errors::{LeukoError, reporter};
use leuko::{commands, shim};

// TODO: Checkout Tabled crate
// TODO: Checkout owo colours crate
// TODO: Seperate code into different package managers

fn main() {
    // Nom all arguments from environment
    let args = std::env::args().collect();
    let whatami = leuko::whatami(&args);

    if !leuko::SUPPORTED_PACKAGE_MANAGERS.contains(&whatami) {
        let err = LeukoError::UnsupportedPackageManager(whatami.to_string());
        reporter::report(&err);
        std::process::exit(reporter::exit_code(&err));
    }

    match whatami {
        "leuko" => run_as_leuko().unwrap_or_else(|err| {
            reporter::report(&err);
            std::process::exit(reporter::exit_code(&err))
        }),
        _ => run_as_shim(&whatami, &args).unwrap_or_else(|err| {
            reporter::report(&err);
            std::process::exit(reporter::exit_code(&err))
        }),
    }
}

fn run_as_shim(package_manager: &str, args: &Vec<String>) -> Result<(), LeukoError> {
    /*
        If node packages fetch details from node registry
        TODO: If python packages fetch details from ???

        Fetching details for packages could be something leuko does via commands too
        Giving the user options, if they do not install the shim for a specific package manager
    */

    // Check if subcommand is a supported add package command
    let subcommand: &str = args.get(1).map(String::as_str).unwrap_or("");
    let is_installing: bool = leuko::SUPPORTED_ADD_PACKAGE_CMDS.contains(&subcommand);

    // Get packages from arguments
    let packages = leuko::extract_packages(args);

    // If you are installing packages, run audit
    if is_installing && !packages.is_empty() {
        let _ = commands::audit::run_pkgs(&packages);
    }

    shim::execute(&package_manager)
}

fn run_as_leuko() -> Result<(), LeukoError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Audit(audit) => commands::audit::run(audit)?,
    }

    Ok(())
}
