use rust_decimal::Decimal;
use thiserror::Error;
use chrono::{DateTime, Local};

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

pub fn format_timestamp(ts: u64) -> String {
    let utc_dt = DateTime::from_timestamp(
        (ts / 1_000_000_000) as i64,
        (ts % 1_000_000_000) as u32,
    ).unwrap();

    let datetime: DateTime<Local> = utc_dt.with_timezone(&Local);

    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}