use crate::errors::LeukoError;
use crate::types::ParsedArgs;
use std::path::Path;
use std::path::PathBuf;
use std::{env, io::Write, process::Command};

fn add_verb(target: &str) -> Result<&'static str, LeukoError> {
    match target {
        "npm" => Ok("install"),
        "pnpm" | "yarn" | "bun" => Ok("add"),
        _ => Err(LeukoError::UnsupportedPackageManager(target.to_string())),
    }
}

pub fn build_args(
    target: &str,
    flags: &[String],
    packages: &[String],
) -> Result<Vec<String>, LeukoError> {
    let mut args = Vec::with_capacity(flags.len() + packages.len() + 3);
    args.push(add_verb(target)?.to_string());
    args.extend(flags.iter().cloned());
    // Prevent any lifetime execution missed
    args.push("--ignore-scripts".into());
    args.push("--".into());
    args.extend(packages.iter().cloned());
    Ok(args)
}

pub fn passthrough(target: &str, raw_args: &[String]) -> Result<(), LeukoError> {
    let package_manager_path = locate_package_manager(target)?;
    launch(&package_manager_path, raw_args)
}

pub fn execute(target: &str, raw_args: &ParsedArgs, packages: &[String]) -> Result<(), LeukoError> {
    let package_manager_path = locate_package_manager(&target)?;
    let args = build_args(target, &raw_args.flags, packages)?;
    launch(&package_manager_path, &args)
}

fn locate_package_manager(target: &str) -> Result<PathBuf, LeukoError> {
    // parent() -> return parent folder to ignore later when detected in the path env
    // canonicalize() -> return absolute path, normalized and symbolic links resolved
    let leuko_dir = env::current_exe()
        .and_then(|p| p.canonicalize())
        .map_err(|_| LeukoError::ExePathUnavailable)?;

    let path_env = env::var_os("PATH").ok_or(LeukoError::PathEnvMissing)?;

    for dir in env::split_paths(&path_env) {
        if !dir.is_absolute() {
            continue;
        }

        for candidate in candidates(&dir, target) {
            if !is_executable(&candidate) {
                continue;
            }
            // Skip leuko and leuko symlinks
            match candidate.canonicalize() {
                Ok(real) if real == leuko_dir => continue,
                Ok(_) => return Ok(candidate),
                Err(_) => continue,
            }
        }
    }

    Err(LeukoError::NotFound(target.to_string()))
}

fn launch(real: &Path, args: &[String]) -> Result<(), LeukoError> {
    let mut cmd = Command::new(real);
    cmd.args(args);

    // exec() skips destructors and buffer flushing, so flush first.
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // exec only returns on failure, and it replaces this process on success,
        // so signals, TTY and exit code all belong to npm.
        Err(LeukoError::LaunchFailed(cmd.exec()))
    }

    #[cfg(not(unix))]
    {
        let status = cmd.status().map_err(LeukoError::LaunchFailed)?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

#[cfg(unix)]
fn candidates(dir: &Path, name: &str) -> Vec<PathBuf> {
    vec![dir.join(name)]
}

#[cfg(windows)]
fn candidates(dir: &Path, name: &str) -> Vec<PathBuf> {
    let exts = env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT".into());
    exts.split(';')
        .filter(|e| !e.is_empty())
        .map(|e| dir.join(format!("{name}{e}")))
        .collect()
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata() // follows symlinks
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(p: &Path) -> bool {
    p.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod locate_package_manager {
        use super::*;

        #[test]
        fn smoke_test() {
            let result = locate_package_manager("mkdir");
            assert!(matches!(result, Ok(path) if path.ends_with("mkdir")));
        }

        #[test]
        fn not_found() {
            let result = locate_package_manager("this_does_not_exist");
            assert!(matches!(result, Err(LeukoError::NotFound(t)) if t == "this_does_not_exist"));
        }
    }

    mod execute {
        use super::*;

        // #[test]
        // fn support() {
        //     assert!(execute("npm", &vec!["vite".to_string()]).is_ok());
        //     assert!(execute("bun", &vec!["vite".to_string()]).is_ok());
        // }
    }
}
