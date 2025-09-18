use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;


#[derive(Debug, Clone)]
pub struct Trade {
    pub trade_id: Uuid,
    pub instrument: String,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: u64,
}

impl Trade {
     pub fn new(instrument: String, price: u64, quantity: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let trade_id = Uuid::new_v4();
        Trade {
            trade_id,
            instrument,
            price,
            quantity,
            timestamp,
        }
    }
}