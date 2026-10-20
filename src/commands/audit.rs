use crate::cli::{self, AuditArgs, AuditCommands, utils::muted};
use crate::errors::LeukoError;
use crate::registry::npm::get_package_registry_details;
use crate::registry::parse_package_name_and_version;
use crate::registry::types::{MetaVersionResponse, NpmRegistryMeta};
use crate::{BatchConfig, get_json_many};
use console::style;
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use reqwest::Request;
use reqwest::header::{ACCEPT, HeaderValue};
use std::fmt::format;
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime;

fn is_prerelease(version: &str) -> bool {
    version.split('+').next().unwrap_or("").contains('-')
}

pub fn audit_package_file() -> Result<(), LeukoError> {
    Ok(())
}

pub fn audit_packages(packages: &[String]) -> Result<(), LeukoError> {
    /*
        Score Card Ideas
        - Maintainer count & account tenure
        - Publish cadence (a dormant package suddenly publishing is a red flag)
        - Presence of install scripts (preinstall/postinstall)
        - Provenance attestation (npm/Sigstore signing present?)
        - OpenSSF Scorecard rating
        - Dependency count / bus factor

        ~ Performance time for running all packages togeather ~
            - vite react drizzle-orm zod | Time average: 1.2s
            - lodash typescript prettier eslint | Time average: 1.9s

        ~ Performance time for running packages in paralelle ~
            - vite react drizzle-orm zod | Time average: 4s
            - lodash typescript prettier eslint | Time average: 4s
    */

    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = reqwest::Client::new();
    let mp = MultiProgress::new();

    let header = mp.add(ProgressBar::new_spinner());
    header.set_style(
        ProgressStyle::with_template("{spinner} {msg}\n\n")
            .unwrap()
            .tick_strings(&cli::ui::SPINNER_CELL_SMALL),
    );
    header.set_message(format!("leuko is auditing {} packages", packages.len()));
    header.enable_steady_tick(Duration::from_millis(100));

    let plain_style = ProgressStyle::with_template("\t{msg}").unwrap();

    let targets: Vec<(String, String)> = packages
        .iter()
        .map(|name| parse_package_name_and_version(name))
        .collect();

    let name_width = targets
        .iter()
        .map(|p| p.0.chars().count())
        .max()
        .unwrap_or(0);
    let version_width = targets
        .iter()
        .map(|p| p.1.chars().count())
        .max()
        .unwrap_or(0);

    let start = Instant::now();
    let pkg_spinner_style = ProgressStyle::with_template(&format!("\t{{msg}}\t{{spinner:.cyan}}"))
        .unwrap()
        .tick_strings(&cli::ui::PROGRESS_DNA);

    let results: Vec<_> = rt.block_on(async {
        stream::iter(targets)
            .map(|(name, version)| {
                let client = client.clone();
                let mp = mp.clone();
                let spinner_style = pkg_spinner_style.clone();
                let plain_style = plain_style.clone();
                async move {
                    let pb = mp.add(ProgressBar::new_spinner());
                    pb.set_tab_width(4);
                    pb.set_style(spinner_style);
                    pb.set_message(format!(
                        "{:<w1$} {:<w2$}",
                        name,
                        version,
                        w1 = name_width,
                        w2 = version_width
                    ));
                    pb.enable_steady_tick(Duration::from_millis(80));

                    // TODO: Add support for pre-release tags
                    if is_prerelease(&version) {
                        pb.set_style(plain_style);
                        pb.finish_with_message(format!(
                            "{:<w1$} {:<w2$}\t{}",
                            name,
                            version,
                            style("pre-release/build tag not supported, skipping").yellow(),
                            w1 = name_width,
                            w2 = version_width
                        ));
                        return (name, None);
                    }

                    let res = get_package_registry_details(&client, &name, &version).await;

                    let duration = Duration::from_secs(2);
                    thread::sleep(duration);

                    (name, Some(res))
                }
            })
            .buffer_unordered(16)
            .collect()
            .await
    });

    header.finish_and_clear();
    mp.clear().ok();

    for result in results {
        match result {
            (name, Some(Ok(r))) => println!("{} Ok", name),
            (name, None) => println!("{} No result", name),
            (_, Some(Err(e))) => eprintln!("{} Error", e),
        }
    }

    Ok(())
}

fn sandbox(packages: &[String]) -> Result<(), LeukoError> {
    Ok(())
}

pub fn run(args: AuditArgs) -> Result<(), LeukoError> {
    match args.command {
        Some(AuditCommands::Pkgs { packages }) => audit_packages(&packages),
        Some(AuditCommands::Test { packages }) => sandbox(&packages),
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
