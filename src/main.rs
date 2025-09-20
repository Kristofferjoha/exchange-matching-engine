use exchange_matching_engine::engine::MatchingEngine;
use exchange_matching_engine::order::Order;
use exchange_matching_engine::utils::{Side, format_timestamp};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Operation {
    #[serde(rename = "TYPE")]
    op_type: String,
    #[serde(rename = "INSTRUMENT")]
    instrument: String,
    #[serde(rename = "SIDE")]
    side: Option<String>,
    #[serde(rename = "ORDER_TYPE")]
    order_type: Option<String>,
    #[serde(rename = "PRICE")]
    price: Option<Decimal>,
    #[serde(rename = "QUANTITY")]
    quantity: Option<Decimal>,
    #[serde(rename = "LABEL")]
    label: Option<String>,
}

fn run_simulation() -> Result<(), Box<dyn Error>> {
    let mut engine = MatchingEngine::new();
    let mut order_ids: HashMap<String, Uuid> = HashMap::new();

    println!("Starting trading simulation from operations.csv");

    let mut reader = csv::Reader::from_path("operations.csv")?;

    for result in reader.deserialize() {
        let operation: Operation = result?;

        if !engine.has_market(&operation.instrument) {
            engine.add_market(operation.instrument.clone());
            println!("Market created for {}", operation.instrument);
        }

        match operation.op_type.as_str() {
            "ADD" => {
                let side = match operation.side.as_deref() {
                    Some("BUY") => Side::Buy,
                    Some("SELL") => Side::Sell,
                    _ => {
                        eprintln!(" -> Error: ADD operation requires a valid SIDE.");
                        continue;
                    }
                };
                
                let order = match operation.order_type.as_deref() {
                    Some("LIMIT") => Order::new_limit(
                        operation.instrument.clone(),
                        side,
                        operation.price.unwrap_or_default(),
                        operation.quantity.unwrap_or_default(),
                    ),
                    Some("MARKET") => Order::new_market(
                        operation.instrument.clone(),
                        side,
                        operation.quantity.unwrap_or_default(),
                    ),
                    _ => {
                        eprintln!(" -> Error: ADD operation requires a valid ORDER_TYPE.");
                        continue;
                    }
                };

                println!(
                    " ts: {} | Submitting Order: [ID: {}, Side: {:?}, Type: {:?}, Qty: {}, Price: {}]",
                    format_timestamp(order.timestamp),
                    order.order_id,
                    order.side,
                    order.order_type,
                    order.quantity,
                    order.price.unwrap_or_default(),
                );

                if let Some(label) = operation.label.as_ref() {
                    order_ids.insert(label.clone(), order.order_id);
                }

                match engine.process_order(order) {
                    Ok(trades) if !trades.is_empty() => {
                        println!(" Trades Executed!");
                        for trade in trades {
                             println!(" -> Trade: Price: {}, Qty: {}", trade.price, trade.quantity);
                        }
                    },
                    Ok(_) => println!("Order rested on book, no trades."),
                    Err(e) => eprintln!(" -> Error processing order: {}", e),
                }
            }
            "CANCEL" => {
                if let Some(label) = operation.label.as_ref() {
                    if let Some(order_id) = order_ids.get(label) {
                         println!("Attempting to cancel order labeled '{}' (ID: {})", label, order_id);
                         match engine.cancel_order_by_id(order_id, &operation.instrument) {
                            Ok(canceled_order) => println!("Order {} canceled successfully.", canceled_order.order_id),
                            Err(e) => println!("Failed to cancel order {}: {}", order_id, e),
                        }
                    } else {
                         eprintln!(" -> Error: No order found with label '{}' to cancel.", label);
                    }
                } else {
                     eprintln!(" -> Error: CANCEL operation requires a LABEL.");
                }
            }
            _ => {
                eprintln!(" -> Error: Unknown operation type '{}'", operation.op_type);
            }
        }
        
        if let Some(display) = engine.get_order_book_display(&operation.instrument) {
            println!("\n--- ORDER BOOK: {} ---", operation.instrument);
            
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

    println!("\n\nSimulation finished.");
    Ok(())
}


fn main() {
    if let Err(e) = run_simulation() {
        eprintln!("Application error: {}", e);
    }
}

