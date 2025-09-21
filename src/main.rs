mod engine;
mod orderbook;
mod trade;
mod order;
mod simulation;
mod utils;
use std::str::FromStr;
mod logging;
use logging::utils::{LoggingMode, create_logger};
use engine::MatchingEngine;

use utils::display_final_matching_engine;

use simulation::run_simulation;



fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode_str = args.get(1).expect("Usage: cargo run <logging_mode>");
    let mode = LoggingMode::from_str(mode_str).expect("Invalid logging mode");
    
    let mut logger = create_logger(mode);

    let mut engine = MatchingEngine::new();
    let instruments = vec!["PUMPTHIS".to_string()];

    println!("Initializing markets...");
    for instrument in &instruments {
        engine.add_market(instrument.clone());
        println!("  - Market created for {}", instrument);
    }

    if let Err(e) = run_simulation(&mut logger, &mut engine) {
        eprintln!("Application error: {}", e);
    }

    display_final_matching_engine(&instruments,&engine);

    logger.finalize();
}
