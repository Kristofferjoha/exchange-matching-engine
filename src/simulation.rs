use crate::engine::{MatchingEngine};
use crate::order::Order;
use crate::utils::Side;
use std::error::Error;
use uuid::Uuid;
use crate::logging::utils::SimLogger;
use crate::utils::Operation;

pub fn run_simulation(
    logger: &mut Box<dyn SimLogger>,
    engine: &mut MatchingEngine,
    operations: &[Operation],
) -> Result<(), Box<dyn Error>> {
    for operation in operations {
        match operation.operation.as_str() {
            "NEW" => {
                let Some(id_str) = operation.order_to_cancel.as_ref() else {
                    eprintln!(" -> Error: NEW operation requires an ID in the 'order_to_cancel' column.");
                    continue;
                };

                let Ok(order_id) = Uuid::parse_str(id_str) else {
                    eprintln!(" -> Error: Invalid UUID format for new order: '{}'", id_str);
                    continue;
                };

                let side = match operation.side.as_deref() {
                    Some("BUY") => Side::Buy,
                    Some("SELL") => Side::Sell,
                    _ => {
                        eprintln!(" -> Error: NEW operation requires a valid SIDE.");
                        continue;
                    }
                };
                
                let order = match operation.order_type.as_deref() {
                    Some("LIMIT") => {
                        let Some(price) = operation.price else {
                            eprintln!(" -> Error: LIMIT order requires a valid PRICE.");
                            continue;
                        };
                        Order::new_limit(
                            order_id,
                            operation.instrument.clone(),
                            side,
                            price,
                            operation.quantity.unwrap_or_default(),
                        )
                    },
                    Some("MARKET") => Order::new_market(
                        order_id,
                        operation.instrument.clone(),
                        side,
                        operation.quantity.unwrap_or_default(),
                    ),
                    _ => {
                        eprintln!(" -> Error: NEW operation requires a valid ORDER_TYPE.");
                        continue;
                    }
                };

                logger.log_order_submission(&order);

                match engine.process_order(order, logger) {
                    Ok(_) => {
                    }
                    Err(e) => eprintln!(" -> Error processing order: {}", e),
                }
            }
            "CANCEL" => {
                let Some(id_str_to_cancel) = operation.order_to_cancel.as_ref() else {
                    eprintln!(" -> Error: CANCEL operation requires an ID in the 'order_to_cancel' column.");
                    continue;
                };

                let Ok(order_id) = Uuid::parse_str(id_str_to_cancel) else {
                    eprintln!(" -> Error: Invalid UUID format for order to cancel: '{}'", id_str_to_cancel);
                    continue;
                };

                let success = engine.cancel_order_by_id(&order_id, &operation.instrument).is_ok();
                
                logger.log_order_cancel(&order_id, success);
            }
            _ => {
                eprintln!(" -> Error: Unknown operation type '{}'", operation.operation);
            }
        }
    }

    println!("\nFinished processing simulation operations.");
    Ok(())
}