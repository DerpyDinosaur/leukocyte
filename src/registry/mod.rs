mod npm;
pub use npm::NpmPackage;
mod types;

pub trait Package {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn parse_package_name_and_version(package: &str) -> (String, String) {
        /*
           May have to worry about these later
           Permitted Symbols: '-', '_', '.'
               cannot be leading characters unless in scoped variant

           Banned Symbols: '~', '(', ')', '`', '!', '*', '%', '&', '?'
               cannot have uppercase letters
        */
        let default: (String, String) = (package.to_string(), "latest".to_string());

        // Scoped package e.g. @org/package@1.0.0
        if let Some(striped_prefix) = package.strip_prefix('@') {
            return match striped_prefix.split_once('@') {
                Some((name, version)) if !version.is_empty() => {
                    (format!("@{}", name), version.to_string())
                }
                _ => default,
            };
        }

        // Version pinned package e.g. package@1.0.0
        match package.split_once('@') {
            Some((name, version)) if !name.is_empty() && !version.is_empty() => {
                (name.to_string(), version.to_string())
            }
            _ => default,
        }
    }
}

pub fn encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        // Safe characters per RFC 3986 unreserved set: A-Z, a-z, 0-9, -, _, ., ~
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}
