use crate::errors::LeukoError;
use crate::fetch_npm_registry_details;

pub fn run(packages: &[String]) -> Result<(), LeukoError> {
    for package in packages {
        let _details = fetch_npm_registry_details(package)?;
        println!("{} {}", _details.name, _details.version)
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_all_package_install_syntax() {
        let packages = ["zod", "zod@latest", "zod@^1.0.0"]
            .map(str::to_string)
            .to_vec();
        let result = run(&packages);
        assert!(result.is_ok());
    }
}
