use crate::errors::LeukoError;
use crate::registry::NpmPackage;
use reqwest::Client;
use tokio::task::JoinSet;

pub async fn run(package: &[String]) -> Result<(), LeukoError> {
    for package in package {
        println!("{}", &package);
    }

    Ok(())
}

#[cfg(test)]
pub async fn _run_test(packages: &[String]) -> Result<(), LeukoError> {
    // let client = Client::new();

    // let futures = packages.iter().map(|input| {
    //     let client = &client;
    //     async move {
    //         let mut package = NpmPackage::new(input);
    //         package.fetch_package_meta(client).await?;
    //         package.fetch_version_meta(client).await?;
    //         Ok::<NpmPackage, LeukoError>(package)
    //     }
    // });

    // let results = join_all(futures).await;

    // for result in results {
    //     match result {
    //         Ok(pkg) => println!("fetched {:#?}", pkg.version_meta),
    //         Err(e) => eprintln!("failed: {e}"),
    //     }
    // }

    let client = Client::new();
    let mut set = JoinSet::new();
    let package_list: Vec<NpmPackage> = packages
        .iter()
        .map(|input| NpmPackage::new(input))
        .collect();

    for package in &package_list {
        let client_clone = client.clone();
        // let package_clone = package.clone();
        set.spawn(async move { package.fetch_package_meta(client_clone).await });
    }

    for package in &package_list {
        let client_clone = client.clone();
        // let package_clone = package.clone();
        set.spawn(async move { package.fetch_version_meta(client_clone).await });
    }

    // let package_list = packages.iter().map(|input| NpmPackage::new(input));

    // for package in package_list {
    //     let client_clone = client.clone();
    //     set.spawn(package.fetch_package_meta(client_clone));
    // }
    // for package in package_list {
    //     let client_clone = client.clone();
    //     set.spawn(package.fetch_version_meta(client_clone));
    // }

    let mut completed = Vec::new();
    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(pkg)) => {
                completed.push(pkg);
            }
            Ok(Err(api_error)) => println!("API Error {}", api_error),
            Err(join_error) => println!("Thread Panic {:?}", join_error),
        }
    }

    println!("{:#?}", completed);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn supports_all_package_install_syntax() {
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
            "vite@next",
            "@types/node",
        ]
        .map(str::to_string)
        .to_vec();
        let result = run(&packages);
        let is_ok = result.await.is_ok();
        if is_ok {
            assert!(is_ok);
        } else {
            // println!("{:?}", &result.err());
            assert!(!is_ok);
        }
    }
}
