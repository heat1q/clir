use std::process;

use clir::run;

fn main() {
    if let Err(e) = run() {
        println!("error: {}", e);
        process::exit(1);
    }
}
