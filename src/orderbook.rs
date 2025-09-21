use crate::order::Order;
use crate::trade::Trade;
use crate::utils::{MatchingEngineError, OrderBookDisplay, OrderStatus, OrderType, PriceLevel, Side};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap, VecDeque};
use uuid::Uuid;

pub struct OrderBook {
    instrument: String,
    bids: BTreeMap<Decimal, VecDeque<Uuid>>,
    asks: BTreeMap<Decimal, VecDeque<Uuid>>,
    orders: HashMap<Uuid, Order>,
}

impl OrderBook {
    pub fn new(instrument: String) -> Self {
        OrderBook {
            instrument,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
        }
    }

    pub fn add_order(&mut self, mut order: Order) -> Vec<Trade> {
        let trades = self.match_order(&mut order);

        if !order.is_filled() && order.order_type == OrderType::Limit {
            let order_id = order.order_id;
            if let Some(price) = order.price {
                let book_side = match order.side {
                    Side::Buy => &mut self.bids,
                    Side::Sell => &mut self.asks,
                };
                book_side.entry(price).or_default().push_back(order_id);
                
                self.orders.insert(order_id, order);
            }
        }

        trades
    }

    pub fn cancel_order(&mut self, order_id: &Uuid) -> Result<Order, MatchingEngineError> {
        if let Some(mut order_to_cancel) = self.orders.remove(order_id) {
            let book = match order_to_cancel.side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };

            if let Some(price) = order_to_cancel.price {
                if let Some(queue) = book.get_mut(&price) {
                    queue.retain(|id| id != order_id);
                    if queue.is_empty() {
                        book.remove(&price);
                    }
                }
            }
            
            order_to_cancel.status = OrderStatus::Canceled;
            Ok(order_to_cancel)
        } else {
            Err(MatchingEngineError::OrderNotFound(*order_id))
        }
    }

    fn match_order(&mut self, incoming: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        let prices_to_process = self.get_matchable_prices(incoming);

        for price in prices_to_process {
            if incoming.is_filled() {
                break;
            }
            let mut trades_at_price = self.process_level(incoming, price);
            trades.append(&mut trades_at_price);
        }

        trades
    }

    fn process_level(&mut self, incoming: &mut Order, price: Decimal) -> Vec<Trade> {
        let mut trades = Vec::new();
        let (opposite_book, _opposite_side) = match incoming.side {
            Side::Buy => (&mut self.asks, Side::Sell),
            Side::Sell => (&mut self.bids, Side::Buy),
        };

        while let Some(queue) = opposite_book.get_mut(&price) {
            if incoming.is_filled() || queue.is_empty() {
                break;
            }

            let resting_id = *queue.front().expect("Queue is not empty, so front must exist.");
            let resting = self.orders.get_mut(&resting_id).expect("Order must exist in master map.");

            let trade_qty = incoming.remaining_quantity.min(resting.remaining_quantity);

            incoming.fill(trade_qty);
            resting.fill(trade_qty);

            let (buy_order_id, sell_order_id) = if incoming.side == Side::Buy {
                (incoming.order_id, resting.order_id)
            } else {
                (resting.order_id, incoming.order_id)
            };
            
            trades.push(Trade::new(
                self.instrument.clone(),
                price,
                trade_qty,
                buy_order_id,
                sell_order_id,
                incoming.side,
            ));

            if resting.is_filled() {
                queue.pop_front();
                self.orders.remove(&resting_id);
            }
        }

        if let Some(queue) = opposite_book.get(&price) {
            if queue.is_empty() {
                opposite_book.remove(&price);
            }
        }

        trades
    }

    fn get_matchable_prices(&self, incoming: &Order) -> Vec<Decimal> {
        let mut prices = Vec::new();
        match incoming.side {
            Side::Buy => {
                for (&price, queue) in self.asks.iter() {
                    if queue.is_empty() { continue; }

                    if let Some(limit_price) = incoming.price {
                        if price <= limit_price {
                            prices.push(price);
                        } else {
                            break;
                        }
                    } else {
                        prices.push(price);
                    }
                }
            }
            Side::Sell => {
                for (&price, queue) in self.bids.iter().rev() {
                     if queue.is_empty() { continue; }

                    if let Some(limit_price) = incoming.price {
                        if price >= limit_price {
                            prices.push(price);
                        } else {
                            break;
                        }
                    } else {
                        prices.push(price);
                    }
                }
            }
        }
        prices
    }

    pub fn display(&self) -> OrderBookDisplay {
        let bids = self.bids
            .iter()
            .rev()
            .map(|(&price, queue)| {
                let volume: Decimal = queue
                    .iter()
                    .map(|id| self.orders.get(id).unwrap().remaining_quantity)
                    .sum();
                PriceLevel { price, volume }
            })
            .filter(|level| !level.volume.is_zero())
            .collect();

        let asks = self.asks
            .iter()
            .map(|(&price, queue)| {
                let volume: Decimal = queue
                    .iter()
                    .map(|id| self.orders.get(id).unwrap().remaining_quantity)
                    .sum();
                PriceLevel { price, volume }
            })
            .filter(|level| !level.volume.is_zero())
            .collect();

        OrderBookDisplay { bids, asks }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn setup_book() -> OrderBook {
        OrderBook::new("TEST-STOCK".to_string())
    }

    #[test]
    fn test_new_order_book_is_empty() {
        let book = setup_book();
        assert_eq!(book.instrument, "TEST-STOCK");
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
        assert!(book.orders.is_empty());
    }

    #[test]
    fn test_add_single_buy_order() {
        let mut book = setup_book();
        let order = Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(150.0), dec!(10));
        let order_id = order.order_id;

        let trades = book.add_order(order);

        assert!(trades.is_empty());
        assert_eq!(book.orders.len(), 1);
        assert_eq!(book.bids.len(), 1);
        assert!(book.asks.is_empty());
        assert!(book.orders.contains_key(&order_id));
        assert_eq!(book.bids.get(&dec!(150.0)).unwrap().front().unwrap(), &order_id);
    }

    #[test]
    fn test_add_multiple_orders_at_same_price_level() {
        let mut book = setup_book();
        let order1 = Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(150.0), dec!(10));
        let order2 = Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(150.0), dec!(5));
        let order1_id = order1.order_id;
        let order2_id = order2.order_id;

        book.add_order(order1);
        book.add_order(order2);

        assert_eq!(book.orders.len(), 2);
        assert_eq!(book.bids.len(), 1);
        
        let price_level_queue = book.bids.get(&dec!(150.0)).unwrap();
        assert_eq!(price_level_queue.len(), 2);
        assert_eq!(price_level_queue.get(0).unwrap(), &order1_id);
        assert_eq!(price_level_queue.get(1).unwrap(), &order2_id);
    }

    #[test]
    fn test_cancel_order() {
        let mut book = setup_book();
        let order = Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Sell, dec!(200.0), dec!(5));
        let order_id = order.order_id;
        book.add_order(order);
        assert!(!book.orders.is_empty());
        assert!(!book.asks.is_empty());

        let result = book.cancel_order(&order_id);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().order_id, order_id);
        assert!(book.orders.is_empty());
        assert!(book.asks.is_empty());
    }
    
    #[test]
    fn test_cancel_order_from_level_with_multiple_orders() {
        let mut book = setup_book();
        let order1 = Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(100.0), dec!(10));
        let order2 = Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(100.0), dec!(5));
        let order1_id = order1.order_id;
        let order2_id = order2.order_id;
        book.add_order(order1);
        book.add_order(order2);

        let result = book.cancel_order(&order1_id);

        assert!(result.is_ok());
        assert_eq!(book.orders.len(), 1);
        assert_eq!(book.bids.len(), 1);

        let price_level_queue = book.bids.get(&dec!(100.0)).unwrap();
        assert_eq!(price_level_queue.len(), 1);
        assert_eq!(price_level_queue.front().unwrap(), &order2_id);
    }

    #[test]
    fn test_cancel_non_existent_order_returns_err() {
        let mut book = setup_book();
        let non_existent_id = Uuid::new_v4();

        let result = book.cancel_order(&non_existent_id);

        assert!(result.is_err());
        matches!(result.unwrap_err(), MatchingEngineError::OrderNotFound(id) if id == non_existent_id);
    }
    
    #[test]
    fn test_get_matchable_prices_for_buy_limit_order() {
        let mut book = setup_book();

        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Sell, dec!(101.0), dec!(10)));
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Sell, dec!(102.0), dec!(10)));
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Sell, dec!(103.0), dec!(10)));

        let incoming_order = Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(102.0), dec!(5));

        let prices = book.get_matchable_prices(&incoming_order);

        assert_eq!(prices, vec![dec!(101.0), dec!(102.0)]);
    }

    #[test]
    fn test_get_matchable_prices_for_sell_limit_order() {
        let mut book = setup_book();
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(99.0), dec!(10)));
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(100.0), dec!(10)));
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(101.0), dec!(10)));

        let incoming_order = Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Sell, dec!(100.0), dec!(5));

        let prices = book.get_matchable_prices(&incoming_order);

        assert_eq!(prices, vec![dec!(101.0), dec!(100.0)]);
    }

    #[test]
    fn test_get_matchable_prices_for_buy_market_order() {
        let mut book = setup_book();
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Sell, dec!(101.0), dec!(10)));
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Sell, dec!(102.0), dec!(10)));
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Sell, dec!(103.0), dec!(10)));

        let incoming_order = Order::new_market(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(5));

        let prices = book.get_matchable_prices(&incoming_order);

        assert_eq!(prices, vec![dec!(101.0), dec!(102.0), dec!(103.0)]);
    }

    #[test]
    fn test_get_matchable_prices_for_sell_market_order() {
        let mut book = setup_book();
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(98.0), dec!(10)));
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(99.0), dec!(10)));
        book.add_order(Order::new_limit(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Buy, dec!(97.0), dec!(10)));

        let incoming_order = Order::new_market(Uuid::new_v4(), "TEST-STOCK".to_string(), Side::Sell, dec!(5));

        let prices = book.get_matchable_prices(&incoming_order);

        assert_eq!(prices, vec![dec!(99.0), dec!(98.0), dec!(97.0)]);
    }
}