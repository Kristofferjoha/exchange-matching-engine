use rust_decimal::Decimal;
use thiserror::Error;
use crate::engine::MatchingEngine;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
}

#[derive(Debug, Deserialize)]
pub struct Operation {
    pub operation: String,
    pub instrument: String,
    pub side: Option<String>,
    pub order_type: Option<String>,
    pub quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub order_to_cancel: Option<String>,
}

#[derive(Error, Debug)]
pub enum MatchingEngineError {
    #[error("Market for instrument '{0}' does not exist")]
    MarketNotFound(String), 
    #[error("Order ID '{0}' not found")]
    OrderNotFound(uuid::Uuid),
    #[error("Invalid order price: Market orders cannot have a price, and limit orders must")]
    InvalidOrderPrice,
}

#[derive(Debug)]
pub struct PriceLevel {
    pub price: Decimal,
    pub volume: Decimal,
}

#[derive(Debug)]
pub struct OrderBookDisplay {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

pub fn display_final_matching_engine(instruments: &[String], engine: &MatchingEngine) {
    println!("\n--- FINAL ORDER BOOKS ---");
    for instrument in instruments {
        if let Some(display) = engine.get_order_book_display(instrument) {
            println!("\n--- ORDER BOOK: {} ---", instrument);
            
            println!("  ASKS (Sell Orders):");
            if display.asks.is_empty() {
                println!("    (empty)");
            } else {
                for level in display.asks.iter().rev() {
                    println!("    Price: {:<10} | Volume: {}", level.price.round_dp(2), level.volume);
                }
            }
            
            println!("  ---------------------------");

            println!("  BIDS (Buy Orders):");
            if display.bids.is_empty() {
                println!("    (empty)");
            } else {
                for level in &display.bids {
                    println!("    Price: {:<10} | Volume: {}", level.price.round_dp(2), level.volume);
                }
            }
            println!("-----------------------------");
        }
    }
}

pub fn load_operations(path: &str) -> Result<Vec<Operation>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    
    const EXPECTED_RECORDS: usize = 100_000; 
    let mut ops = Vec::with_capacity(EXPECTED_RECORDS);

    for result in reader.deserialize() {
        ops.push(result?);
    }
    
    Ok(ops)
}

pub fn report_latencies(latencies: &[(u128, u128)]) {
    if latencies.is_empty() {
        println!("No latencies recorded.");
        return;
    }

    let mut process_latencies: Vec<u128> = latencies.iter().map(|(p, _)| *p).collect();
    let mut log_latencies: Vec<u128> = latencies.iter().map(|(_, l)| *l).collect();

    process_latencies.sort_unstable();
    log_latencies.sort_unstable();

    let count = process_latencies.len();
    let process_sum: u128 = process_latencies.iter().sum();
    let log_sum: u128 = log_latencies.iter().sum();
    let process_mean = process_sum as f64 / count as f64;
    let log_mean = log_sum as f64 / count as f64;
    let process_median = process_latencies[count / 2];
    let log_median = log_latencies[count / 2];
    let process_p99 = process_latencies[((count as f64 * 0.99).ceil() as usize).min(count - 1)];
    let log_p99 = log_latencies[((count as f64 * 0.99).ceil() as usize).min(count - 1)];
    let process_p999 = process_latencies[((count as f64 * 0.999).ceil() as usize).min(count - 1)];
    let log_p999 = log_latencies[((count as f64 * 0.999).ceil() as usize).min(count - 1)];

    println!("\n--- Latency Distribution (nanoseconds) ---");
    println!("Processing:");
    println!("{:<25} {}", "Count:", count);
    println!("{:<25} {:.2}", "Mean:", process_mean);
    println!("{:<25} {}", "Median:", process_median);
    println!("{:<25} {}", "99th Percentile:", process_p99);
    println!("{:<25} {}", "99.9th Percentile:", process_p999);
    println!("Logging:");
    println!("{:<25} {}", "Count:", count);
    println!("{:<25} {:.2}", "Mean:", log_mean);
    println!("{:<25} {}", "Median:", log_median);
    println!("{:<25} {}", "99th Percentile:", log_p99);
    println!("{:<25} {}", "99.9th Percentile:", log_p999);
    println!("------------------------------------------");
}