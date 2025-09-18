use std::time::{SystemTime, UNIX_EPOCH};
use crate::utils::{OrderType, OrderStatus, Side};
use uuid::Uuid;


#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: Uuid,
    pub instrument: String,
    pub side: Side,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub price: u64,
    pub quantity: u64,
    pub remaining_quantity: u64,
    pub timestamp: u64,
}

impl Order {
    pub fn new(
        instrument: String,
        side: Side,
        order_type: OrderType,
        price: u64,
        quantity: u64,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let new_order_id = Uuid::new_v4();
        
        Order {
            order_id: new_order_id,
            instrument,
            side,
            order_type,
            status: OrderStatus::New,
            price,
            quantity,
            remaining_quantity: quantity,
            timestamp,
        }
    }
}