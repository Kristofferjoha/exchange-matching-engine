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