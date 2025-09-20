use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use rust_decimal::prelude::FromPrimitive;
use rand::{Rng, rng};
use uuid::Uuid;
use std::fs::File;
use csv::Writer;
use rand::prelude::IndexedRandom;

const INSTRUMENT: &str = "PUMPTHIS";
const TOTAL_OPERATIONS: usize = 100_000;
const BOOK_BUILD_OPS: usize = 3_000;
const MID_PRICE: Decimal = dec!(100);
const SPREAD: Decimal = dec!(0.5);
const TICK_SIZE: Decimal = dec!(0.05);

#[derive(Clone, Copy)]
enum OpType {
    NewLimit,
    NewMarket,
    Cancel,
}

const OP_WEIGHTS: &[(OpType, f64)] = &[
    (OpType::NewLimit, 0.55),
    (OpType::NewMarket, 0.15),
    (OpType::Cancel, 0.30),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rng();
    let file = File::create("operations.csv")?;
    let mut wtr = Writer::from_writer(file);

    wtr.write_record(&["operation", "instrument", "side", "order_type", "quantity", "price", "order_to_cancel"])?;

    let mut open_limit_orders: Vec<Uuid> = Vec::with_capacity(TOTAL_OPERATIONS);

    for i in 0..TOTAL_OPERATIONS {
        let op_type = if i < BOOK_BUILD_OPS {
            OpType::NewLimit
        } else {
            OP_WEIGHTS.choose_weighted(&mut rng, |item| item.1).unwrap().0
        };

        match op_type {
            OpType::NewLimit => {
                let side = if rng.random_range(0..=1) == 1 { "BUY" } else { "SELL" };
                let price_offset = Decimal::from_f64(rng.random_range(0.05..2.0)).unwrap().round_dp(2);
                let is_aggressive = rng.random_bool(0.1); 

                let raw_price = if is_aggressive {

                    if side == "BUY" {
                        MID_PRICE + SPREAD + price_offset
                    } else {
                        MID_PRICE - SPREAD - price_offset
                    }
                } else {

                    if side == "BUY" {
                        MID_PRICE - SPREAD - price_offset
                    } else {
                        MID_PRICE + SPREAD + price_offset
                    }
                };

                let price = (raw_price / TICK_SIZE).round() * TICK_SIZE;

                let quantity_int = rng.random_range(1..=100); 
                let quantity = Decimal::from(quantity_int);
                let new_order_id = Uuid::new_v4();
                open_limit_orders.push(new_order_id);

                wtr.write_record(&[
                    "NEW",
                    INSTRUMENT,
                    side,
                    "LIMIT",
                    &quantity.to_string(),
                    &price.to_string(),
                    &new_order_id.to_string(),
                ])?;
            }
            OpType::NewMarket => {
                let side = if rng.random_range(0..=1) == 1 { "BUY" } else { "SELL" };
                let quantity_int = rng.random_range(50..=500); 
                let quantity = Decimal::from(quantity_int);
                let new_order_id = Uuid::new_v4();
                wtr.write_record(&[
                    "NEW",
                    INSTRUMENT,
                    side,
                    "MARKET",
                    &quantity.to_string(),
                    "",
                    &new_order_id.to_string(),
                ])?;
            }
            OpType::Cancel => {
                if !open_limit_orders.is_empty() {
                    let index_to_cancel = rng.random_range(0..open_limit_orders.len());
                    let order_id_to_cancel = open_limit_orders.remove(index_to_cancel);
                    wtr.write_record(&["CANCEL", INSTRUMENT, "", "", "", "", &order_id_to_cancel.to_string()])?;
                }
            }
        }
    }

    wtr.flush()?;
    println!("Generated operations.csv with {} records.", TOTAL_OPERATIONS);
    Ok(())
}