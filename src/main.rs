use clap::Parser;

mod cli;
mod shim;

static _PACKAGE_MANAGER: [&str; 4] = ["bun", "npm", "yarn", "pnpm"];

fn main() {
    // Nom all arguments from environment
    let args = std::env::args().collect();
    let whatami = leuko::whatami(&args);

    match whatami {
        "leuko" => run_as_leuko(&args),
        _ => run_as_shim(&whatami, &args),
    }
}

fn run_as_shim(package_manager: &str, args: &Vec<String>) {
    let _subcommand = args.get(1).map(String::as_str).unwrap_or("");

    // Detect if the user is installing packages

    // If node packages fetch details from node registry
    // TODO: If python packages fetch details from ???

    // Fetching details for packages could be something leuko does via commands too
    // Giving the user options, if they do not install the shim for a specific package manager

    match package_manager {
        "npm" => shim::execute("npm"),
        "bun" => shim::execute("bun"),
        _ => panic!("Unsupported package manager!")
    }
}

fn run_as_leuko(_args: &Vec<String>) {
    let _cli = cli::Cli::parse();
    println!("Leuko: Hi im Leuko!");

    // match cli.command {
    //     cli::Commands::Install { packages, npm_flags } => {
    //         let mut all_args = packages.clone();
    //         all_args.extend(npm_flags);
    //         commands::install::run(&all_args);
    //     }
    //     cli::Commands::Audit { packages } => {
    //         commands::audit::run(&packages);
    //     }
    // }
}
