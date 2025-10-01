use crate::order::Order;
use crate::orderbook::OrderBook;
use crate::trade::Trade;
use crate::utils::{MatchingEngineError, OrderBookDisplay, OrderType};
use std::collections::HashMap;
use uuid::Uuid;
use crate::logging::logger_trait::SimLogger;
use std::time::Instant;

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

    pub fn process_order(&mut self, order: Order, logger: &mut Box<dyn SimLogger>) -> Result<(Vec<Trade>, u128), MatchingEngineError> {
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
            Some(book) => {
                let (trades, filled_orders, final_incoming_state) = book.add_order(order);

                let log_start = Instant::now();
                for trade in &trades {
                    logger.log_trade(trade);
                }
                for filled_order in filled_orders {
                    logger.log_order_filled(&filled_order);
                }
                if final_incoming_state.is_filled() || final_incoming_state.order_type == OrderType::Market {
                    logger.log_order_filled(&final_incoming_state);
                }
                let log_duration = log_start.elapsed().as_nanos();

                Ok((trades, log_duration))
            }
            None => Err(MatchingEngineError::MarketNotFound(order.instrument)),
        }
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
    use crate::logging::types::LoggingMode;
    use crate::logging::create_logger;
    use crate::order::{Order};
    use crate::utils::{Side, OrderType};
    use crate::utils::MatchingEngineError;
    use rust_decimal_macros::dec;
    use uuid::Uuid;



    #[test]
    fn test_process_order_for_non_existent_market() {
        let mut engine = MatchingEngine::new();
        let order = Order::new_limit(Uuid::new_v4(), "NON-EXISTENT".to_string(), Side::Buy, dec!(100.0), dec!(10));
        let mut logger = create_logger(LoggingMode::Baseline);
        
        let result = engine.process_order(order, &mut logger);

        assert!(result.is_err());
        matches!(result.unwrap_err(), MatchingEngineError::MarketNotFound(market) if market == "NON-EXISTENT");
    }

    #[test]
    fn test_process_order_invalid_price_rules() {
        let mut engine = MatchingEngine::new();
        engine.add_market("SOFI".to_string());
        let mut logger = create_logger(LoggingMode::Baseline);

        let mut limit_no_price = Order::new_market(Uuid::new_v4(), "SOFI".to_string(), Side::Buy, dec!(10));
        limit_no_price.order_type = OrderType::Limit;
        let res1 = engine.process_order(limit_no_price, &mut logger);
        assert!(matches!(res1.unwrap_err(), MatchingEngineError::InvalidOrderPrice));

        let mut market_with_price = Order::new_limit(Uuid::new_v4(), "SOFI".to_string(), Side::Buy, dec!(100.0), dec!(10));
        market_with_price.order_type = OrderType::Market;
        let res2 = engine.process_order(market_with_price, &mut logger);
        assert!(matches!(res2.unwrap_err(), MatchingEngineError::InvalidOrderPrice));
    }
}