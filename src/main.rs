mod engine;
mod orderbook;
mod trade;
mod order;
mod simulation;
mod utils;
use std::str::FromStr;
mod logging;
use logging::types::LoggingMode;
use crate::logging::create_logger;
use engine::MatchingEngine;
use std::time::Instant;
use std::fs;

use utils::{display_final_matching_engine, load_operations, report_latencies};

use simulation::run_simulation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("output_logs")?;
    
    let args: Vec<String> = std::env::args().collect();
    let mode_str = args.get(1).ok_or("Usage: cargo run <logging_mode>")?;
    let mode = LoggingMode::from_str(mode_str).map_err(|_| "Invalid logging mode")?;
    
    let mut logger = create_logger(mode);

    let mut engine = MatchingEngine::new();
    let instruments = vec!["PUMPTHIS".to_string()];

    for instrument in &instruments {
        engine.add_market(instrument.clone());
        println!("Market created for {}", instrument);
    }

    let operations = load_operations("operations.csv")?;

    let mut latencies: Vec<(u128, u128)> = Vec::with_capacity(operations.len());

    let start = Instant::now();
    if let Err(e) = run_simulation(&mut logger, &mut engine, &operations, &mut latencies) {
        eprintln!("Application error: {}", e);
    }
    display_final_matching_engine(&instruments, &engine);
    println!("Simulation completed in {:.2?}", start.elapsed());

    report_latencies(&latencies);

    let finalize_start = Instant::now();
    logger.finalize();
    let finalize_duration = finalize_start.elapsed().as_nanos();
    println!("Logger finalize took {} ns", finalize_duration);

    Ok(())
}