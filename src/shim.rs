use std::env;
use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
enum LocateError {
    #[error("Failed to get leuko's path")]
    ExePathUnavailable,
    #[error("PATH is not defined in the environment")]
    PathEnvMissing,
    #[error("Could not find package manager: {0}")]
    NotFound(String),
}

pub fn execute(target: &str) {
    let _package_manager_path = locate_package_manager(&target);
}

fn locate_package_manager(target: &str) -> Result<PathBuf, LocateError> {
    // parent() -> return parent folder to ignore later when detected in the path env
    // canonicalize() -> return absolute path, normalized and symbolic links resolved
    let leuko_dir = match env::current_exe() {
        Ok(exe) => exe.parent().and_then(|p| p.canonicalize().ok()),
        Err(_) => return Err(LocateError::ExePathUnavailable),
    };

    let path_env = env::var_os("PATH").ok_or(LocateError::PathEnvMissing)?;

    for dir in env::split_paths(&path_env) {
        // Skip paths that are leuko
        // *leuko is dereferencing the value to make it the same type as dir
        let is_leuko_dir = leuko_dir
            .as_ref()
            .and_then(|leuko| dir.canonicalize().ok().map(|c| c == *leuko))
            .unwrap_or(false);

        if is_leuko_dir {
            continue;
        }

        // Find true path
        let candidate = dir.join(target);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(LocateError::NotFound(target.to_string()))
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
            assert!(
                matches!(result, Err(leuko::LeukoError::NotFound(t)) if t == "this_does_not_exist")
            );
        }
    }

    mod execute {
        use super::*;

        #[test]
        fn support() {
            assert!(execute("npm").is_ok());
            assert!(execute("bun").is_ok());
        }
    }
}
