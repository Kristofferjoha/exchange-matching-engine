use crate::logging::logger_trait::SimLogger;
use crate::order::Order;
use crate::trade::Trade;
use chrono::{TimeZone, Utc};
use std::fs::File;
use std::io::{self, Write};
use uuid::Uuid;

/// A simple logger that writes formatted log messages directly to a file.
/// This is a "naive" implementation because each write operation is a blocking
/// system call, which can cause significant and unpredictable latency.
pub struct NaiveFileWriteLogger {
    writer: io::Result<File>,
}

impl NaiveFileWriteLogger {
    pub fn new(path: &str) -> Self {
        Self {
            writer: File::create(path),
        }
    }
}

impl SimLogger for NaiveFileWriteLogger {
    fn log_order_submission(&mut self, order: &Order) {
        if let Ok(writer) = &mut self.writer {
            let dt = Utc.timestamp_nanos(order.timestamp as i64);
            let _ = writeln!(
                writer,
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
    }

    fn log_trade(&mut self, trade: &Trade) {
        if let Ok(writer) = &mut self.writer {
            let dt = Utc.timestamp_nanos(trade.timestamp as i64);
            let _ = writeln!(
                writer,
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
    }

    fn log_order_cancel(&mut self, order_id: &Uuid, success: bool) {
        if let Ok(writer) = &mut self.writer {
            let dt = Utc::now();
            if success {
                let _ = writeln!(
                    writer,
                    "{} | ORDER CANCEL: id={} successfully cancelled",
                    dt.format("%Y-%m-%d %H:%M:%S%.3f"),
                    order_id
                );
            } else {
                let _ = writeln!(
                    writer,
                    "{} | ORDER CANCEL: id={} already filled",
                    dt.format("%Y-%m-%d %H:%M:%S%.3f"),
                    order_id
                );
            }
        }
    }

    fn log_order_filled(&mut self, order: &Order) {
        if let Ok(writer) = &mut self.writer {
            let dt = Utc::now();
            let _ = writeln!(
                writer,
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
    }

    fn finalize(mut self: Box<Self>) {
        if let Ok(writer) = &mut self.writer {
            let _ = writer.flush();
        }
    }
}
