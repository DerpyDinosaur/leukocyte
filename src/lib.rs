use futures::stream::{self, StreamExt};
use reqwest::{Client, RequestBuilder, header::HeaderMap};
use serde::de::DeserializeOwned;

pub mod cli;
pub mod commands;
pub mod errors;
pub mod registry;
pub mod shim;
pub mod types;

pub static SUPPORTED_PACKAGE_MANAGERS: [&str; 5] = ["bun", "leuko", "npm", "pnpm", "yarn"];
pub static SUPPORTED_ADD_PACKAGE_CMDS: [&str; 4] = ["install", "i", "add", "a"];

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub params: Vec<(String, String)>,
    pub headers: HeaderMap,
    pub concurrency: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            params: Vec::new(),
            headers: HeaderMap::new(),
            concurrency: 15,
        }
    }
}

pub fn whatami(args: &Vec<String>) -> &str {
    /*
        What program was invoked and caught by leuko
        Take the first argument which would be the path of the called software
    */
    const DEFAULT: &str = "leuko";

    // Args may have nothing called so we return as leuko by default.
    if args.len() < 1 {
        return DEFAULT;
    }

    // filen_name() -> Get the file name of the executable
    // and_then() -> Convert to string via a map
    let name = std::path::Path::new(&args[0])
        .file_name()
        .and_then(|n| n.to_str());

    return match name {
        Some(value) => value,
        None => DEFAULT,
    };
}

pub fn extract_packages(args: &[String]) -> Vec<String> {
    /*
        Get all packages from arguments
    */
    let mut packages = Vec::new();
    for arg in args {
        if arg.starts_with('-') {
            continue;
        }
        if SUPPORTED_PACKAGE_MANAGERS.contains(&arg.as_str()) {
            continue;
        }
        if SUPPORTED_ADD_PACKAGE_CMDS.contains(&arg.as_str()) {
            continue;
        }
        packages.push(arg.to_string());
    }
    packages
}

/// Get single json request and parse into struct with serde, this is to provide flexibility
/// in my request patterns for asynchronous small batches or to use it within get_json_many()
/// and fetch loads while having flexibility on my options for params and headers.
///
/// Examples:
/// ```ignore
/// // Empty query params and headers
/// let a = get_json<DummyJSON>(client.get(url)).await?;
/// // Batched
/// let results = stream::iter(&urls)
///    .map(|u| get_json<DummyJSON>(client.get(u).query(&params)))
///    .buffer_unordered(8)
///    .collect()
///    .await;
///```
/// TODO: Implement a way to do the same flow for `get_json()` but with `get_json_many()`
/// this way I can deprecate `BatchConfig` struct
pub async fn get_json<T: DeserializeOwned>(req: RequestBuilder) -> Result<T, reqwest::Error> {
    req.send().await?.error_for_status()?.json().await
}

pub async fn get_json_many<T>(
    client: &Client,
    urls: &[String],
    opts: BatchConfig,
) -> Vec<Result<T, reqwest::Error>>
where
    T: DeserializeOwned,
{
    let BatchConfig {
        params,
        headers,
        concurrency,
    } = opts;

    stream::iter(urls)
        .map(|url| get_json::<T>(client.get(url).query(&params).headers(headers.clone())))
        .buffered(concurrency.max(1))
        .collect()
        .await
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

#[cfg(test)]
mod tests {
    use super::*;

    mod whatami {
        use super::*;
        #[test]
        fn smoke_test() {
            let args = vec!["/usr/local/bin/leuko".to_string(), "audit".to_string()];
            assert_eq!(whatami(&args), "leuko");
        }

        #[test]
        fn empty_arguments_fallback_to_leuko() {
            let args = vec![];
            assert_eq!(whatami(&args), "leuko");
        }

        #[test]
        fn relative_path() {
            let args = vec!["./relative/path/to/npm".to_string()];
            assert_eq!(whatami(&args), "npm");
        }
    }

    mod extract_packages {
        use super::*;
        #[test]
        fn filters_flags() {
            let args = ["add", "-D", "zod"].map(str::to_string).to_vec();
            assert_eq!(extract_packages(&args), vec!["zod"]);
        }

        #[test]
        fn filters_subcommands() {
            let args = ["add", "zod", "react"].map(str::to_string).to_vec();
            assert_eq!(extract_packages(&args), vec!["zod", "react"]);
        }

        #[test]
        fn empty() {
            let args = [].map(str::to_string).to_vec();
            assert!(extract_packages(&args).is_empty());
        }
    }
}
