use crate::errors::LeukoError;
use crate::fetch_many_results;
use crate::registry::parse_package_name_and_version;
use crate::registry::types::MetaVersionResponse;
use tokio::runtime;

pub fn run(packages: &[String]) -> Result<(), LeukoError> {
    let parsed_packages: Vec<(String, String)> = packages
        .iter()
        .map(|name| parse_package_name_and_version(name))
        .collect();

    let urls: Vec<String> = parsed_packages
        .iter()
        .map(|(name, version)| format!("https://registry.npmjs.org/{}/{}", name, version))
        .collect();

    let threaded_rt = runtime::Runtime::new().unwrap();
    let results = threaded_rt.block_on(fetch_many_results::<MetaVersionResponse>(&urls, None));

    for result in results {
        match result {
            Ok(pkg) => println!("{} @ {}\n{:?}", pkg.name, pkg.version, pkg.scripts),
            Err(err) => eprintln!("error: {}\n", err),
        }
    }

    Ok(())
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
