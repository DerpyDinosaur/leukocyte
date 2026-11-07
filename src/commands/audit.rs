use crate::cli::utils::{green, muted};
use crate::cli::{self, AuditArgs, AuditCommands};
use crate::errors::LeukoError;
use crate::registry::npm::get_package_registry_details;
use crate::registry::parse_package_name_and_version;
use crate::registry::types::{MetaVersionResponse, NpmRegistryMeta, NpmRegistryVersion};
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::any::Any;
use std::thread;
use std::time::Duration;

const PROGRESS_BAR_WIDTH: usize = 10;

#[derive(Debug, PartialEq)]
enum Severity {
    Blocked,
    High,
    Medium,
    Low,
}

struct FindingScoreRule {
    value: u8,
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

fn audit(
    meta: &NpmRegistryMeta,
    version: &NpmRegistryVersion,
    previous_version: &NpmRegistryVersion,
) -> Audit {
    /*
        Audit Ideas
        - [ ] Maintainer count & account tenure
        - [ ] Publish cadence (a dormant package suddenly publishing is a red flag)
        - [ ] Provenance attestation (npm/Sigstore signing present?)
        - [ ] OpenSSF Scorecard rating
        - [x] Package Age
        - [x] Dependency count / bus factor
        - [x] Presence of install scripts (preinstall/postinstall)

        Implement
        min-max normalization for score
        https://en.wikipedia.org/wiki/Weighted_arithmetic_mean
    */
    let mut result: Audit = Audit {
        name: version.name.to_string(),
        version: version.version.to_string(),
        score: 100,
        findings: Vec::new(),
    };

    let mut scores: Vec<FindingScoreRule> = Vec::new();

    'scripts_finding: {
        let mut rule = FindingScoreRule {
            value: 0,
            min: 0,
            max: 1,
            weight: 0.7,
            inverse: true,
        };

        let Some(scripts) = &version.scripts else {
            scores.push(rule);
            break 'scripts_finding;
        };

        let first = ["preinstall", "install", "postinstall"]
            .iter()
            .find_map(|&name| scripts.get(name).map(|script| (name, script)));

        let Some((name, script)) = first else {
            scores.push(rule);
            break 'scripts_finding;
        };

        result.findings.push(Finding {
            label: "scripts".to_string(),
            message: format!("{name}: {script}"),
            severity: Severity::Blocked,
        });

        scores.push(rule);
    }

    'dependancies_finding: {
        let mut rule: FindingScoreRule = FindingScoreRule {
            value: 0,
            min: 0,
            max: 1,
            weight: 0.1,
            inverse: true,
        };

        let Some(dependancies) = &version.dependancies else {
            scores.push(rule);
            break 'dependancies_finding;
        };

        let count_of_depends = dependancies.iter().count();

        result.findings.push(Finding {
            label: "dependencies".to_string(),
            message: format!("{}", count_of_depends),
            severity: Severity::Low,
        });

        scores.push(rule);
    }

    'age_finding: {
        let mut rule: FindingScoreRule = FindingScoreRule {
            value: 0,
            min: 0,
            max: 42,
            weight: 0.2,
            inverse: false,
        };
        let Some(date_created) = &meta.time.get(&version.version) else {
            scores.push(rule);
            break 'age_finding;
        };

        let date_created: jiff::Timestamp = match date_created.parse() {
            Ok(d) => d,
            Err(e) => {
                result.findings.push(Finding {
                    label: "age".to_string(),
                    message: "could not find age".to_string(),
                    severity: Severity::Blocked,
                });
                scores.push(rule);
                break 'age_finding;
            }
        };
        // Divide by seconds in the day
        let elapsed_days = jiff::Timestamp::now()
            .duration_since(date_created)
            .as_secs()
            / 86_400;
        if elapsed_days < 3 {
            result.findings.push(Finding {
                label: "age".to_string(),
                message: "version is too new".to_string(),
                severity: Severity::Blocked,
            })
        } else if elapsed_days < 7 {
            result.findings.push(Finding {
                label: "age".to_string(),
                message: "version was released this week".to_string(),
                severity: Severity::High,
            })
        } else if elapsed_days < 42 {
            result.findings.push(Finding {
                label: "age".to_string(),
                message: "version was less than 42 days".to_string(),
                severity: Severity::Medium,
            })
        }

        scores.push(rule);
    }

    #[cfg(test)]
    {
        let total: f32 = scores.iter().map(|s| s.weight).sum();
        assert!(
            total == 1.0,
            "score weights sum to {total}, expected 1.0 ({} rules)",
            scores.len()
        );
    }

    result
}

pub fn audit_package_file() -> Result<(), LeukoError> {
    Ok(())
}

/// Final report output for when `audit_packages()` finishes
fn audit_packages_report(
    audits: Vec<Audit>,
    measurements: (usize, usize),
) -> Result<(), LeukoError> {
    fn progress_bar(score: u8) -> String {
        let filled = ((score as usize) * PROGRESS_BAR_WIDTH + 50) / 100;
        format!(
            "{}{}",
            "⣿".repeat(filled).style(green()),
            "⣿".repeat(PROGRESS_BAR_WIDTH - filled).style(muted())
        )
    }

    println!(
        "{} {}",
        "leuko".bold(),
        format!("audited {} package/s", audits.len()).dimmed()
    );
    println!();
    for audit in audits {
        println!(
            "    {:<w$}\t{} {}  {}",
            format!("{} {}", audit.name, audit.version),
            progress_bar(audit.score),
            format!("{:>3}", audit.score),
            "clean".green(),
            w = measurements.0 + measurements.1 + 1
        );

        let label_width = audit
            .findings
            .iter()
            .map(|p| p.label.chars().count())
            .max()
            .unwrap_or(0);

        let message_width = audit
            .findings
            .iter()
            .map(|p| p.message.chars().count())
            .max()
            .unwrap_or(0);

        for finding in &audit.findings {
            println!(
                "        {:<l$}    {:<m$}",
                finding.label.dimmed(),
                finding.message,
                l = label_width,
                m = message_width
            )
        }
    }
    Ok(())
}

async fn reveal(pb: &ProgressBar, score: u8) {
    const FRAMES: u64 = 16;
    const FRAME_DELAY: Duration = Duration::from_millis(30);

    let style = ProgressStyle::with_template(&format!(
        "\t{{msg}}\t{{bar:{PROGRESS_BAR_WIDTH}.green/240}} {{pos:>3}}"
    ))
    .expect("reveal template is valid")
    // Hover for tool-tip this takes 3 chars as the following
    // (filled, current, to do)
    .progress_chars("⣿⡇⣿");

    pb.set_length(100);
    pb.set_position(0);
    pb.set_style(style);

    for f in 1..=FRAMES {
        pb.set_position(u64::from(score) * f / FRAMES);
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
    header.set_message(format!(
        "{} {}",
        "leuko".bold(),
        format!("auditing {} package/s", packages.len()).dimmed()
    ));
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
                            "pre-release/build tag not supported, skipping".yellow(),
                            w1 = name_width,
                            w2 = version_width
                        ));
                        return (name, None);
                    }

                    let res = get_package_registry_details(&client, &name, &version).await;
                    // TODO: Implement auditing
                    let package_audit = match &res {
                        Ok((m, v, pv)) => audit(m, v, pv),
                        Err(e) => {
                            pb.finish_with_message(format!("{:?}", e));
                            Audit {
                                name: name.to_string(),
                                version: version.to_string(),
                                score: 0,
                                findings: Vec::new(),
                            }
                        }
                    };
                    reveal(&pb, package_audit.score).await;

                    // DELETE: This is for viewing animations while designing
                    tokio::time::sleep(Duration::from_secs(1)).await;

                    (name, Some(package_audit))
                }
            })
            .buffer_unordered(16)
            .collect()
            .await
    });

    // TODO: Need to view what tasks failed and why

    header.finish_and_clear();
    mp.clear().ok();

    let audits: Vec<Audit> = results.into_iter().filter_map(|(_, audit)| audit).collect();
    audit_packages_report(audits, (name_width, version_width));

    // for result in results {
    //     println!("{:?}", result);
    // }

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

    fn fixture_package_details() -> (NpmRegistryMeta, NpmRegistryVersion, NpmRegistryVersion) {
        let mut m = NpmRegistryMeta::default();
        let mut v = NpmRegistryVersion::default();
        let mut p = NpmRegistryVersion::default();
        (m, v, p)
    }

    #[test]
    #[ignore = "Should not be an integration test"]
    fn audit_pkgs_supports_all_install_notations() {
        /*
            npm install                         # Install all deps from package.json (also: npm i)
            npm install <pkg>                   # Install latest version
            npm install <pkg>@1.2.3             # Specific version
            npm install <pkg>@">=1.2.3 <2.0.0"  # Version range
            npm install <pkg>@latest            # Explicitly latest tag
            npm install <pkg>@next              # 'next' dist-tag (beta/RC)
        */
        let packages = [
            "vite",
            "vite@1.0.0",
            "vite@^1.0.0",
            "vite@>=1.2.3 <2.0.0",
            "vite@latest",
            "vite@rc-beta",
            "@types/node",
        ]
        .map(str::to_string)
        .to_vec();
        let result = audit_packages(&packages);
        let is_ok = result.is_ok();
        if is_ok {
            assert!(is_ok);
        } else {
            println!("{:?}", &result.err());
            assert!(!is_ok);
        }
    }

    #[test]
    fn audit_scripts_finding() {
        let name = String::from("test");
        let version_string = String::from("1.0.0");
        let previous_version_string = String::from("0.1.0");

        let mut versions = HashMap::new();
        versions.insert(version_string.clone(), IgnoredAny);
        versions.insert(previous_version_string.clone(), IgnoredAny);

        let mut dist = HashMap::new();

        let mut time = HashMap::new();
        time.insert(
            "modified".to_string(),
            "2000-01-01T00:00:00.000Z".to_string(),
        );
        time.insert(
            version_string.clone(),
            "2000-01-01T00:00:00.000Z".to_string(),
        );

        let mut scripts = HashMap::new();
        scripts.insert(String::from("install"), String::from("node install.js"));

        let meta: NpmRegistryMeta = NpmRegistryMeta {
            name: name.clone(),
            versions,
            dist,
            time,
        };
        let version: NpmRegistryVersion = NpmRegistryVersion {
            name: name.clone(),
            version: version_string.clone(),
            scripts: Some(scripts),
            dependancies: None,
            dev_dependancies: None,
            peer_dependancies: None,
        };
        let previous_version: NpmRegistryVersion = NpmRegistryVersion {
            name: name.clone(),
            version: previous_version_string.clone(),
            scripts: None,
            dependancies: None,
            dev_dependancies: None,
            peer_dependancies: None,
        };

        let result = audit(&meta, &version, &previous_version);
        let findings = &result.findings;

        assert_eq!(findings.len(), 1);

        let script_finding = &findings[0];
        assert_eq!(script_finding.label, "scripts");
        assert_eq!(script_finding.message, "install: node install.js");
        assert_eq!(script_finding.severity, Severity::Blocked);
    }

    #[test]
    fn audit_score_weights_sum_to_one() {
        let (meta, version, previous) = fixture_package_details();
        audit(&meta, &version, &previous);
    }

    #[test]
    fn audit_report_colours() {
        let audits = vec![
            Audit {
                name: "zod".to_string(),
                version: "1.0.0".to_string(),
                score: 50,
                findings: vec![
                    Finding {
                        label: String::from("script"),
                        message: String::from("postinstall: node install.js"),
                        severity: Severity::High,
                    },
                    Finding {
                        label: String::from("age"),
                        message: String::from("package is too young"),
                        severity: Severity::Blocked,
                    },
                ],
            },
            Audit {
                name: "tailwindcss".to_string(),
                version: "1.0.0".to_string(),
                score: 50,
                findings: vec![
                    Finding {
                        label: String::from("script"),
                        message: String::from("postinstall: node install.js"),
                        severity: Severity::High,
                    },
                    Finding {
                        label: String::from("age"),
                        message: String::from("package is too young"),
                        severity: Severity::Blocked,
                    },
                ],
            },
        ];
        audit_packages_report(audits, (11, 5));
    }
}
