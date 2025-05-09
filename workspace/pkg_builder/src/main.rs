use std::error::Error;

use cli::cli::run_cli;

fn main() {
    let result = run_cli();
    match result {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(err) => {
            eprintln!("{}", format_error(&err));
            std::process::exit(1);
        }
    }
}

fn format_error(err: &dyn Error) -> String {
    let message = err.to_string();
    let mut chain = vec![message.clone()];
    let mut current = err.source();
    while let Some(source) = current {
        chain.push(source.to_string());
        current = source.source();
    }
    let chain_str = chain
        .iter()
        .map(|s| s.lines().next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" -> ");
    format!("Error:\n>{}\n{}", chain_str, message)
}