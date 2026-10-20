use anyhow::Error;
use owo_colors::OwoColorize;
use reqwest::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use tokio::runtime;

use crate::get_json;
use crate::{BatchConfig, errors::LeukoError};

use super::parse_package_name_and_version;
use super::types::{MetaVersionResponse, NpmRegistryMeta};

pub async fn get_package_registry_details(
    client: &Client,
    name: &str,
    version: &str,
) -> Result<(NpmRegistryMeta, MetaVersionResponse, MetaVersionResponse), LeukoError> {
    // Scoped packages: "@scope/pkg" -> "@scope%2fpkg"
    let encoded = name.replace('/', "%2f");
    let packument_url = format!("https://registry.npmjs.org/{encoded}");
    let version_url = format!("https://registry.npmjs.org/{encoded}/{version}");
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.npm.install-v1+json"),
    );

    // Runs both requests concurrently; returns early on the first error.
    let (meta, ver) = tokio::try_join!(
        get_json::<NpmRegistryMeta>(client.get(&packument_url).headers(headers)),
        get_json::<MetaVersionResponse>(client.get(&version_url)),
    )?;

    let verver = get_json::<MetaVersionResponse>(client.get(&version_url)).await?;

    /*
        Code for getting previous package version
    */
    // let mut package_version_list: Vec<&String> =
    //     meta.versions.keys().filter(|s| !s.contains('-')).collect();
    // package_version_list.sort_by_key(|v| {
    //     v.split('.')
    //         .map(|s| s.parse::<u32>().unwrap_or(0))
    //         .collect::<Vec<u32>>()
    // });

    // let version_index = package_version_list
    //     .iter()
    //     .position(|v| **v == doc.version)
    //     .map(|v| v - 1)
    //     .unwrap_or(package_version_list.len() - 1);

    Ok((meta, ver, verver))
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tokio;

//     #[tokio::test]
//     async fn parse_name_version() {
//         let package = NpmPackage::new("vite@1.0.0");
//         assert_eq!(package.name(), "vite");
//         assert_eq!(package.version(), "1.0.0");
//     }

//     #[tokio::test]
//     async fn successful_get_meta() {
//         let client = Client::new();
//         let package = NpmPackage::new("vite");

//         let package = package
//             .fetch_package_meta(client.clone())
//             .await
//             .expect("fetch_package_meta failed");

//         let package = package
//             .fetch_version_meta(client)
//             .await
//             .expect("fetch_version_meta failed");

//         assert!(package.meta.is_some());
//         assert!(package.version_meta.is_some());
//     }
// }
