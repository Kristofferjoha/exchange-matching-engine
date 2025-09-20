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