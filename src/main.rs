mod v1;

fn main() {
    let result = v1::cli::run_cli();
    match result {
        Ok(_) => {
            std::process::exit(0);
        },
        Err(err) => {
            println!("Failed to run: {:?}", err);
            std::process::exit(1);
        },
    }
}