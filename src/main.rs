mod v1;

fn main() {
    let result = v1::cli::run_cli();
    match result {
        Ok(_) => println!("Success"),
        Err(err) => println!("Failed to run: {:?}", err),
    }
}
