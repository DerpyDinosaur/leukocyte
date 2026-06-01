use super::LeukoError;
use colored::Colorize;

pub fn report(err: &LeukoError) {
    // Print top level error
    eprintln!("{} {}", "error:".red().bold(), err);

    // Print any chain of causes (anyhow preserves these)
    if let LeukoError::Other(inner) = err {
        for cause in inner.chain().skip(1) {
            eprintln!("  {} {}", "caused by:".yellow(), cause);
        }
    }

    // Print contextual hints for errors you know how to fix
    if let Some(hint) = hint(err) {
        eprintln!("\n{} {}", "hint:".cyan().bold(), hint);
    }
}

pub fn exit_code(err: &LeukoError) -> i32 {
    match err {
        _ => 1,
    }
}

fn hint(err: &LeukoError) -> Option<&'static str> {
    match err {
        LeukoError::Network(_) => Some("check your internet connection and try again"),
        _ => None,
    }
}
