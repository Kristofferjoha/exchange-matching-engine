use crate::order::{Order};
use crate::trade::{Trade};
use crate::utils::{OrderType, Side, OrderStatus};
use std::collections::{BTreeMap, HashMap};

    // try to match order
    // process market order
    // process limit order
    // add order to orderbook
    // cancel order
    // display orderbook depth

pub struct OrderBook {
    instrument: String,
    bids: BTreeMap<u64, Vec<Order>>,
    asks: BTreeMap<u64, Vec<Order>>,
    order_map: HashMap<u64, Order>,
}

impl OrderBook {
    pub fn new(instrument: String) -> Self {
        OrderBook {
            instrument,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_map: HashMap::new(),
        }
    }

}