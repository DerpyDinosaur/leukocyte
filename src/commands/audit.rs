use crate::cli::{self, AuditArgs, AuditCommands};
use crate::errors::LeukoError;
use crate::registry::npm::get_package_registry_details;
use crate::registry::parse_package_name_and_version;
use crate::registry::types::{MetaVersionResponse, NpmRegistryMeta};
use console::style;
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::thread;
use std::time::Duration;

#[derive(Debug, PartialEq)]
enum Severity {
    Blocked,
    High,
    Medium,
    Low,
}

struct FindingScoreRule {
    min: u8,
    max: u8,
    weight: f32,
    inverse: bool,
}

#[derive(Debug)]
struct Finding {
    label: String,
    message: String,
    severity: Severity,
}

#[derive(Debug)]
struct Audit {
    name: String,
    version: String,
    score: u8,
    findings: Vec<Finding>,
}

fn is_prerelease(version: &str) -> bool {
    version.split('+').next().unwrap_or("").contains('-')
}

fn finding_maintainer() {
    const RULE: FindingScoreRule = FindingScoreRule {
        min: 0,
        max: 1,
        weight: 0.1,
        inverse: true,
    };
}

fn audit(
    meta: &NpmRegistryMeta,
    version: &MetaVersionResponse,
    previous_version: &MetaVersionResponse,
) -> Audit {
    /*
        Audit Ideas
        - Maintainer count & account tenure
        - Publish cadence (a dormant package suddenly publishing is a red flag)
        - Presence of install scripts (preinstall/postinstall)
        - Provenance attestation (npm/Sigstore signing present?)
        - OpenSSF Scorecard rating
        - Dependency count / bus factor
    */
    let mut result: Audit = Audit {
        name: version.name.to_string(),
        version: version.version.to_string(),
        score: 0,
        findings: Vec::new(),
    };

    // Script Finding
    if let Some(scripts) = &version.scripts {
        let first = ["preinstall", "install", "postinstall"]
            .iter()
            .find_map(|&name| scripts.get(name).map(|script| (name, script)));

        if let Some((name, script)) = first {
            result.findings.push(Finding {
                label: "scripts".to_string(),
                message: format!("{name}: {script}"),
                severity: Severity::Blocked,
            });
        }
    }

    result
}

pub fn audit_package_file() -> Result<(), LeukoError> {
    Ok(())
}

/// Final report output for when `audit_packages()` finishes
fn _audit_packages_report() -> Result<(), LeukoError> {
    Ok(())
}

async fn reveal(pb: &ProgressBar) {
    const BAR_WIDTH: usize = 10;
    const FRAMES: u64 = 16;
    const FRAME_DELAY: Duration = Duration::from_millis(30);

    let style = ProgressStyle::with_template(&format!(
        "\t{{msg}}\t{{bar:{BAR_WIDTH}.green/240}} {{pos:>3}}"
    ))
    .expect("reveal template is valid")
    // Hover for tool-tip this takes 3 chars as the following
    // (filled, current, to do)
    .progress_chars("⣿⡇⣿");

    pb.set_length(100);
    pb.set_position(0);
    pb.set_style(style);

    for f in 1..=FRAMES {
        pb.set_position(100 * f / FRAMES);
        tokio::time::sleep(FRAME_DELAY).await;
    }
    // This keeps the current message in progress while waiting for others
    // As opposed to finish_with_message which would complete instantly
    pb.abandon();
}

pub fn audit_packages(packages: &[String]) -> Result<(), LeukoError> {
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
                        &version,
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
                            &version,
                            style("pre-release/build tag not supported, skipping").yellow(),
                            w1 = name_width,
                            w2 = version_width
                        ));
                        return (name, None);
                    }

                    let res = get_package_registry_details(&client, &name, &version).await;
                    // TODO: Implement auditing
                    let package_audit = match &res {
                        Ok((m, v, pv)) => audit(m, v, pv),
                        Err(_) => Audit {
                            name: name.to_string(),
                            version: version.to_string(),
                            score: 0,
                            findings: Vec::new(),
                        },
                    };
                    reveal(&pb).await;

                    // DELETE: This is for viewing animations while designing
                    tokio::time::sleep(Duration::from_secs(1)).await;

                    (name, Some(package_audit))
                }
            })
            .buffer_unordered(16)
            .collect()
            .await
    });

    header.finish_and_clear();
    mp.clear().ok();

    for result in results {
        println!("{:?}", result);
    }

    Ok(())
}

fn sandbox(_packages: &[String]) -> Result<(), LeukoError> {
    Ok(())
}

pub fn run(args: AuditArgs) -> Result<(), LeukoError> {
    match args.command {
        Some(AuditCommands::Pkgs { packages }) => audit_packages(&packages),
        Some(AuditCommands::Test { packages }) => sandbox(&packages),
        None => audit_package_file(),
    }
}

#[cfg(test)]
mod tests {
    use crate::registry::types::MetaScripts;
    use serde::de::IgnoredAny;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn audit_scripts_finding() {
        let name = String::from("test");
        let version_string = String::from("1.0.0");
        let previous_version_string = String::from("0.1.0");
        let mut versions = HashMap::new();
        versions.insert(version_string.clone(), IgnoredAny);
        versions.insert(previous_version_string.clone(), IgnoredAny);
        let mut dist = HashMap::new();
        let mut scripts = HashMap::new();
        scripts.insert(String::from("install"), String::from("node install.js"));

        let meta: NpmRegistryMeta = NpmRegistryMeta {
            name: name.clone(),
            versions,
            dist,
        };
        let version: MetaVersionResponse = MetaVersionResponse {
            name: name.clone(),
            version: version_string.clone(),
            scripts: Some(scripts),
        };
        let previous_version: MetaVersionResponse = MetaVersionResponse {
            name: name.clone(),
            version: previous_version_string.clone(),
            scripts: None,
        };

        let result = audit(&meta, &version, &previous_version);
        let findings = &result.findings;

        assert_eq!(findings.len(), 1);

        let script_finding = &findings[0];
        assert_eq!(script_finding.label, "scripts");
        assert_eq!(script_finding.message, "install: node install.js");
        assert_eq!(script_finding.severity, Severity::Blocked);
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
