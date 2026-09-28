mod cli;
mod document;
mod runner;

use std::error::Error;

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("docdoctor: {error}");
        std::process::exit(1);
    }
}
