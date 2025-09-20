use exchange_matching_engine::engine::MatchingEngine;
use exchange_matching_engine::order::Order;
use exchange_matching_engine::utils::{MatchingEngineError, Side};
use rust_decimal_macros::dec;
use uuid::Uuid;

fn setup() -> MatchingEngine {
    let mut engine = MatchingEngine::new();
    engine.add_market("SOFI".to_string());
    engine
}

#[test]
fn test_add_non_matching_limit_order() {
    let mut engine = setup();
    let order = Order::new_limit("SOFI".to_string(), Side::Buy, dec!(100.0), dec!(10));

    let trades = engine.process_order(order).unwrap();
    assert!(trades.is_empty());

    let book = engine.get_order_book_display("SOFI").unwrap();
    assert_eq!(book.bids.len(), 1);
    assert_eq!(book.asks.len(), 0);
    assert_eq!(book.bids[0].price, dec!(100.0));
    assert_eq!(book.bids[0].volume, dec!(10));
}

#[test]
fn test_simple_full_match() {
    let mut engine = setup();
    
    let sell_order = Order::new_limit("SOFI".to_string(), Side::Sell, dec!(102.5), dec!(5));
    engine.process_order(sell_order).unwrap();

    let buy_order = Order::new_limit("SOFI".to_string(), Side::Buy, dec!(102.5), dec!(5));
    let trades = engine.process_order(buy_order).unwrap();

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].price, dec!(102.5));
    assert_eq!(trades[0].quantity, dec!(5));

    let book = engine.get_order_book_display("SOFI").unwrap();
    assert!(book.bids.is_empty());
    assert!(book.asks.is_empty());
}

#[test]
fn test_partial_match() {
    let mut engine = setup();

    let sell_order = Order::new_limit("SOFI".to_string(), Side::Sell, dec!(200.0), dec!(10));
    engine.process_order(sell_order).unwrap();

    let buy_order = Order::new_limit("SOFI".to_string(), Side::Buy, dec!(200.0), dec!(3));
    let trades = engine.process_order(buy_order).unwrap();

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, dec!(3));

    let book = engine.get_order_book_display("SOFI").unwrap();
    assert!(book.bids.is_empty());
    assert_eq!(book.asks.len(), 1);
    assert_eq!(book.asks[0].volume, dec!(7));
}

#[test]
fn test_match_across_multiple_price_levels() {
    let mut engine = setup();

    engine.process_order(Order::new_limit("SOFI".to_string(), Side::Sell, dec!(102.0), dec!(10))).unwrap();
    engine.process_order(Order::new_limit("SOFI".to_string(), Side::Sell, dec!(101.0), dec!(5))).unwrap();

    let buy_order = Order::new_limit("SOFI".to_string(), Side::Buy, dec!(103.0), dec!(12));
    let trades = engine.process_order(buy_order).unwrap();

    assert_eq!(trades.len(), 2);
    assert_eq!(trades[0].price, dec!(101.0));
    assert_eq!(trades[0].quantity, dec!(5));
    assert_eq!(trades[1].price, dec!(102.0));
    assert_eq!(trades[1].quantity, dec!(7));

    let book = engine.get_order_book_display("SOFI").unwrap();
    assert!(book.bids.is_empty());
    assert_eq!(book.asks.len(), 1);
    assert_eq!(book.asks[0].price, dec!(102.0));
    assert_eq!(book.asks[0].volume, dec!(3));
}


#[test]
fn test_price_time_priority_fifo() {
    let mut engine = setup();

    let sell_order_first = Order::new_limit("SOFI".to_string(), Side::Sell, dec!(100.0), dec!(5));
    let first_order_id = sell_order_first.order_id;
    engine.process_order(sell_order_first).unwrap();

    let sell_order_second = Order::new_limit("SOFI".to_string(), Side::Sell, dec!(100.0), dec!(5));
    engine.process_order(sell_order_second).unwrap();

    let buy_order = Order::new_limit("SOFI".to_string(), Side::Buy, dec!(100.0), dec!(5));
    let trades = engine.process_order(buy_order).unwrap();

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].sell_order_id, first_order_id);
    assert_eq!(trades[0].quantity, dec!(5));
    assert_eq!(trades[0].price, dec!(100.0));
    let book = engine.get_order_book_display("SOFI").unwrap();
    assert!(book.bids.is_empty());
    assert_eq!(book.asks.len(), 1);
    assert_eq!(book.asks[0].price, dec!(100.0));
}


#[test]
fn test_cancel_partially_filled_order() {
    let mut engine = setup();

    let sell_order = Order::new_limit("SOFI".to_string(), Side::Sell, dec!(200.0), dec!(10));
    let sell_order_id = sell_order.order_id;
    engine.process_order(sell_order).unwrap();
    engine.process_order(Order::new_limit("SOFI".to_string(), Side::Buy, dec!(200.0), dec!(4))).unwrap();

    let result = engine.cancel_order_by_id(&sell_order_id, "SOFI");
    
    assert!(result.is_ok());
    let book = engine.get_order_book_display("SOFI").unwrap();
    assert!(book.asks.is_empty());
}

#[test]
fn test_cancel_non_existent_order() {
    let mut engine = setup();
    let random_id = Uuid::new_v4();
    
    let result = engine.cancel_order_by_id(&random_id, "SOFI");

    assert!(result.is_err());
    matches!(result.unwrap_err(), MatchingEngineError::OrderNotFound(id) if id == random_id);
}


#[test]
fn test_market_order_insufficient_liquidity() {
    let mut engine = setup();
    
    engine.process_order(Order::new_limit("SOFI".to_string(), Side::Sell, dec!(100.0), dec!(5))).unwrap();

    let market_buy = Order::new_market("SOFI".to_string(), Side::Buy, dec!(10));
    let trades = engine.process_order(market_buy).unwrap();
    
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity, dec!(5));

    let book = engine.get_order_book_display("SOFI").unwrap();
    assert!(book.asks.is_empty());
    assert!(book.bids.is_empty(), "Unfilled part of market order should not be in the book");
}

#[test]
fn test_process_order_for_unknown_market() {
    let mut engine = MatchingEngine::new();
    let order = Order::new_limit("UNKNOWN".to_string(), Side::Buy, dec!(10.0), dec!(1));

    let result = engine.process_order(order);

    assert!(result.is_err());
    matches!(result.unwrap_err(), MatchingEngineError::MarketNotFound(market) if market == "UNKNOWN");
}
