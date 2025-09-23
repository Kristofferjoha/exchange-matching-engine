use crate::order::Order;
use crate::trade::Trade;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggingMode {
    Baseline,
    Naive,
    NaiveFileWrite,
    BufferedFileWrite,
    AsyncString,
    AsyncClosure,
    AsyncEnum,
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
            "asyncenum" | "ae" => Ok(Self::AsyncEnum),
            _ => Err("Unknown logging mode"),
        }
    }
}

#[derive(Clone)]
pub struct OrderCancelLogData {
    pub order_id: Uuid,
    pub success: bool,
}

#[derive(Clone)]
pub enum LogMessage {
    OrderSubmission(Order),
    Trade(Trade),
    OrderCancel(OrderCancelLogData),
    OrderFilled(Order),
}
