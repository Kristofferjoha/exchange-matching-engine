use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::str::FromStr;
use crate::order::Order;
use crate::trade::Trade;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggingMode {
    Baseline,
    Naive,
    NaiveFileWrite,
    BufferedFileWrite,
}

impl FromStr for LoggingMode {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "baseline" => Ok(Self::Baseline),
            "println" | "naive" => Ok(Self::Naive),
            "naivefilewrite" => Ok(Self::NaiveFileWrite),
            "bufferedfilewrite" | "bfw" => Ok(Self::BufferedFileWrite),
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
        println!(
            "{} | ORDER RECEIVED: id={}, instrument={}, side={:?}, type={:?}, qty={}, price={}",
            order.timestamp,
            order.order_id,
            order.instrument,
            order.side,
            order.order_type,
            order.quantity,
            order.price.unwrap_or_default()
        );
    }
    fn log_trade(&mut self, trade: &Trade) {
        println!(
            "{} | TRADE EXECUTED: id={}, instrument={}, price={}, qty={}, taker_side={:?}, buy_order_id={}, sell_order_id={}",
            trade.timestamp,
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
        let ts = chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        if success {
            println!(
                "{} | ORDER CANCEL: id={} successfully cancelled", 
                ts,
                order_id
            );
        } else {
            println!(
                "{} | ORDER CANCEL: id={} already filled", 
                ts,
                order_id,
            );
        }

    }
    fn log_order_filled(&mut self, order: &Order) {
        let ts = chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        println!(
            "{} | ORDER FILLED: id={}, instrument={}, type={:?}, final_status={:?}, quantity={}, quantity_filled={}",
            ts,
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
            let _ = writeln!(
                writer,
                "{} | ORDER RECEIVED: id={}, instrument={}, side={:?}, type={:?}, qty={}, price={}",
                order.timestamp,
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
            let _ = writeln!(
                writer,
                "{} | TRADE EXECUTED: id={}, instrument={}, price={}, qty={}, taker_side={:?}, buy_order_id={}, sell_order_id={}",
                trade.timestamp,
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
        let ts = chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        if let Ok(writer) = &mut self.writer {
            if success {
                let _ = writeln!(
                    writer,
                    "{} | ORDER CANCEL: id={} successfully cancelled",
                    ts,
                    order_id
                );
            } else {
                let _ = writeln!(
                    writer,
                    "{} | ORDER CANCEL: id={} already filled",
                    ts,
                    order_id
                );
            }
        }
    }
        fn log_order_filled(&mut self, order: &Order) {
        let ts = chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        if let Ok(writer) = &mut self.writer {
            let _ = writeln!(
                writer,
                "{} | ORDER FILLED: id={}, instrument={}, type={:?}, final_status={:?}, quantity={}, quantity_filled={}",
                ts,
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
            let _ = writeln!(
                writer,
                "{} | ORDER RECEIVED: id={}, instrument={}, side={:?}, type={:?}, qty={}, price={}",
                order.timestamp,
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
            let _ = writeln!(
                writer,
                "{} | TRADE EXECUTED: id={}, instrument={}, price={}, qty={}, taker_side={:?}, buy_order_id={}, sell_order_id={}",
                trade.timestamp,
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
        let ts = chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        if let Ok(writer) = &mut self.writer {
            if success {
                let _ = writeln!(
                    writer,
                    "{} | ORDER CANCEL: id={} successfully cancelled",
                    ts, order_id
                );
            } else {
                let _ = writeln!(
                    writer,
                    "{} | ORDER CANCEL: id={} already filled",
                    ts, order_id
                );
            }
        }
    }

    fn log_order_filled(&mut self, order: &Order) {
        let ts = chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
        if let Ok(writer) = &mut self.writer {
            let _ = writeln!(
                writer,
                "{} | ORDER FILLED: id={}, instrument={}, type={:?}, final_status={:?}, quantity={}, quantity_filled={}",
                ts,
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
            // This flush is crucial for buffered writers to ensure all data
            // in the buffer is written to the file before the program exits.
            let _ = writer.flush();
        }
    }
}


pub fn create_logger(mode: LoggingMode) -> Box<dyn SimLogger> {
    match mode {
        LoggingMode::Baseline => Box::new(NoOpLogger),
        LoggingMode::Naive => Box::new(PrintlnLogger),
        LoggingMode::NaiveFileWrite => Box::new(NaiveFileWriteLogger::new("naive_output.log")),
        LoggingMode::BufferedFileWrite => Box::new(BufferedFileWriteLogger::new("buffered_output.log")),
    }
}