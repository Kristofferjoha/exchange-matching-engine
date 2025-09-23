use crate::logging::logger_trait::SimLogger;
use crate::logging::types::{LogMessage, OrderCancelLogData};
use crate::order::Order;
use crate::trade::Trade;
use chrono::{TimeZone, Utc};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use uuid::Uuid;

/// The final and most performant logger. It offloads all I/O and formatting
/// work to a background thread and avoids heap allocations on the critical path
/// by sending stack-allocated enums over the channel.
pub struct AsyncEnumLogger {
    sender: Sender<LogMessage>,
    handle: Option<JoinHandle<()>>,
}

impl AsyncEnumLogger {
    pub fn new(path: &str) -> Self {
        let (sender, receiver) = mpsc::channel::<LogMessage>();
        let path_owned = path.to_string();

        let handle = thread::spawn(move || {
            if let Ok(file) = File::create(&path_owned) {
                let mut writer = BufWriter::new(file);

                // The logging thread receives the enum and does all the formatting.
                for msg in receiver.iter() {
                    match msg {
                        LogMessage::OrderSubmission(order) => {
                            let dt = Utc.timestamp_nanos(order.timestamp as i64);
                            let _ = writeln!(writer,"{} | ORDER RECEIVED: id={}, instrument={}, side={:?}, type={:?}, qty={}, price={}",dt.format("%Y-%m-%d %H:%M:%S%.3f"),order.order_id,order.instrument,order.side,order.order_type,order.quantity,order.price.unwrap_or_default());
                        }
                        LogMessage::Trade(trade) => {
                            let dt = Utc.timestamp_nanos(trade.timestamp as i64);
                            let _ = writeln!(writer,"{} | TRADE EXECUTED: id={}, instrument={}, price={}, qty={}, taker_side={:?}, buy_order_id={}, sell_order_id={}",dt.format("%Y-%m-%d %H:%M:%S%.3f"),trade.trade_id,trade.instrument,trade.price,trade.quantity,trade.taker_side,trade.buy_order_id,trade.sell_order_id);
                        }
                        LogMessage::OrderCancel(data) => {
                            let dt = Utc::now();
                            let status = if data.success { "successfully cancelled" } else { "already filled" };
                            let _ = writeln!(writer,"{} | ORDER CANCEL: id={} {}",dt.format("%Y-%m-%d %H:%M:%S%.3f"),data.order_id,status);
                        }
                        LogMessage::OrderFilled(order) => {
                            let dt = Utc::now();
                            let _ = writeln!(writer,"{} | ORDER FILLED: id={}, instrument={}, type={:?}, final_status={:?}, quantity={}, quantity_filled={}",dt.format("%Y-%m-%d %H:%M:%S%.3f"),order.order_id,order.instrument,order.order_type,order.status,order.quantity,order.quantity - order.remaining_quantity);
                        }
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

impl SimLogger for AsyncEnumLogger {
    // The log methods now create a lightweight enum variant and send it.
    // This is extremely fast as it avoids heap allocation (`Box`).
    fn log_order_submission(&mut self, order: &Order) {
        let _ = self
            .sender
            .send(LogMessage::OrderSubmission(order.clone()));
    }

    fn log_trade(&mut self, trade: &Trade) {
        let _ = self.sender.send(LogMessage::Trade(trade.clone()));
    }

    fn log_order_cancel(&mut self, order_id: &Uuid, success: bool) {
        let data = OrderCancelLogData {
            order_id: *order_id,
            success,
        };
        let _ = self.sender.send(LogMessage::OrderCancel(data));
    }

    fn log_order_filled(&mut self, order: &Order) {
        let _ = self.sender.send(LogMessage::OrderFilled(order.clone()));
    }

    fn finalize(mut self: Box<Self>) {
        drop(self.sender);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

