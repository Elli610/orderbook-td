# High-Performance Order Book With Rust

A competitive programming challenge to build the fastest possible order book data structure in Rust. The goal is to achieve sub-nanosecond operations for critical hot-path functions.

## Objective

Complete the `OrderBook` trait implementation in `src/orderbook.rs` and optimize it for maximum performance. The faster your implementation, the better!

## What is an Order Book?

An order book is a fundamental data structure in financial trading systems that maintains:
- **Bids**: Buy orders sorted by price (highest first)
- **Asks**: Sell orders sorted by price (lowest first)
- **Price levels**: Each price point with its associated quantity

## 🏗️ Project Structure

```
src/
├── main.rs          # Entry point with benchmarks and tests
├── interfaces.rs    # OrderBook trait and type definitions
├── orderbook.rs     # Your implementation goes here (currently TODO)
└── benchmarks.rs    # Comprehensive benchmarking framework
```

## 🔧 Implementation Requirements

Implement the `OrderBook` trait with the following methods:

### Core Operations (HOT PATH - Optimize heavily!)
- `apply_update(&mut self, update: Update)` - Add/update/remove price levels
- `get_spread(&self) -> Option<Price>` - Calculate bid-ask spread

### Query Operations
- `get_best_bid(&self) -> Option<Price>` - Get highest bid price
- `get_best_ask(&self) -> Option<Price>` - Get lowest ask price
- `get_quantity_at(&self, price: Price, side: Side) -> Option<Quantity>`
- `get_top_levels(&self, side: Side, n: usize) -> Vec<(Price, Quantity)>`
- `get_total_quantity(&self, side: Side) -> Quantity`

## Getting Started

1. **Clone and setup**:
   ```bash
   cargo build --release
   ```

2. **Implement the trait** in `src/orderbook.rs`:
   ```rust
   use std::collections::BTreeMap;

   pub struct OrderBookImpl {
       bids: BTreeMap<Price, Quantity>,
       asks: BTreeMap<Price, Quantity>,
   }

   impl OrderBook for OrderBookImpl {
       fn new() -> Self {
           OrderBookImpl {
               bids: BTreeMap::new(),
               asks: BTreeMap::new(),
           }
       }
       // ... implement other methods
   }
   ```

3. **Run tests** to verify correctness:
   ```bash
   cargo test
   ```

4. **Run benchmarks** to measure performance:
   ```bash
   cargo run --release
   ```

## 📊 Benchmark Metrics

The benchmark suite measures:
- **Update operations** (avg, P50, P95, P99)
- **Get best bid/ask** latency
- **Spread calculation** latency
- **Random reads** performance
- **Total operations**: 100,000 iterations

Example output:
```
============================================================
  BENCHMARK RESULTS: OrderBook
============================================================
  Total Operations: 100000
  ---
  Update Operations:
    Average: 45.23 ns
    P50:     42 ns
    P95:     67 ns
    P99:     89 ns
  ---
  Get Best Bid:
    Average: 12.45 ns
  ...
```

## 💡 Optimization Tips

1. **Data Structures**: Carefully chose your data structure. It will be the most critical choice

2. **Hot Path Optimization**:
   - Minimize allocations in `apply_update()`
   - Maximize cache usage

3. **Profiling Tools**:
   ```bash
   # Install flamegraph
   cargo install flamegraph

   # Generate flame graph
   cargo flamegraph

   # Run micro-benchmarks
   cargo bench
   ```

4. **Advanced Techniques**:
   - SIMD for batch operations
   - Lock-free data structures
   - Memory pooling for allocations
   - Branch prediction optimization

## Competition Goal

**Achieve sub-nanosecond operations!**

##  Correctness Tests

Two test suites ensure implementation correctness:

1. **Basic Operations**: Tests bid/ask insertion and queries
2. **Updates & Removes**: Tests quantity updates and level removal

All tests must pass before benchmarking.

## Type Definitions

```rust
// Price in units of 10^-4 (e.g., 10000 = 1.0000)
pub type Price = i64;

// Quantity at a price level
pub type Quantity = u64;

// Order side
pub enum Side { Bid, Ask }

// Update operations
pub enum Update {
    Set { price: Price, quantity: Quantity, side: Side },
    Remove { price: Price, side: Side },
}
```

## Contributing

This is a competitive programming challenge. May the fastest implementation win!

---

**Good luck and happy optimizing!**
