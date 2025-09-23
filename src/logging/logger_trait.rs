use crate::order::Order;
use crate::trade::Trade;
use uuid::Uuid;

pub trait SimLogger: Send {
    fn log_order_submission(&mut self, order: &Order);
    fn log_trade(&mut self, trade: &Trade);
    fn log_order_cancel(&mut self, order_id: &Uuid, success: bool);
    fn log_order_filled(&mut self, order: &Order);
    fn finalize(self: Box<Self>);
}