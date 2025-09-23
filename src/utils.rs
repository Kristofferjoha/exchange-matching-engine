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
    
pub fn report_latencies(latencies: &mut Vec<u128>) {
    if latencies.is_empty() {
        return;
    }

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