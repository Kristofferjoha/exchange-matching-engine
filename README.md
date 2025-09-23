# Order-Book-Matching-Engine

| Logging Mode | Total Time | Mean Latency  | Median Latency | p99 Latency | p99.9 Latency | 
| ----- | ----- | ----- | ----- | ----- | ----- | 
| `none` (Baseline) | 49.37 ms | 476 ns | 200 ns | 2800 ns | 4700 ns | 
| `ae` (Async Enum) | 66.56 ms | 620 ns | 200 ns | 3900 ns| 9200 ns | 
| `ac` (Async Closure) | 72.21 ms | 667 ns | 200 ns | 4100 ns | 15000 ns | 
| `bfw` (Buffered) | 179.25 ms | 1501 | 200 ns | 12300 ns | 39600 ns | 
| `as` (Async String) | 208.50 ms | 1668 ns | 200 ns | 12400 ns | 18500 ns | 
| `nfw` (Naive File) | 4.58 s | 37184 ns | 300 ns | 304500 ns | 453300 ns | 
| `naive` (println!) | 11.09 s | 81673 ns | 1000 ns | 678000 ns | 941900 ns | 