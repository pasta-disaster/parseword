// src/main.rs

fn main() {
    println!("Initialising Parseword backend");

    if let Err(e) = parseword::cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
