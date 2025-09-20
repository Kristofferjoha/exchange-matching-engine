use crate::utils::{OrderStatus, OrderType, Side};
use rust_decimal::Decimal;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub order_id: Uuid,
    pub instrument: String,
    pub side: Side,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub timestamp: u64,
}

impl Order {
    pub fn new_limit(
        instrument: String,
        side: Side,
        price: Decimal,
        quantity: Decimal,
    ) -> Self {
        Self::new(instrument, side, OrderType::Limit, Some(price), quantity)
    }

    pub fn new_market(instrument: String, side: Side, quantity: Decimal) -> Self {
        Self::new(instrument, side, OrderType::Market, None, quantity)
    }

    fn new(
        instrument: String,
        side: Side,
        order_type: OrderType,
        price: Option<Decimal>,
        quantity: Decimal,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before the UNIX epoch, something is very wrong.")
            .as_nanos() as u64;

        Order {
            order_id: Uuid::new_v4(),
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
    pub fn is_filled(&self) -> bool {
        self.remaining_quantity.is_zero()
    }

    pub fn fill(&mut self, qty: Decimal) {
        if qty > self.remaining_quantity {
            self.remaining_quantity = Decimal::ZERO;
        } else {
            self.remaining_quantity -= qty;
        }

        if self.is_filled() {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
    }
}