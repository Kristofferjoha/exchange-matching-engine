use crate::order::Order;
use crate::orderbook::OrderBook;
use crate::trade::Trade;
use crate::utils::{MatchingEngineError, OrderBookDisplay, OrderType};
use std::collections::HashMap;
use uuid::Uuid;

pub struct MatchingEngine {
    books: HashMap<String, OrderBook>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        MatchingEngine {
            books: HashMap::new(),
        }
    }

    pub fn add_market(&mut self, instrument: String) {
        self.books.insert(instrument.clone(), OrderBook::new(instrument));
    }

    pub fn process_order(&mut self, order: Order) -> Result<Vec<Trade>, MatchingEngineError> {
        match order.order_type {
            OrderType::Market if order.price.is_some() => {
                return Err(MatchingEngineError::InvalidOrderPrice)
            }
            OrderType::Limit if order.price.is_none() => {
                return Err(MatchingEngineError::InvalidOrderPrice)
            }
            _ => (),
        }

        match self.books.get_mut(&order.instrument) {
            Some(book) => Ok(book.add_order(order)),
            None => Err(MatchingEngineError::MarketNotFound(order.instrument)),
        }
    }

    pub fn has_market(&self, instrument: &str) -> bool {
        self.books.contains_key(instrument)
    }


    pub fn cancel_order_by_id(&mut self, order_id: &Uuid, instrument: &str) -> Result<Order, MatchingEngineError> {
        if let Some(book) = self.books.get_mut(instrument) {
            book.cancel_order(order_id)
        } else {
            Err(MatchingEngineError::MarketNotFound(instrument.to_string()))
        }
    }

    pub fn get_order_book_display(&self, instrument: &str) -> Option<OrderBookDisplay> {
        self.books.get(instrument).map(|book| book.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::{Order};
    use crate::utils::{Side, OrderType};
    use crate::utils::MatchingEngineError;
    use rust_decimal_macros::dec;

    #[test]
    fn test_add_and_has_market() {
        let mut engine = MatchingEngine::new();
        assert!(!engine.has_market("SOFI"));
        
        engine.add_market("SOFI".to_string());
        assert!(engine.has_market("SOFI"));
    }

    #[test]
    fn test_process_order_for_non_existent_market() {
        let mut engine = MatchingEngine::new();
        let order = Order::new_limit("NON-EXISTENT".to_string(), Side::Buy, dec!(100.0), dec!(10));
        
        let result = engine.process_order(order);

        assert!(result.is_err());
        matches!(result.unwrap_err(), MatchingEngineError::MarketNotFound(market) if market == "NON-EXISTENT");
    }

    #[test]
    fn test_process_order_invalid_price_rules() {
        let mut engine = MatchingEngine::new();
        engine.add_market("SOFI".to_string());

        let mut limit_no_price = Order::new_market("SOFI".to_string(), Side::Buy, dec!(10));
        limit_no_price.order_type = OrderType::Limit;
        let res1 = engine.process_order(limit_no_price);
        assert!(matches!(res1.unwrap_err(), MatchingEngineError::InvalidOrderPrice));

        let mut market_with_price = Order::new_limit("SOFI".to_string(), Side::Buy, dec!(100.0), dec!(10));
        market_with_price.order_type = OrderType::Market;
        let res2 = engine.process_order(market_with_price);
        assert!(matches!(res2.unwrap_err(), MatchingEngineError::InvalidOrderPrice));
    }
}