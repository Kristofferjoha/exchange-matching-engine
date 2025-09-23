use crate::logging::logger_trait::SimLogger;
use crate::order::Order;
use crate::trade::Trade;
use chrono::{TimeZone, Utc};
use uuid::Uuid;

/// A simple logger that prints formatted log messages directly to the console
/// using the `println!` macro. This is a "naive" implementation that can
/// introduce significant, unpredictable latency.
pub struct PrintlnLogger;

impl SimLogger for PrintlnLogger {
    fn log_order_submission(&mut self, order: &Order) {
        let dt = Utc.timestamp_nanos(order.timestamp as i64);
        println!(
            "{} | ORDER RECEIVED: id={}, instrument={}, side={:?}, type={:?}, qty={}, price={}",
            dt.format("%Y-%m-%d %H:%M:%S%.3f"),
            order.order_id,
            order.instrument,
            order.side,
            order.order_type,
            order.quantity,
            order.price.unwrap_or_default()
        );
    }

    fn log_trade(&mut self, trade: &Trade) {
        let dt = Utc.timestamp_nanos(trade.timestamp as i64);
        println!(
            "{} | TRADE EXECUTED: id={}, instrument={}, price={}, qty={}, taker_side={:?}, buy_order_id={}, sell_order_id={}",
            dt.format("%Y-%m-%d %H:%M:%S%.3f"),
            trade.trade_id,
            trade.instrument,
            trade.price,
            trade.quantity,
            trade.taker_side,
            trade.buy_order_id,
            trade.sell_order_id
        );
    }

    fn log_order_cancel(&mut self, order_id: &Uuid, success: bool) {
        let dt = Utc::now();
        if success {
            println!(
                "{} | ORDER CANCEL: id={} successfully cancelled",
                dt.format("%Y-%m-%d %H:%M:%S%.3f"),
                order_id
            );
        } else {
            println!(
                "{} | ORDER CANCEL: id={} already filled",
                dt.format("%Y-%m-%d %H:%M:%S%.3f"),
                order_id
            );
        }
    }

    fn log_order_filled(&mut self, order: &Order) {
        let dt = Utc::now();
        println!(
            "{} | ORDER FILLED: id={}, instrument={}, type={:?}, final_status={:?}, quantity={}, quantity_filled={}",
            dt.format("%Y-%m-%d %H:%M:%S%.3f"),
            order.order_id,
            order.instrument,
            order.order_type,
            order.status,
            order.quantity,
            order.quantity - order.remaining_quantity
        );
    }

    fn finalize(self: Box<Self>) {}
}
