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

// What program was invoked and caught by leuko
// Take the first argument which would be the path of the called software
pub fn whatami(args: &Vec<String>) -> &str {
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

pub fn fetch_npm_registry_details(package: &str) -> Result<String, anyhow::Error> {
    // NPM Registry
    // Get Specific Version -> https://registry.npmjs.org/<package-name>/<version>
    // Get Package Info -> https://registry.npmjs.org/<package-name>
    let mut res = reqwest::blocking::get(format!("https://registry.npmjs.org/{}", &package))?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    println!("Body:\n{}", body);

    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whatami_smoke_test() {
        let args = vec!["/usr/local/bin/leuko".to_string(), "audit".to_string()];
        assert_eq!(whatami(&args), "leuko");
    }

    #[test]
    fn whatami_empty_arguments_fallback_to_leuko() {
        let args = vec![];
        assert_eq!(whatami(&args), "leuko");
    }

    #[test]
    fn whatami_relative_path() {
        let args = vec!["./relative/path/to/npm".to_string()];
        assert_eq!(whatami(&args), "npm");
    }

    #[test]
    fn fetch_smoke_test() {
        let result = fetch_npm_registry_details("zod");
        assert!(result.is_ok());
    }
}
