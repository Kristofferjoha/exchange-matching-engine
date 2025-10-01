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
Nine distinct logging methods were benchmarked against a no-op baseline.

| Logging Mode | Description | 
| ----- | ----- | 
| `none` (Baseline) | No logging is performed. This measures the raw performance of the matching engine. | 
| `ae` (Async Enum) | Sends a lightweight enum variant over an MPSC channel to a dedicated logging thread for processing, minimizing critical path overhead. | 
| `ac` (Async Closure) | Sends a closure over a channel to a logger thread, deferring all processing for low latency. | 
| `bfw` (Buffered) | A synchronous file writer wrapped in a std::io::BufWriter to reduce syscalls, balancing latency and persistence | 
| `as` (Async String) | Formats a string on the critical path and sends it over a channel to a logger thread, suitable for structured logging with moderate overhead. | 
| `nfw` (Naive File) | A synchronous, unbuffered file write performed directly on the critical path, high latency but persistent. | 
| `tf` (Tracing File) | Uses the tracing crate with a non-blocking file appender to log events to a file, high overhead but structured. | 
| `tc` (Tracing Console) | Uses the tracing crate to log events to stdout with structured formatting, high overhead. | 
| `naive` (println!) | The simplest approach: blocking writes to standard output, which is notoriously slow. | 

## Methodology
Test Load: The simulation was run against a sequence of 1000000 order operations (NEW and CANCEL) from operations.csv, averaged over 10 runs per logging mode.

Critical Path: Latency was measured for both processing (engine.process_order, cancel_order_by_id) and logging operations, with a focus on logging latency to evaluate overhead.

Metrics: Mean, median, 99th percentile (P99), and 99.9th percentile (P999) latencies were calculated for logging operations. Total time is the average runtime across 10 runs for 1000000 operations, including processing, logging, and overheads.

To run the simulation, use cargo run --release and then logging version you want to use, fx "ae"

## Results

Certainly. Here is the table in that specific format.

Of course, here is the data formatted as a Markdown table.

| Logging Mode | Total Time | Mean Latency | Median Latency | p99 Latency | p99.9 Latency |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Baseline | 0.06 s | 0.046 µs | 0.000 µs | 0.200 µs | 0.300 µs |
| AsyncEnum | 0.08 s | 0.204 µs | 0.100 µs | 1.200 µs | 3.800 µs |
| AsyncClosure | 0.08 s | 0.236 µs | 0.100 µs | 1.300 µs | 10.700 µs |
| BufferedFileWrite | 0.18 s | 1.246 µs | 0.400 µs | 10.000 µs | 28.900 µs |
| AsyncString | 0.22 s | 1.625 µs | 0.800 µs | 10.700 µs | 18.400 µs |
| NaiveFileWrite | 5.01 s | 49.273 µs | 20.600 µs | 339.500 µs | 513.300 µs |
| TracingFile | 11.00 s | 108.242 µs | 45.500 µs | 686.500 µs | 977.800 µs |
| TracingConsole | 11.14 s | 109.525 µs | 45.900 µs | 694.300 µs | 982.900 µs |
| Naive | 11.52 s | 113.242 µs | 47.800 µs | 725.700 µs | 1058.400 µs |