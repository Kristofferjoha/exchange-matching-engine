use exchange_matching_engine::engine::MatchingEngine;
use exchange_matching_engine::order::{Order};
use exchange_matching_engine::utils::Side;
use rust_decimal_macros::dec;

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