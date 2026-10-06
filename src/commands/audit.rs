use std::collections::HashMap;

use crate::cli::{AuditArgs, AuditCommands};
use crate::errors::LeukoError;
use crate::registry::parse_package_name_and_version;
use crate::registry::types::MetaVersionResponse;
use crate::{BatchConfig, big_fetch, get_json_many};
use reqwest::header::{ACCEPT, HeaderValue};
use serde::Deserialize;
use serde::de::IgnoredAny;
use tokio::runtime;

pub fn audit_package_file() -> Result<(), LeukoError> {
    Ok(())
}

pub fn audit_packages(packages: &[String]) -> Result<(), LeukoError> {
    let parsed_packages: Vec<(String, String)> = packages
        .iter()
        .map(|name| parse_package_name_and_version(name))
        .collect();

    let urls: Vec<String> = parsed_packages
        .iter()
        .map(|(name, version)| format!("https://registry.npmjs.org/{}/{}", name, version))
        .collect();

    let threaded_rt = runtime::Runtime::new().unwrap();
    let results = threaded_rt.block_on(get_json_many::<MetaVersionResponse>(
        &urls,
        BatchConfig {
            ..BatchConfig::default()
        },
    ));

    for result in results {
        match result {
            Ok(pkg) => println!("{} @ {}\n{:?}", pkg.name, pkg.version, pkg.scripts),
            Err(err) => eprintln!("error: {}\n", err),
        }
    }

    Ok(())
}

fn test_audit_score_card(packages: &[String]) -> Result<(), LeukoError> {
    /*
        Score Card Ideas
        - Maintainer count & account tenure
        - Publish cadence (a dormant package suddenly publishing is a red flag)
        - Presence of install scripts (preinstall/postinstall)
        - Provenance attestation (npm/Sigstore signing present?)
        - OpenSSF Scorecard rating
        - Dependency count / bus factor
    */

    #[derive(Debug, Deserialize)]
    struct NpmMetadata {
        // name: String,
        versions: HashMap<String, IgnoredAny>,
    }

    /*
        If a user is using a pre-release skip the auditing -
        process because that in of itself is a flight risk.
    */

    let targets: Vec<(String, String)> = packages
        .iter()
        .map(|name| parse_package_name_and_version(name))
        .collect();

    let package_version_urls: Vec<String> = targets
        .iter()
        .map(|(name, version)| format!("https://registry.npmjs.org/{}/{}", name, version))
        .collect();

    let packument_urls: Vec<String> = targets
        .iter()
        .map(|(name, _)| format!("https://registry.npmjs.org/{}", name))
        .collect();

    let thread_rt = runtime::Runtime::new().unwrap();
    let client = reqwest::Client::new();

    let default_cfg = BatchConfig::default();
    let abbreviate_response_cfg = {
        let mut cfg = BatchConfig::default();
        cfg.headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.npm.install-v1+json"),
        );
        cfg
    };

    let (pkg_version, pkg_meta) = thread_rt.block_on(async {
        tokio::join!(
            big_fetch::<MetaVersionResponse>(&client, &package_version_urls, &default_cfg),
            big_fetch::<NpmMetadata>(&client, &packument_urls, &abbreviate_response_cfg)
        )
    });

    for ((url, version_res), (_, meta_res)) in pkg_version.iter().zip(&pkg_meta) {
        let (doc, meta) = match (version_res, meta_res) {
            (Ok(d), Ok(m)) => (d, m),
            (Err(e), _) | (_, Err(e)) => {
                eprintln!("error: {url}: {e}");
                continue;
            }
        };

        let mut package_version_list: Vec<&String> =
            meta.versions.keys().filter(|s| !s.contains('-')).collect();
        package_version_list.sort_by_key(|&a| std::cmp::Reverse(a));

        let version_index = package_version_list
            .iter()
            .position(|v| **v == doc.version)
            .map(|idx| idx + 1)
            .unwrap_or(package_version_list.len() - 1);

        println!("{}@{}", doc.name, doc.version);
        println!("Previous Version: {}", package_version_list[version_index]);
    }

    Ok(())
}

pub fn run(args: AuditArgs) -> Result<(), LeukoError> {
    match args.command {
        Some(AuditCommands::Pkgs { packages }) => audit_packages(&packages),
        Some(AuditCommands::Test { packages }) => test_audit_score_card(&packages),
        None => audit_package_file(),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn supports_all_package_install_syntax() {
//         /*
//             npm install                         # Install all deps from package.json (also: npm i)
//             npm install <pkg>                   # Install latest version
//             npm install <pkg>@1.2.3             # Specific version
//             npm install <pkg>@">=1.2.3 <2.0.0"  # Version range
//             npm install <pkg>@latest            # Explicitly latest tag
//             npm install <pkg>@next              # 'next' dist-tag (beta/RC)
//         */
//         let packages = [
//             "vite",
//             "vite@1.0.0",
//             "vite@^1.0.0",
//             "vite@>=1.2.3 <2.0.0",
//             "vite@latest",
//             "vite@next",
//             "@types/node",
//         ]
//         .map(str::to_string)
//         .to_vec();
//         let result = run(&packages);
//         let is_ok = result.await.is_ok();
//         if is_ok {
//             assert!(is_ok);
//         } else {
//             // println!("{:?}", &result.err());
//             assert!(!is_ok);
//         }
//     }
// }
