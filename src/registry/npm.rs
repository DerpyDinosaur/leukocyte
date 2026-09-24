// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tokio;

//     #[tokio::test]
//     async fn parse_name_version() {
//         let package = NpmPackage::new("vite@1.0.0");
//         assert_eq!(package.name(), "vite");
//         assert_eq!(package.version(), "1.0.0");
//     }

//     #[tokio::test]
//     async fn successful_get_meta() {
//         let client = Client::new();
//         let package = NpmPackage::new("vite");

//         let package = package
//             .fetch_package_meta(client.clone())
//             .await
//             .expect("fetch_package_meta failed");

//         let package = package
//             .fetch_version_meta(client)
//             .await
//             .expect("fetch_version_meta failed");

//         assert!(package.meta.is_some());
//         assert!(package.version_meta.is_some());
//     }
// }
