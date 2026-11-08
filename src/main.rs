use clap::Parser;
use leuko::cli::{Cli, Commands};
use leuko::errors::{LeukoError, reporter};
use leuko::{commands, shim};

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
    let raw_args = leuko::parse_args(&args[1..]);
    let is_installing: bool = raw_args
        .verb
        .as_deref()
        .is_some_and(|v| leuko::SUPPORTED_ADD_PACKAGE_CMDS.contains(&v));

    if is_installing && !raw_args.packages.is_empty() {
        let audited_packages = commands::audit::audit_packages(&raw_args.packages)?;
        shim::execute(&package_manager, &raw_args, &audited_packages)
    } else {
        shim::passthrough(&package_manager, &args[1..])
    }
}

fn run_as_leuko() -> Result<(), LeukoError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Audit(audit) => commands::audit::run(audit)?,
    }

    Ok(())
}
