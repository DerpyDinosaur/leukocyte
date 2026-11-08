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

/// Declared script strings that are known to be problematic.
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
    pub scripts: Option<HashMap<String, String>>,
    // pub maintainers: Vec<MetaMaintainers>,
}

// New
#[derive(Debug, Deserialize, Default)]
pub struct NpmRegistryMeta {
    pub name: String,
    pub versions: HashMap<String, IgnoredAny>,
    #[serde(rename = "dist-tags")]
    pub dist: HashMap<String, String>,
    // pub time: HashMap<String, String>, REMOVED DUE TO TIME IT TAKES FOR API TO RESPOND
    pub modified: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct NpmRegistryVersion {
    pub name: String,
    pub version: String,
    pub scripts: Option<HashMap<String, String>>,
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "peerDependencies")]
    pub peer_dependencies: Option<HashMap<String, String>>,
}
