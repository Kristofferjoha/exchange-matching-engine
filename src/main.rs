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
use std::time::Instant;

use utils::{display_final_matching_engine, load_operations};

use simulation::run_simulation;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mode_str = args.get(1).expect("Usage: cargo run <logging_mode>");
    let mode = LoggingMode::from_str(mode_str).expect("Invalid logging mode");
    
    let mut logger = create_logger(mode);

    let mut engine = MatchingEngine::new();
    let instruments = vec!["PUMPTHIS".to_string()];

    for instrument in &instruments {
        engine.add_market(instrument.clone());
        println!("Market created for {}", instrument);
    }

    let operations = load_operations("operations.csv")?;

    let mut latencies: Vec<u128> = Vec::with_capacity(operations.len());

    let start = Instant::now();
    if let Err(e) = run_simulation(&mut logger, &mut engine, &operations, &mut latencies) {
        eprintln!("Application error: {}", e);
    }
    display_final_matching_engine(&instruments, &engine);
    println!("Simulation completed in {:.2?}", start.elapsed());

    if !latencies.is_empty() {
        latencies.sort_unstable();

        let count = latencies.len();
        let sum: u128 = latencies.iter().sum();
        let mean = sum / count as u128;
        let median = latencies[count / 2];
        let p99 = latencies[(count as f64 * 0.99) as usize];
        let p999 = latencies[(count as f64 * 0.999) as usize];

        println!("\n--- Latency Distribution (nanoseconds) ---");
        println!("          Count: {}", count);
        println!("           Mean: {}", mean);
        println!("         Median: {}", median);
        println!("  99th Percentile: {}", p99);
        println!("99.9th Percentile: {}", p999);
        println!("------------------------------------------");
    }


    logger.finalize();

    Ok(())
}