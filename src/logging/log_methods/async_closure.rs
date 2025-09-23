use crate::logging::logger_trait::SimLogger;
use crate::order::Order;
use crate::trade::Trade;
use chrono::{TimeZone, Utc};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use uuid::Uuid;

type LogClosure = Box<dyn FnOnce(&mut BufWriter<File>) + Send>;

/// An advanced asynchronous logger that offloads both I/O and string formatting.
/// It works by sending a closure (the "instructions" for logging) to a
/// dedicated background thread, which then executes the closure to perform
/// the expensive work away from the main application thread.
pub struct AsyncClosureLogger {
    sender: Sender<LogClosure>,
    handle: Option<JoinHandle<()>>,
}

impl AsyncClosureLogger {
    pub fn new(path: &str) -> Self {
        let (sender, receiver) = mpsc::channel::<LogClosure>();
        let path_owned = path.to_string();

        let handle = thread::spawn(move || {
            if let Ok(file) = File::create(&path_owned) {
                let mut writer = BufWriter::new(file);

                for log_closure in receiver.iter() {
                    log_closure(&mut writer);
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

impl SimLogger for AsyncClosureLogger {
    fn log_order_submission(&mut self, order: &Order) {
        let order_data = order.clone();
        let log_closure = move |writer: &mut BufWriter<File>| {
            let dt = Utc.timestamp_nanos(order_data.timestamp as i64);
            let _ = writeln!(
                writer,
                "{} | ORDER RECEIVED: id={}, instrument={}, side={:?}, type={:?}, qty={}, price={}",
                dt.format("%Y-%m-%d %H:%M:%S%.3f"),
                order_data.order_id,
                order_data.instrument,
                order_data.side,
                order_data.order_type,
                order_data.quantity,
                order_data.price.unwrap_or_default()
            );
        };
        let _ = self.sender.send(Box::new(log_closure));
    }

    fn log_trade(&mut self, trade: &Trade) {
        let trade_data = trade.clone();
        let log_closure = move |writer: &mut BufWriter<File>| {
            let dt = Utc.timestamp_nanos(trade_data.timestamp as i64);
            let _ = writeln!(
                writer,
                "{} | TRADE EXECUTED: id={}, instrument={}, price={}, qty={}, taker_side={:?}, buy_order_id={}, sell_order_id={}",
                dt.format("%Y-%m-%d %H:%M:%S%.3f"),
                trade_data.trade_id,
                trade_data.instrument,
                trade_data.price,
                trade_data.quantity,
                trade_data.taker_side,
                trade_data.buy_order_id,
                trade_data.sell_order_id
            );
        };
        let _ = self.sender.send(Box::new(log_closure));
    }

    fn log_order_cancel(&mut self, order_id: &Uuid, success: bool) {
        let order_id_data = *order_id;
        let log_closure = move |writer: &mut BufWriter<File>| {
            let dt = Utc::now();
            let status = if success {
                "successfully cancelled"
            } else {
                "already filled"
            };
            let _ = writeln!(
                writer,
                "{} | ORDER CANCEL: id={} {}",
                dt.format("%Y-%m-%d %H:%M:%S%.3f"),
                order_id_data,
                status
            );
        };
        let _ = self.sender.send(Box::new(log_closure));
    }

    fn log_order_filled(&mut self, order: &Order) {
        let order_data = order.clone();
        let log_closure = move |writer: &mut BufWriter<File>| {
            let dt = Utc::now();
            let _ = writeln!(
                writer,
                "{} | ORDER FILLED: id={}, instrument={}, type={:?}, final_status={:?}, quantity={}, quantity_filled={}",
                dt.format("%Y-%m-%d %H:%M:%S%.3f"),
                order_data.order_id,
                order_data.instrument,
                order_data.order_type,
                order_data.status,
                order_data.quantity,
                order_data.quantity - order_data.remaining_quantity
            );
        };
        let _ = self.sender.send(Box::new(log_closure));
    }

    fn finalize(mut self: Box<Self>) {
        drop(self.sender);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
