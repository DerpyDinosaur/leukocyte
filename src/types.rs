use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NpmRegistryResponse {
    pub name: String,
    pub version: String,
    pub scripts: NpmRegistryScripts,
}

#[derive(Debug, Deserialize)]
pub struct NpmRegistryScripts {
    pub preinstall: Option<String>,
    pub postinstall: Option<String>,
    pub install: Option<String>,
}
