// src/orderbook.rs

use std::collections::{BTreeMap, HashMap};
use crate::models::{Order, OrderType, Side, Trade};

pub struct OrderBook {
    bids: BTreeMap<u64, Vec<Order>>,
    asks: BTreeMap<u64, Vec<Order>>,
    order_map: HashMap<u64, Order>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_map: HashMap::new(),
        }
    }

    pub fn add_order(&mut self, mut order: Order) -> Vec<Trade> {
        match order.order_type {
            OrderType::Market => {
                match order.side {
                    Side::Buy => order.price = u64::MAX,
                    Side::Sell => order.price = 0,
                }
            }
            _ => (), // For Limit orders, the price is already set
        }
        
        self.add_limit_order(order)
    }

    fn add_limit_order(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        match order.side {
            Side::Buy => {
                while let Some(mut entry) = self.asks.first_entry() {
                    if order.price < *entry.key() {
                        break;
                    }

                    // Get the price BEFORE the mutable borrow
                    let ask_price = *entry.key();
                    let asks_at_price = entry.get_mut();
                    let mut orders_to_remove = Vec::new();

                    for (i, ask_order) in asks_at_price.iter_mut().enumerate() {
                        let trade_quantity = std::cmp::min(order.quantity, ask_order.quantity);
                        
                        trades.push(Trade {
                            price: ask_price, // Use the saved variable
                            quantity: trade_quantity,
                            buyer_order_id: order.order_id,
                            seller_order_id: ask_order.order_id,
                        });

                        order.quantity -= trade_quantity;
                        ask_order.quantity -= trade_quantity;

                        if ask_order.quantity == 0 {
                            self.order_map.remove(&ask_order.order_id);
                            orders_to_remove.push(i);
                        }
                        if order.quantity == 0 {
                            break;
                        }
                    }

                    for &i in orders_to_remove.iter().rev() {
                        asks_at_price.remove(i);
                    }
                    if asks_at_price.is_empty() {
                        entry.remove();
                    }
                    if order.quantity == 0 {
                        return trades;
                    }
                }

                self.order_map.insert(order.order_id, order);
                self.bids.entry(order.price).or_default().push(order);
            }
            Side::Sell => {
                while let Some(mut entry) = self.bids.last_entry() {
                    if order.price > *entry.key() {
                        break;
                    }
                    
                    // Get the price BEFORE the mutable borrow
                    let bid_price = *entry.key();
                    let bids_at_price = entry.get_mut();
                    let mut orders_to_remove = Vec::new();

                    for (i, bid_order) in bids_at_price.iter_mut().enumerate() {
                        let trade_quantity = std::cmp::min(order.quantity, bid_order.quantity);

                        trades.push(Trade {
                            price: bid_price, // Use the saved variable
                            quantity: trade_quantity,
                            buyer_order_id: bid_order.order_id,
                            seller_order_id: order.order_id,
                        });

                        order.quantity -= trade_quantity;
                        bid_order.quantity -= trade_quantity;

                        if bid_order.quantity == 0 {
                            self.order_map.remove(&bid_order.order_id);
                            orders_to_remove.push(i);
                        }
                        if order.quantity == 0 {
                            break;
                        }
                    }
                    
                    for &i in orders_to_remove.iter().rev() {
                        bids_at_price.remove(i);
                    }
                    if bids_at_price.is_empty() {
                        entry.remove();
                    }
                    if order.quantity == 0 {
                        return trades;
                    }
                }
                
                self.order_map.insert(order.order_id, order);
                self.asks.entry(order.price).or_default().push(order);
            }
        }
        trades
    }

    pub fn cancel_order(&mut self, order_id: u64) -> bool {
        if let Some(order) = self.order_map.remove(&order_id) {
            let price_level = match order.side {
                Side::Buy => self.bids.get_mut(&order.price),
                Side::Sell => self.asks.get_mut(&order.price),
            };

            if let Some(price_level) = price_level {
                if let Some(index) = price_level.iter().position(|o| o.order_id == order_id) {
                    price_level.remove(index);
                    if price_level.is_empty() {
                        match order.side {
                            Side::Buy => self.bids.remove(&order.price),
                            Side::Sell => self.asks.remove(&order.price),
                        };
                    }
                    return true;
                }
            }
        }
        false
    }
    // Add this function inside the `impl OrderBook` block in src/orderbook.rs

    pub fn display_depth(&self) {
        println!("\n--- ORDER BOOK DEPTH ---");
        println!("------------------------");

        // Print asks (lowest price first)
        println!("Side  | Price | Quantity");
        println!("------|-------|---------");
        for (price, orders) in self.asks.iter() {
            let total_quantity: u64 = orders.iter().map(|o| o.quantity).sum();
            println!("Ask   | {:<5} | {}", price, total_quantity);
        }

        // Print bids (highest price first)
        for (price, orders) in self.bids.iter().rev() {
            let total_quantity: u64 = orders.iter().map(|o| o.quantity).sum();
            println!("Bid   | {:<5} | {}", price, total_quantity);
        }
        println!("------------------------");
    }
}   