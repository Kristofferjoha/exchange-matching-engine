use std::env;

mod engine;
mod orderbook;
mod trade;
mod order;
mod simulation;
mod utils;
mod logging;

use simulation::run_simulation;
use logging::LoggingStrategy;


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut strategy = LoggingStrategy::Baseline;

    if args.iter().any(|arg| arg == "--naive" || arg == "-v") {
        strategy = LoggingStrategy::Naive;
    }

    if let Err(e) = run_simulation(strategy) {
        eprintln!("Application error: {}", e);
    }
}