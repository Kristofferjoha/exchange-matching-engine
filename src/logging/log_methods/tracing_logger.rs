use crate::logging::logger_trait::SimLogger;
use crate::order::Order;
use crate::trade::Trade;
use chrono::{TimeZone, Utc};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use uuid::Uuid;

pub struct TracingLogger {

    _guard: Option<WorkerGuard>,
}

impl TracingLogger {
    pub fn new(guard: Option<WorkerGuard>) -> Self {
        Self { _guard: guard }
    }
}

impl SimLogger for TracingLogger {
    fn log_order_submission(&mut self, order: &Order) {
        let dt = Utc.timestamp_nanos(order.timestamp as i64);
        info!(
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
        info!(
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
        let status_msg = if success {
            "successfully cancelled"
        } else {
            "already filled"
        };
        info!(
            "{} | ORDER CANCEL: id={} {}",
            dt.format("%Y-%m-%d %H:%M:%S%.3f"),
            order_id,
            status_msg
        );
    }

    fn log_order_filled(&mut self, order: &Order) {
        let dt = Utc::now();
        info!(
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

    fn finalize(self: Box<Self>) {
    }
}
