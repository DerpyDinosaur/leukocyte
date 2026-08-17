use serde::Deserialize;

// Global

#[derive(Debug, Deserialize)]
pub struct MetaMaintainers {
    pub name: String,
    pub email: String,
}

// Meta Response

#[derive(Debug, Deserialize)]
pub struct MetaResponse {
    pub name: String,
    pub maintainers: Vec<MetaMaintainers>,
}

// Meta Versioned Response

#[derive(Debug, Deserialize)]
pub struct MetaScripts {
    pub preinstall: Option<String>,
    pub postinstall: Option<String>,
    pub install: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaVersionResponse {
    pub name: String,
    pub version: String,
    pub scripts: MetaScripts,
    pub maintainers: Vec<MetaMaintainers>,
}

// TRASH

pub struct MetaDependency {
    pub name: String,
    pub version: String,
    pub is_dev: bool,
    pub is_poisoned: bool,
    pub scripts: MetaScripts,
}

pub struct NpmRegistry {
    pub name: String,
    pub version: String,
    pub is_poisoned: bool,
    pub meta: MetaResponse,
    pub version_meta: MetaVersionResponse,
    pub dependencies: Vec<MetaDependency>,
    pub dev_dependencies: Vec<MetaDependency>,
    input_value: String,
    url: String,
    version_url: Option<String>,
}
