use cli::cli::run_cli;

fn main() {
    let result = run_cli();
    match result {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(err) => {
            println!("Failed to run: {:?}", err);
            std::process::exit(1);
        }
    }
}
