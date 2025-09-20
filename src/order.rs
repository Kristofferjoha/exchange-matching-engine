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

    pub fn new_market(instrument: String,
        side: Side,
        quantity: Decimal
    ) -> Self {
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


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_limit_order_creation() {
        let order = Order::new_limit("SOFI".to_string(), Side::Buy, dec!(29), dec!(1));
        assert!(order.order_id != Uuid::nil());
        assert_eq!(order.instrument, "SOFI");
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.status, OrderStatus::New);
        assert_eq!(order.price, Some(dec!(29)));
        assert_eq!(order.quantity, dec!(1));
        assert_eq!(order.remaining_quantity, dec!(1));
        assert!(order.timestamp > 0);
    }

    #[test]
    fn test_limit_order_filling() {
        let mut order = Order::new_limit("SOFI".to_string(), Side::Buy, dec!(29), dec!(1));

        order.fill(dec!(1));
        assert_eq!(order.remaining_quantity, dec!(0));
        assert_eq!(order.status, OrderStatus::Filled);
        assert!(order.is_filled());
    }

    #[test]
    fn test_limit_order_partially_filling() {
        let mut order = Order::new_limit("SOFI".to_string(), Side::Buy, dec!(29), dec!(1));
        order.fill(dec!(0.4));
        assert_eq!(order.remaining_quantity, dec!(0.6));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert!(!order.is_filled());
    }

    #[test]
    fn test_limit_order_partially_and_filling() {
        let mut order = Order::new_limit("SOFI".to_string(), Side::Buy, dec!(29), dec!(1));
        order.fill(dec!(0.4));
        assert_eq!(order.remaining_quantity, dec!(0.6));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert!(!order.is_filled());

        order.fill(dec!(0.6));
        assert_eq!(order.remaining_quantity, dec!(0));
        assert_eq!(order.status, OrderStatus::Filled);
        assert!(order.is_filled());
    }

    #[test]
    fn test_market_order_creation() {
        let order = Order::new_market("NVO".to_string(), Side::Sell, dec!(2));
        assert!(order.order_id != Uuid::nil());
        assert_eq!(order.instrument, "NVO");
        assert_eq!(order.side, Side::Sell);
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.status, OrderStatus::New);
        assert_eq!(order.price, None);
        assert_eq!(order.quantity, dec!(2));
        assert_eq!(order.remaining_quantity, dec!(2));
        assert!(order.timestamp > 0);
    }

    #[test]
    fn test_market_order_filling() {
        let mut order = Order::new_market("NVO".to_string(), Side::Sell, dec!(2));

        order.fill(dec!(2));
        assert_eq!(order.remaining_quantity, dec!(0));
        assert_eq!(order.status, OrderStatus::Filled);
        assert!(order.is_filled());
    }

    #[test]
    fn test_market_order_partially_filling() {
        let mut order = Order::new_market("NVO".to_string(), Side::Sell, dec!(2));
        order.fill(dec!(0.5));
        assert_eq!(order.remaining_quantity, dec!(1.5));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert!(!order.is_filled());
    }

    #[test]
    fn test_market_order_partially_and_filling() {
        let mut order = Order::new_market("NVO".to_string(), Side::Sell, dec!(2));
        order.fill(dec!(0.5));
        assert_eq!(order.remaining_quantity, dec!(1.5));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert!(!order.is_filled());

        order.fill(dec!(1.5));
        assert_eq!(order.remaining_quantity, dec!(0));
        assert_eq!(order.status, OrderStatus::Filled);
        assert!(order.is_filled());
    }
}