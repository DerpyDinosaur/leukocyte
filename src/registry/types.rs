use std::collections::HashMap;

use serde::{Deserialize, de::IgnoredAny};

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
    pub scripts: Option<MetaScripts>,
    // pub maintainers: Vec<MetaMaintainers>,
}

// New
#[derive(Debug, Deserialize)]
pub struct NpmRegistryMeta {
    pub name: String,
    pub versions: HashMap<String, IgnoredAny>,
    #[serde(rename = "dist-tags")]
    pub dist: HashMap<String, String>,
}
