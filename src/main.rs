use std::env;

mod engine;
mod orderbook;
mod trade;
mod order;
mod simulation;
mod utils;

use simulation::run_simulation;


fn main() {
    let args: Vec<String> = env::args().collect();
    let naive = args.iter().any(|arg| arg == "--naive" || arg == "-v");

    if let Err(e) = run_simulation(naive) {
        eprintln!("Application error: {}", e);
    }
}