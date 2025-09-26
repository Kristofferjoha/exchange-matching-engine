# Order Book Matching Engine
A high-performance order book matching engine written in Rust, designed specifically to benchmark and quantify the performance overhead of various low-latency logging strategies in High-Frequency Trading (HFT) systems.

# Overview
In latency-sensitive domains like HFT, every microsecond counts. While logging is crucial for debugging, auditing, and compliance, it is fundamentally an I/O-bound operation that can introduce significant, unpredictable latency spikes on the critical path.

This project provides a realistic simulation of a matching engine to serve as a testbed for different logging approaches. It demonstrates empirically how choosing the right logging strategy is not a minor detail but a critical architectural decision for any high-performance application.

# The Logging Benchmark
The primary goal of this project is to measure the precise latency impact of different logging implementations on the application's critical path.

### Example Log Output
The engine produces structured logs that clearly detail every event. Here is a sample:

```text
2025-09-26 16:05:19.541 | ORDER RECEIVED: id=3f74a20a-ce66-4796-8c78-7d1aec6491c9, instrument=PUMPTHIS, side=Buy, type=Limit, qty=67, price=99.25
2025-09-26 16:05:19.889 | ORDER RECEIVED: id=198738b4-c21b-48bc-9b54-74eb0e5e2600, instrument=PUMPTHIS, side=Buy, type=Limit, qty=22, price=98.75
2025-09-26 16:05:19.952 | ORDER RECEIVED: id=f8231017-8af3-474e-bc2a-a9db017ef446, instrument=PUMPTHIS, side=Buy, type=Limit, qty=50, price=98.50
2025-09-26 16:05:20.103 | ORDER RECEIVED: id=7bc076c5-1929-49e7-9386-8dc01dd64c91, instrument=PUMPTHIS, side=Sell, type=Limit, qty=89, price=98.7
2025-09-26 16:05:20.103 | TRADE EXECUTED: id=9ad5e9c4-3219-4f3f-a752-3feaee8f5426, instrument=PUMPTHIS, price=99.25, qty=67, taker_side=Sell, buy_order_id=3f74a20a-ce66-4796-8c78-7d1aec6491c9, sell_order_id=7bc076c5-1929-49e7-9386-8dc01dd64c91
2025-09-26 16:05:20.103 | TRADE EXECUTED: id=74427a2c-28ad-49ff-85c2-ed63a95e1d2f, instrument=PUMPTHIS, price=98.75, qty=22, taker_side=Sell, buy_order_id=198738b4-c21b-48bc-9b54-74eb0e5e2600, sell_order_id=7bc076c5-1929-49e7-9386-8dc01dd64c91
2025-09-26 16:05:20.103 | ORDER FILLED: id=3f74a20a-ce66-4796-8c78-7d1aec6491c9, instrument=PUMPTHIS, type=Limit, final_status=Filled, quantity=67, quantity_filled=67
2025-09-26 16:05:20.103 | ORDER FILLED: id=7bc076c5-1929-49e7-9386-8dc01dd64c91, instrument=PUMPTHIS, type=Limit, final_status=Filled, quantity=89, quantity_filled=89
2025-09-26 16:05:20.104 | ORDER RECEIVED: id=f689ec92-783d-4575-84f2-dbbf711b3e81, instrument=PUMPTHIS, side=Sell, type=Limit, qty=7, price=99.6
2025-09-26 16:05:20.104 | ORDER CANCEL: id=f8231017-8af3-474e-bc2a-a9db017ef446 successfully cancelled
```

## Strategies Tested
Seven distinct logging methods were benchmarked against a no-op baseline.

| Logging Mode | Description | 
| ----- | ----- | 
| `none` (Baseline) | No logging is performed. This measures the raw performance of the matching engine. | 
| `ae` (Async Enum) | Sends a lightweight enum variant over an MPSC channel to a dedicated logging thread for processing. | 
| `ac` (Async Closure) | Sends a closure over a channel to a logger thread, deferring all processing. | 
| `bfw` (Buffered) | A synchronous file writer wrapped in a std::io::BufWriter to reduce syscalls. | 
| `as` (Async String) | Formats a string on the critical path and sends it over a channel to a logger thread. | 
| `nfw` (Naive File) | A synchronous, unbuffered file write performed directly on the critical path for every event. | 
| `naive` (println!) | The simplest approach: blocking writes to standard output, which is notoriously slow. | 

## Methodology
Test Load: The simulation was run against a sequence of 100,000 order operations.

Critical Path: Latency was measured exclusively for the engine.process_order function call.

Environment: All benchmarks were compiled and run in release mode (cargo run --release) to enable full compiler optimizations.

## Results

| Logging Mode | Total Time | Mean Latency  | Median Latency | p99 Latency | p99.9 Latency | 
| ----- | ----- | ----- | ----- | ----- | ----- | 
| `none` (Baseline) | 49.37 ms | 476 ns | 200 ns | 2800 ns | 4700 ns | 
| `ae` (Async Enum) | 66.56 ms | 620 ns | 200 ns | 3900 ns| 9200 ns | 
| `ac` (Async Closure) | 72.21 ms | 667 ns | 200 ns | 4100 ns | 15000 ns | 
| `bfw` (Buffered) | 179.25 ms | 1501 ns | 200 ns | 12300 ns | 39600 ns | 
| `as` (Async String) | 208.50 ms | 1668 ns | 200 ns | 12400 ns | 18500 ns | 
| `nfw` (Naive File) | 4.58 s | 37184 ns | 300 ns | 304500 ns | 453300 ns | 
| `naive` (println!) | 11.09 s | 81673 ns | 1000 ns | 678000 ns | 941900 ns | 
