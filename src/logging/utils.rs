use crate::order::Order;
use crate::trade::Trade;
use chrono::{TimeZone, Utc};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::str::FromStr;
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggingMode {
    Baseline,
    Naive,
    NaiveFileWrite,
    BufferedFileWrite,
    AsyncString,
    AsyncClosure,
}

impl FromStr for LoggingMode {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "baseline" => Ok(Self::Baseline),
            "println" | "naive" => Ok(Self::Naive),
            "naivefilewrite" | "nfw" => Ok(Self::NaiveFileWrite),
            "bufferedfilewrite" | "bfw" => Ok(Self::BufferedFileWrite),
            "asyncstring" | "as" => Ok(Self::AsyncString),
            "asyncclosure" | "ac" => Ok(Self::AsyncClosure),
            _ => Err("Unknown logging mode"),
        }
    }
}

pub trait SimLogger: Send {
    fn log_order_submission(&mut self, order: &Order);
    fn log_trade(&mut self, trade: &Trade);
    fn log_order_cancel(&mut self, order_id: &Uuid, success: bool);
    fn log_order_filled(&mut self, order: &Order);
    fn finalize(self: Box<Self>);
}

pub struct NoOpLogger;

impl SimLogger for NoOpLogger {
    fn log_order_submission(&mut self, _order: &Order) {}
    fn log_trade(&mut self, _trade: &Trade) {}
    fn log_order_cancel(&mut self, _order_id: &Uuid, _success: bool) {}
    fn log_order_filled(&mut self, _order: &Order) {}
    fn finalize(self: Box<Self>) {}
}

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

pub struct NaiveFileWriteLogger {
    writer: io::Result<File>,
}

impl NaiveFileWriteLogger {
    fn new(path: &str) -> Self {
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

pub struct BufferedFileWriteLogger {
    writer: io::Result<BufWriter<File>>,
}

impl BufferedFileWriteLogger {
    fn new(path: &str) -> Self {
        let file = File::create(path);
        Self {
            writer: file.map(BufWriter::new),
        }
    }
}

impl SimLogger for BufferedFileWriteLogger {
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

pub struct AsyncStringLogger {
    sender: Sender<String>,
    handle: Option<JoinHandle<()>>,
}

impl AsyncStringLogger {
    pub fn new(path: &str) -> Self {
        let (sender, receiver) = mpsc::channel::<String>();

        let path_owned = path.to_string();

        let handle = thread::spawn(move || {
            let mut writer = BufferedFileWriteLogger::new(&path_owned);

            for msg in receiver.iter() {
                if let Ok(w) = &mut writer.writer {
                    if writeln!(w, "{}", msg).is_err() {
                        break;
                    }
                }
            }

            Box::new(writer).finalize();
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
        let status = if success { "successfully cancelled" } else { "already filled" };
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

type LogClosure = Box<dyn FnOnce(&mut BufWriter<File>) + Send>;

pub struct AsyncClosureLogger {
    sender: Sender<LogClosure>,
    handle: Option<JoinHandle<()>>,
}

impl AsyncClosureLogger {
    pub fn new(path: &str) -> Self {
        let (sender, receiver) = mpsc::channel::<LogClosure>();
        let path_owned = path.to_string();

        let handle = thread::spawn(move || {
            let file = File::create(&path_owned).expect("Failed to create log file");
            let mut writer = BufWriter::new(file);

            for log_closure in receiver.iter() {
                log_closure(&mut writer);
            }
            let _ = writer.flush();
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
            let status = if success { "successfully cancelled" } else { "already filled" };
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


pub fn create_logger(mode: LoggingMode) -> Box<dyn SimLogger> {
    match mode {
        LoggingMode::Baseline => Box::new(NoOpLogger),
        LoggingMode::Naive => Box::new(PrintlnLogger),
        LoggingMode::NaiveFileWrite => Box::new(NaiveFileWriteLogger::new("naive_output.log")),
        LoggingMode::BufferedFileWrite => {
            Box::new(BufferedFileWriteLogger::new("buffered_output.log"))
        }
        LoggingMode::AsyncString => Box::new(AsyncStringLogger::new("async_string_output.log")),
        LoggingMode::AsyncClosure => Box::new(AsyncClosureLogger::new("async_closure_output.log")),
    }
}