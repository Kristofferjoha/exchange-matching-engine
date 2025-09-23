use crate::logging::logger_trait::SimLogger;
use crate::order::Order;
use crate::trade::Trade;
use chrono::{TimeZone, Utc};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use uuid::Uuid;
/// An asynchronous logger that performs string formatting on the main thread
/// but sends the resulting string to a dedicated background thread for file I/O.
/// This decouples the main application from slow, blocking disk writes.
pub struct AsyncStringLogger {
    sender: Sender<String>,
    handle: Option<JoinHandle<()>>,
}

impl AsyncStringLogger {
    pub fn new(path: &str) -> Self {
        let (sender, receiver) = mpsc::channel::<String>();

        let path_owned = path.to_string();

        let handle = thread::spawn(move || {
            if let Ok(file) = File::create(&path_owned) {
                let mut writer = BufWriter::new(file);

                for msg in receiver.iter() {
                    if writeln!(&mut writer, "{}", msg).is_err() {
                        break;
                    }
                }
                let _ = writer.flush();
            } else {
                eprintln!("Failed to create log file: {}", path_owned);
            }
        });

        Self {
            sender,
            handle: Some(handle),
        }
    }
}

impl SimLogger for AsyncStringLogger {
    fn log_order_submission(&mut self, order: &Order) {
        let dt = Utc.timestamp_nanos(order.timestamp as i64);
        let msg = format!(
            "{} | ORDER RECEIVED: id={}, instrument={}, side={:?}, type={:?}, qty={}, price={}",
            dt.format("%Y-%m-%d %H:%M:%S%.3f"),
            order.order_id,
            order.instrument,
            order.side,
            order.order_type,
            order.quantity,
            order.price.unwrap_or_default()
        );
        let _ = self.sender.send(msg);
    }

    fn log_trade(&mut self, trade: &Trade) {
        let dt = Utc.timestamp_nanos(trade.timestamp as i64);
        let msg = format!(
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
        let _ = self.sender.send(msg);
    }

    fn log_order_cancel(&mut self, order_id: &Uuid, success: bool) {
        let dt = Utc::now();
        let status = if success {
            "successfully cancelled"
        } else {
            "already filled"
        };
        let msg = format!(
            "{} | ORDER CANCEL: id={} {}",
            dt.format("%Y-%m-%d %H:%M:%S%.3f"),
            order_id,
            status
        );
        let _ = self.sender.send(msg);
    }

    fn log_order_filled(&mut self, order: &Order) {
        let dt = Utc::now();
        let msg = format!(
            "{} | ORDER FILLED: id={}, instrument={}, type={:?}, final_status={:?}, quantity={}, quantity_filled={}",
            dt.format("%Y-%m-%d %H:%M:%S%.3f"),
            order.order_id,
            order.instrument,
            order.order_type,
            order.status,
            order.quantity,
            order.quantity - order.remaining_quantity
        );
        let _ = self.sender.send(msg);
    }

    fn finalize(mut self: Box<Self>) {
        drop(self.sender);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
