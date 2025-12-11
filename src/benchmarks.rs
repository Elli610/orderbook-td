use crate::interfaces::{OrderBook, Side, Update};
use std::hint::black_box;
use std::time::Instant;

// ============================================================================
// BENCHMARKING FRAMEWORK – SUB-NANOSECOND READY
// ============================================================================

const BATCH: u64 = 1000; // amortize Instant::now() noise

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub avg_update_ns: f64,
    pub avg_spread_ns: f64,
    pub avg_best_bid_ns: f64,
    pub avg_best_ask_ns: f64,
    pub avg_random_read_ns: f64,
    pub p50_update_ns: f64,
    pub p95_update_ns: f64,
    pub p99_update_ns: f64,
    pub total_operations: usize,
}

pub struct OrderBookBenchmark;

impl OrderBookBenchmark {
    pub fn run<T: OrderBook>(name: &str, iterations: usize) -> BenchmarkResult {
        let mut ob = T::new();

        Self::warmup(&mut ob);

        let update_timings = Self::benchmark_updates(&mut ob, iterations);

        let spread_timings = Self::benchmark_spread(&ob, iterations / 10);
        let best_bid_timings = Self::benchmark_best_bid(&ob, iterations / 10);
        let best_ask_timings = Self::benchmark_best_ask(&ob, iterations / 10);
        let read_timings = Self::benchmark_random_reads(&ob, iterations / 10);

        let avg_update = Self::average(&update_timings);
        let avg_spread = Self::average(&spread_timings);
        let avg_best_bid = Self::average(&best_bid_timings);
        let avg_best_ask = Self::average(&best_ask_timings);
        let avg_read = Self::average(&read_timings);

        let mut sorted_updates = update_timings.clone();
        sorted_updates.sort_by(|a, b| a.partial_cmp(b).unwrap());

        BenchmarkResult {
            name: name.to_string(),
            avg_update_ns: avg_update,
            avg_spread_ns: avg_spread,
            avg_best_bid_ns: avg_best_bid,
            avg_best_ask_ns: avg_best_ask,
            avg_random_read_ns: avg_read,
            p50_update_ns: sorted_updates[sorted_updates.len() / 2],
            p95_update_ns: sorted_updates[sorted_updates.len() * 95 / 100],
            p99_update_ns: sorted_updates[sorted_updates.len() * 99 / 100],
            total_operations: iterations,
        }
    }

    fn warmup<T: OrderBook>(ob: &mut T) {
        for i in 0..100 {
            ob.apply_update(Update::Set {
                price: 100000 + i * 10,
                quantity: 100,
                side: Side::Bid,
            });
            ob.apply_update(Update::Set {
                price: 100100 + i * 10,
                quantity: 100,
                side: Side::Ask,
            });
        }
    }

    // =========================================================================
    // BENCHMARK UPDATES
    // =========================================================================
    fn benchmark_updates<T: OrderBook>(ob: &mut T, iterations: usize) -> Vec<f64> {
        let mut timings = Vec::with_capacity(iterations);
        let base_price = 100000;

        for i in 0..iterations {
            let update = Update::Set {
                price: base_price + (i as i64 % 1000) * 10,
                quantity: 50 + (i as u64 % 200),
                side: if i % 2 == 0 { Side::Bid } else { Side::Ask },
            };

            let start = Instant::now();
            for _ in 0..BATCH {
                black_box(ob.apply_update(update.clone()));
            }
            let elapsed = start.elapsed().as_nanos() as f64;

            timings.push(elapsed / BATCH as f64);
        }

        timings
    }

    // =========================================================================
    // BENCHMARK SPREAD
    // =========================================================================
    fn benchmark_spread<T: OrderBook>(ob: &T, iterations: usize) -> Vec<f64> {
        let mut timings = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            for _ in 0..BATCH {
                black_box(ob.get_spread());
            }
            let elapsed = start.elapsed().as_nanos() as f64;
            timings.push(elapsed / BATCH as f64);
        }

        timings
    }

    // =========================================================================
    // BENCHMARK BEST BID
    // =========================================================================
    fn benchmark_best_bid<T: OrderBook>(ob: &T, iterations: usize) -> Vec<f64> {
        let mut timings = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            for _ in 0..BATCH {
                black_box(ob.get_best_bid());
            }
            let elapsed = start.elapsed().as_nanos() as f64;
            timings.push(elapsed / BATCH as f64);
        }

        timings
    }

    // =========================================================================
    // BENCHMARK BEST ASK
    // =========================================================================
    fn benchmark_best_ask<T: OrderBook>(ob: &T, iterations: usize) -> Vec<f64> {
        let mut timings = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            for _ in 0..BATCH {
                black_box(ob.get_best_ask());
            }
            let elapsed = start.elapsed().as_nanos() as f64;
            timings.push(elapsed / BATCH as f64);
        }

        timings
    }

    // =========================================================================
    // BENCHMARK RANDOM READS
    // =========================================================================
    fn benchmark_random_reads<T: OrderBook>(ob: &T, iterations: usize) -> Vec<f64> {
        let mut timings = Vec::with_capacity(iterations);
        let base_price = 100_000;

        for i in 0..iterations {
            let price = base_price + (i as i64 % 500) * 10;
            let side = if i % 2 == 0 { Side::Bid } else { Side::Ask };

            let start = Instant::now();
            for _ in 0..BATCH {
                black_box(ob.get_quantity_at(price, side));
            }
            let elapsed = start.elapsed().as_nanos() as f64;
            timings.push(elapsed / BATCH as f64);
        }

        timings
    }

    // =========================================================================
    // STATS
    // =========================================================================
    fn average(v: &[f64]) -> f64 {
        v.iter().sum::<f64>() / v.len() as f64
    }

    pub fn print_results(result: &BenchmarkResult) {
        println!("\n{}", "=".repeat(60));
        println!("  BENCHMARK RESULTS: {}", result.name);
        println!("{}", "=".repeat(60));
        println!("  Total Operations: {}", result.total_operations);
        println!("  ---");
        println!("  Update Operations:");
        println!("    Average: {:.3} ns", result.avg_update_ns);
        println!("    P50:     {:.3} ns", result.p50_update_ns);
        println!("    P95:     {:.3} ns", result.p95_update_ns);
        println!("    P99:     {:.3} ns", result.p99_update_ns);
        println!("  ---");
        println!("  Get Best Bid:   {:.3} ns", result.avg_best_bid_ns);
        println!("  Get Best Ask:   {:.3} ns", result.avg_best_ask_ns);
        println!("  Get Spread:     {:.3} ns", result.avg_spread_ns);
        println!("  Random Reads:   {:.3} ns", result.avg_random_read_ns);
        println!("{}", "=".repeat(60));
    }
}
