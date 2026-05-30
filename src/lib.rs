use reqwest::blocking::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LeukoError {
    // Generic Errors
    #[error("LeukoError: {0}")]
    ExpectedError(String),
    // shim errors
    #[error("Failed to get leuko's path")]
    ExePathUnavailable,
    #[error("PATH is not defined in the environment")]
    PathEnvMissing,
    #[error("Could not find package manager: {0}")]
    NotFound(String),
    // Unknown Errors
    #[error("Unknown error occured: {0}")]
    UnknownError(String),
}

pub static SUPPORTED_PACKAGE_MANAGERS: [&str; 5] = ["bun", "leuko", "npm", "pnpm", "yarn"];
pub static SUPPORTED_ADD_PACKAGE_CMDS: [&str; 4] = ["install", "i", "add", "a"];

#[derive(Deserialize)]
struct NpmRegistryResponse {
    name: String,
    version: String,
    scripts: NpmRegistryScripts,
}

#[derive(Deserialize)]
struct NpmRegistryScripts {
    preinstall: Option<String>,
    postinstall: Option<String>,
    install: Option<String>,
}

pub fn whatami(args: &Vec<String>) -> &str {
    /*
        What program was invoked and caught by leuko
        Take the first argument which would be the path of the called software
    */

    // Args may have nothing called so we return as leuko by default.
    if args.len() < 1 {
        return "leuko";
    }

    // filen_name() -> Get the file name of the executable
    // and_then() -> Convert to string via a map
    // unwrap_or() -> Default to "leuko" if no file name is found
    return std::path::Path::new(&args[0])
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("leuko");
}

pub fn extract_packages(args: &[String]) -> Vec<String> {
    /*
        Get all packages from arguments
    */
    let mut packages = Vec::new();
    for arg in args {
        if arg.starts_with('-') {
            continue;
        }
        if SUPPORTED_PACKAGE_MANAGERS.contains(&arg.as_str()) {
            continue;
        }
        if SUPPORTED_ADD_PACKAGE_CMDS.contains(&arg.as_str()) {
            continue;
        }
        packages.push(arg.to_string());
    }
    packages
}

pub fn fetch_npm_registry_details(package: &str) -> Result<(), LeukoError> {
    // NPM Registry
    // Get Specific Version -> https://registry.npmjs.org/<package-name>/<version>
    // Get Package Info -> https://registry.npmjs.org/<package-name>

    let client = Client::new();
    let url = format!("https://registry.npmjs.org/{}", package);

    let response = match client.get(&url).send() {
        Ok(res) => res,
        Err(e) => return Err(LeukoError::ExpectedError(e.to_string())),
    };

    let data = match response.text() {
        Ok(text) => text,
        Err(e) => return Err(LeukoError::ExpectedError(e.to_string())),
    };

    let details: NpmRegistryResponse = match serde_json::from_str(&data) {
        Ok(parsed) => parsed,
        Err(e) => return Err(LeukoError::ExpectedError(e.to_string())),
    };

    println!("{} {}", details.name, details.version);
    println!("{:?}", details.scripts.postinstall);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod whatami {
        use super::*;
        #[test]
        fn smoke_test() {
            let args = vec!["/usr/local/bin/leuko".to_string(), "audit".to_string()];
            assert_eq!(whatami(&args), "leuko");
        }

        #[test]
        fn empty_arguments_fallback_to_leuko() {
            let args = vec![];
            assert_eq!(whatami(&args), "leuko");
        }

        #[test]
        fn relative_path() {
            let args = vec!["./relative/path/to/npm".to_string()];
            assert_eq!(whatami(&args), "npm");
        }
    }

    mod extract_packages {
        use super::*;
        #[test]
        fn filters_flags() {
            let args = ["add", "-D", "zod"].map(str::to_string).to_vec();
            assert_eq!(extract_packages(&args), vec!["zod"]);
        }

        #[test]
        fn filters_subcommands() {
            let args = ["add", "zod", "react"].map(str::to_string).to_vec();
            assert_eq!(extract_packages(&args), vec!["zod", "react"]);
        }

        #[test]
        fn empty() {
            let args = [].map(str::to_string).to_vec();
            assert!(extract_packages(&args).is_empty());
        }
    }

    mod fetch {
        use super::*;
        #[test]
        fn smoke_test() {
            let result = fetch_npm_registry_details("zod");
            assert!(result.is_ok());
        }
    }
}
