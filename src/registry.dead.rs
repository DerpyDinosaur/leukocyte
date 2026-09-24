use crate::errors::LeukoError;
use crate::types;
use reqwest::blocking::Client;
use serde::Deserialize;
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Deserialize)]
pub struct NpmRegistryScripts {
    pub preinstall: Option<String>,
    pub postinstall: Option<String>,
    pub install: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NpmRegistryResponse {
    pub name: String,
    pub version: String,
    pub scripts: NpmRegistryScripts,
}

pub struct NpmRegistryDependency {
    pub name: String,
    pub version: String,
    pub is_dev: bool,
    pub is_poisoned: bool,
    pub scripts: NpmRegistryScripts,
}

pub struct NpmRegistry {
    pub name: String,
    pub version: String,
    pub is_poisoned: bool,
    pub meta: NpmRegistryResponse,
    pub version_meta: NpmRegistryResponse,
    pub dependencies: Vec<NpmRegistryDependency>,
    pub dev_dependencies: Vec<NpmRegistryDependency>,
    input_value: String,
    url: String,
    version_url: Option<String>,
}

impl NpmRegistry {
    static RE_SEMVER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$")
            .unwrap()
    });

    pub fn new(input_value: &str) -> Self {
        let (package, version) = Self::parse_package_name_and_version(&input_value);
        if !Self::RE_SEMVER.is_match(&version) {
            Self::fetch_registry_meta(&mut self, true);
        }

        Self {
            name: package,
            version: version,
            is_poisoned: false,
            meta: None,
            version_meta: None,
            dependencies: Vec::new(),
            dev_dependencies: Vec::new(),
            input_value: input_value.to_string(),
            url: String::new(),
            version_url: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn _is_poisoned(&self) -> bool {
        self.is_poisoned
    }

    fn parse_package_name_and_version(package: &str) -> (String, String) {
        /*
           May have to worry about these later
           Permitted Symbols: '-', '_', '.'
               cannot be leading characters unless in scoped variant

           Banned Symbols: '~', '(', ')', '`', '!', '*', '%', '&', '?'
               cannot have uppercase letters
        */
        let default: (String, String) = (package.to_string(), "latest".to_string());

        // Scoped package e.g. @org/package@1.0.0
        if let Some(striped_prefix) = package.strip_prefix('@') {
            return match striped_prefix.split_once('@') {
                Some((name, version)) if !version.is_empty() => {
                    (format!("@{}", name), version.to_string())
                }
                _ => default,
            };
        }

        // Version pinned package e.g. package@1.0.0
        match package.split_once('@') {
            Some((name, version)) if !name.is_empty() && !version.is_empty() => {
                (name.to_string(), version.to_string())
            }
            _ => default,
        }
    }

    pub fn fetch_registry_meta(&mut self, fetch_meta: bool) {
        // NPM Registry
        // Get Specific Version -> https://registry.npmjs.org/<package-name>/<version>
        // Get Package Info -> https://registry.npmjs.org/<package-name>
        let client = Client::new();
        let url: String = match fetch_meta {
            true => format!("https://registry.npmjs.org/{}", self.name),
            false => format!("https://registry.npmjs.org/{}/{}", self.name, self.version),
        };

        let response = client.get(&url).send()?;
        let data = response.text()?;
        let details: types::NpmRegistryResponse = serde_json::from_str(&data)?;
        if fetch_meta {
            self.meta = details;
        } else {
            self.version_meta = details;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod npm_registry {
        use super::*;

        #[test]
        fn new_with_valid_argument() {
            let packages = [
                "vite",
                "vite@2",
                "vite@1.0.0",
                "vite@^1.0.0",
                "vite@>=1.2.3 <2.0.0",
                "vite@latest",
                "@types/node",
                "@types/node@1.0.0",
            ]
            .map(str::to_string)
            .to_vec();

            let expected_results: Vec<(&str, &str)> = vec![
                ("vite", "latest"),
                ("vite", "2"),
                ("vite", "1.0.0"),
                ("vite", "^1.0.0"),
                ("vite", ">=1.2.3 <2.0.0"),
                ("vite", "latest"),
                ("@types/node", "latest"),
                ("@types/node", "1.0.0"),
            ];

            for (i, package) in packages.iter().enumerate() {
                let result = NpmRegistry::new(&package);
                assert_eq!(result.name, expected_results[i].0);
                assert_eq!(result.version, expected_results[i].1);
            }
        }

        #[test]
        fn successful_path() {
            let registry = NpmRegistry::new("vite");
            let result = registry.fetch_registry_meta();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().name, "vite");
        }

        // #[test]
        // fn invalid_package() {
        //     let result = fetch_npm_registry_details("invalid_package_name");
        //     assert!(result.is_err());
        // }

        // #[test]
        // fn empty_package_name() {
        //     let result = fetch_npm_registry_details("");
        //     assert!(result.is_err());
        // }
    }
}
