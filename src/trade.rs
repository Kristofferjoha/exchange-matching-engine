use crate::utils::Side;
use rust_decimal::Decimal;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use std::fmt;
use crate::utils::format_timestamp;

#[derive(Debug, Clone)]
pub struct Trade {
    pub trade_id: Uuid,
    pub instrument: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: u64,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub taker_side: Side,
}

impl Trade {
    pub fn new(
        instrument: String,
        price: Decimal,
        quantity: Decimal,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
        taker_side: Side,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;


        Trade {
            trade_id: Uuid::new_v4(),
            instrument,
            price,
            quantity,
            timestamp,
            buy_order_id,
            sell_order_id,
            taker_side,
        }
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Trade {{ id: {}, instrument: {}, price: {}, qty: {}, ts: {}, buy_id: {}, sell_id: {}, side: {:?} }}",
            self.trade_id,
            self.instrument,
            self.price,
            self.quantity,
            format_timestamp(self.timestamp),
            self.buy_order_id,
            self.sell_order_id,
            self.taker_side
        )
    }
}