use super::types;
use super::{Package, encode};
use crate::errors::LeukoError;
use reqwest::Client;

#[derive(Debug)]
pub struct NpmPackage {
    name: String,
    version: String,
    pub meta: Option<types::MetaResponse>,
    pub version_meta: Option<types::MetaVersionResponse>,
}

impl NpmPackage {
    pub fn new(input_value: &str) -> NpmPackage {
        let (name, version) = Self::parse_package_name_and_version(input_value);

        NpmPackage {
            name,
            version,
            meta: None,
            version_meta: None,
        }
    }
}

impl Package for NpmPackage {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }
}

async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &Client,
    url: &str,
) -> Result<T, LeukoError> {
    let response = client.get(url).send().await?;
    Ok(response.error_for_status()?.json().await?)
}

pub async fn fetch_package_meta(
    name: &str,
    client: &Client,
) -> Result<types::MetaResponse, LeukoError> {
    let url = format!("https://registry.npmjs.org/{}", encode(name));
    let result = Some(fetch_json(client, &url).await?);
    Ok(result)
}

// pub async fn fetch_version_meta(
//     name: &str,
//     version: &str,
//     client: &Client,
// ) -> Result<NpmPackage, LeukoError> {
//     let url = format!("https://registry.npmjs.org/{}/{}", encode(name), version);
//     self.version_meta = Some(fetch_json(client, &url).await?);
//     Ok(self)
// }

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn parse_name_version() {
        let package = NpmPackage::new("vite@1.0.0");
        assert_eq!(package.name(), "vite");
        assert_eq!(package.version(), "1.0.0");
    }

    // #[tokio::test]
    // async fn successful_get_meta() {
    //     let client = Client::new();
    //     let mut package = NpmPackage::new("vite");
    //     package.fetch_package_meta(client).await;
    //     package.fetch_version_meta(client.clone()).await;

    //     assert!(package.meta.is_some());
    //     assert!(package.version_meta.is_some());
    // }
    #[tokio::test]
    async fn successful_get_meta() {
        let client = Client::new();
        let package = NpmPackage::new("vite");

        let package = package
            .fetch_package_meta(client.clone())
            .await
            .expect("fetch_package_meta failed");

        let package = package
            .fetch_version_meta(client)
            .await
            .expect("fetch_version_meta failed");

        assert!(package.meta.is_some());
        assert!(package.version_meta.is_some());
    }
}
