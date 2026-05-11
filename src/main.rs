mod delegate;
use std::{env};

const PACKAGE_MANAGER: &[&str] = &["bun", "npm", "yarn", "pnpm"];

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 && PACKAGE_MANAGER.contains(&args[1].as_str()) {
        println!("{} package manager is compatible with leuko", args[1]);
    }

    delegate::execute();
}
