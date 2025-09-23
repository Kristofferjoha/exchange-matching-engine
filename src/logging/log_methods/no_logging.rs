use crate::logging::logger_trait::SimLogger;
use crate::order::Order;
use crate::trade::Trade;
use uuid::Uuid;

/// A no-operation logger that implements the `SimLogger` trait but performs no actions.
/// This will serve as a baseline for performance comparisons.
pub struct NoOpLogger;

impl SimLogger for NoOpLogger {
    fn log_order_submission(&mut self, _order: &Order) {}
    fn log_trade(&mut self, _trade: &Trade) {}
    fn log_order_cancel(&mut self, _order_id: &Uuid, _success: bool) {}
    fn log_order_filled(&mut self, _order: &Order) {}
    fn finalize(self: Box<Self>) {}
}
