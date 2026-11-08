use reqwest::Client;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};

use crate::errors::LeukoError;
use crate::get_json;

use super::types::{NpmRegistryMeta, NpmRegistryVersion};

// References:
// ----- NPM Metadata Registry ---------------------
// https://github.com/npm/registry/blob/main/docs/responses/package-metadata.md

pub async fn get_package_registry_details(
    client: &Client,
    name: &str,
    version: &str,
) -> Result<(NpmRegistryMeta, NpmRegistryVersion, NpmRegistryVersion), LeukoError> {
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
    let (meta_res, version_res) = tokio::try_join!(
        get_json::<NpmRegistryMeta>(client.get(&packument_url).headers(headers)),
        get_json::<NpmRegistryVersion>(client.get(&version_url)),
    )?;

    let mut package_version_list: Vec<&String> = meta_res
        .versions
        .keys()
        .filter(|v| !v.contains('-') || !v.contains('+'))
        .collect();
    package_version_list.sort_by_key(|v| {
        v.split('.')
            .map(|s| s.parse::<u32>().unwrap_or(0))
            .collect::<Vec<u32>>()
    });

    let previous_version_index = package_version_list
        .iter()
        .position(|v| **v == version_res.version)
        .map(|v| v - 1)
        .unwrap_or(package_version_list.len() - 1);

    let previous_version_url = format!(
        "https://registry.npmjs.org/{encoded}/{}",
        package_version_list[previous_version_index]
    );

    let previous_version_res =
        get_json::<NpmRegistryVersion>(client.get(&previous_version_url)).await?;

    Ok((meta_res, version_res, previous_version_res))
}
