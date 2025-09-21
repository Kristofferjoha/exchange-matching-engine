use crate::engine::MatchingEngine;
use crate::order::Order;
use crate::utils::{Side, format_timestamp, MatchingEngineError};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::error::Error;
use uuid::Uuid;
use crate::logging::LoggingStrategy;


#[derive(Debug, Deserialize)]
struct Operation {
    operation: String,
    instrument: String,
    side: Option<String>,
    order_type: Option<String>,
    quantity: Option<Decimal>,
    price: Option<Decimal>,
    order_to_cancel: Option<String>,
}

pub fn run_simulation(strategy: LoggingStrategy) -> Result<(), Box<dyn Error>> {
    let mut engine = MatchingEngine::new();
    let mut instruments = Vec::new();

    if strategy == LoggingStrategy::Naive {
        println!("Starting trading simulation from operations.csv (Verbose Mode)");
    }

    let mut reader = csv::Reader::from_path("operations.csv")?;

    for result in reader.deserialize() {
        let operation: Operation = result?;

        if !engine.has_market(&operation.instrument) {
            engine.add_market(operation.instrument.clone());
            instruments.push(operation.instrument.clone());
            if strategy == LoggingStrategy::Naive {
                println!("Market created for {}", operation.instrument);
            }
        }

        match operation.operation.as_str() {
            "NEW" => {
                let order_id = match operation.order_to_cancel.as_ref().and_then(|id_str| Uuid::parse_str(id_str).ok()) {
                    Some(id) => id,
                    None => {
                        eprintln!(" -> Error: NEW operation requires a valid UUID in the 'order_to_cancel' column.");
                        continue;
                    }
                };
                let side = match operation.side.as_deref() {
                    Some("BUY") => Side::Buy,
                    Some("SELL") => Side::Sell,
                    _ => {
                        eprintln!(" -> Error: NEW operation requires a valid SIDE.");
                        continue;
                    }
                };
                
                let mut order = match operation.order_type.as_deref() {
                    Some("LIMIT") => Order::new_limit(order_id,
                        operation.instrument.clone(),
                        side,
                        operation.price.unwrap_or_default(),
                        operation.quantity.unwrap_or_default(),
                    ),
                    Some("MARKET") => Order::new_market(order_id,
                        operation.instrument.clone(),
                        side,
                        operation.quantity.unwrap_or_default(),
                    ),
                    _ => {
                        eprintln!(" -> Error: NEW operation requires a valid ORDER_TYPE.");
                        continue;
                    }
                };

                if let Some(id_str) = operation.order_to_cancel.as_ref() {
                    match Uuid::parse_str(id_str) {
                        Ok(id) => order.order_id = id,
                        Err(_) => {
                            eprintln!(" -> Error: Invalid UUID format for new order: '{}'", id_str);
                            continue;
                        }
                    }
                } else {
                    eprintln!(" -> Error: NEW operation requires an ID in the 'order_to_cancel' column.");
                    continue;
                }

                if strategy == LoggingStrategy::Naive {
                    println!(
                        " ts: {} | Submitting Order: [ID: {}, Side: {:?}, Type: {:?}, Qty: {}, Price: {}]",
                        format_timestamp(order.timestamp),
                        order.order_id,
                        order.side,
                        order.order_type,
                        order.quantity,
                        order.price.unwrap_or_default(),
                    );
                }

                match engine.process_order(order) {
                    Ok(trades) if !trades.is_empty() => {
                        if strategy == LoggingStrategy::Naive {
                            println!(" Trades Executed!");
                            for trade in trades {
                                println!("{}", trade);
                            }
                        }
                    },
                    Ok(_) => {
                        if strategy == LoggingStrategy::Naive {
                             println!("Order rested on book, no trades.");
                        }
                    },
                    Err(e) => eprintln!(" -> Error processing order: {}", e),
                }
            }
            "CANCEL" => {
                if let Some(id_str_to_cancel) = operation.order_to_cancel.as_ref() {
                    if let Ok(order_id) = Uuid::parse_str(id_str_to_cancel) {
                        if strategy == LoggingStrategy::Naive {
                            println!("Attempting to cancel order ID: {}", order_id);
                        }
                        match engine.cancel_order_by_id(&order_id, &operation.instrument) {
                            Ok(canceled_order) => {
                                if strategy == LoggingStrategy::Naive {
                                    println!(" -> Order {} canceled successfully.", canceled_order.order_id)
                                }
                            },
                            Err(e) => {
                                // This is the updated block.
                                // We now check *why* the cancel failed.
                                match e {
                                    MatchingEngineError::OrderNotFound(_) => {
                                        if strategy == LoggingStrategy::Naive {
                                            println!(" -> Cancel rejected for order {}: not found (likely already filled).", order_id);
                                        }
                                    },
                                    _ => {
                                        // For any other type of error, print it as a real failure.
                                        println!(" -> Failed to cancel order {}: {}", order_id, e);
                                    }
                                }
                            }
                        }
                    } else {
                        eprintln!(" -> Error: Invalid UUID format for order to cancel: '{}'", id_str_to_cancel);
                    }
                } else {
                    eprintln!(" -> Error: CANCEL operation requires an ID in the 'order_to_cancel' column.");
                }
            }
            _ => {
                eprintln!(" -> Error: Unknown operation type '{}'", operation.operation);
            }
        }
    }

    println!("\n--- FINAL ORDER BOOKS ---");
    for instrument in &instruments {
        if let Some(display) = engine.get_order_book_display(instrument) {
            println!("\n--- ORDER BOOK: {} ---", instrument);
            
            println!("  ASKS (Sell Orders):");
            if display.asks.is_empty() {
                println!("    (empty)");
            } else {
                for level in &display.asks {
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

    println!("\nSimulation finished.");
    Ok(())
}