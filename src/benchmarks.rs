use crate::interfaces::{OrderBook, Side, Update};
use std::time::Instant;

// ============================================================================
// BENCHMARKING & TESTING FRAMEWORK
// ============================================================================

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub avg_update_ns: f64,
    pub avg_spread_ns: f64,
    pub avg_best_bid_ns: f64,
    pub avg_best_ask_ns: f64,
    pub avg_random_read_ns: f64,
    pub p50_update_ns: u64,
    pub p95_update_ns: u64,
    pub p99_update_ns: u64,
    pub total_operations: usize,
}

pub struct OrderBookBenchmark;

impl OrderBookBenchmark {
    /// Run comprehensive benchmark suite
    pub fn run<T: OrderBook>(name: &str, iterations: usize) -> BenchmarkResult {
        let mut ob = T::new();

        // Warm up
        Self::warmup(&mut ob);

        // Benchmark updates
        let update_timings = Self::benchmark_updates(&mut ob, iterations);

        // Benchmark spread calculations
        let spread_timings = Self::benchmark_spread(&ob, iterations / 10);

        // Benchmark best bid/ask
        let best_bid_timings = Self::benchmark_best_bid(&ob, iterations / 10);
        let best_ask_timings = Self::benchmark_best_ask(&ob, iterations / 10);

        // Benchmark random reads
        let read_timings = Self::benchmark_random_reads(&ob, iterations / 10);

        let avg_update = Self::average(&update_timings);
        let avg_spread = Self::average(&spread_timings);
        let avg_best_bid = Self::average(&best_bid_timings);
        let avg_best_ask = Self::average(&best_ask_timings);
        let avg_read = Self::average(&read_timings);

        let mut sorted_updates = update_timings.clone();
        sorted_updates.sort();

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
        // Add some initial levels
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

    fn benchmark_updates<T: OrderBook>(ob: &mut T, iterations: usize) -> Vec<u64> {
        let mut timings = Vec::with_capacity(iterations);
        let base_price = 100000;

        for i in 0..iterations {
            let update = Update::Set {
                price: base_price + (i as i64 % 1000) * 10,
                quantity: 50 + (i as u64 % 200),
                side: if i % 2 == 0 { Side::Bid } else { Side::Ask },
            };

            let start = Instant::now();
            ob.apply_update(update);
            let elapsed = start.elapsed().as_nanos() as u64;

            timings.push(elapsed);
        }

        timings
    }

    fn benchmark_spread<T: OrderBook>(ob: &T, iterations: usize) -> Vec<u64> {
        let mut timings = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = ob.get_spread();
            let elapsed = start.elapsed().as_nanos() as u64;
            timings.push(elapsed);
        }

        timings
    }

    fn benchmark_best_bid<T: OrderBook>(ob: &T, iterations: usize) -> Vec<u64> {
        let mut timings = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = ob.get_best_bid();
            let elapsed = start.elapsed().as_nanos() as u64;
            timings.push(elapsed);
        }

        timings
    }

    fn benchmark_best_ask<T: OrderBook>(ob: &T, iterations: usize) -> Vec<u64> {
        let mut timings = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = ob.get_best_ask();
            let elapsed = start.elapsed().as_nanos() as u64;
            timings.push(elapsed);
        }

        timings
    }

    fn benchmark_random_reads<T: OrderBook>(ob: &T, iterations: usize) -> Vec<u64> {
        let mut timings = Vec::with_capacity(iterations);
        let base_price = 100000;

        for i in 0..iterations {
            let price = base_price + (i as i64 % 500) * 10;
            let side = if i % 2 == 0 { Side::Bid } else { Side::Ask };

            let start = Instant::now();
            let _ = ob.get_quantity_at(price, side);
            let elapsed = start.elapsed().as_nanos() as u64;
            timings.push(elapsed);
        }

        timings
    }

    fn average(timings: &[u64]) -> f64 {
        timings.iter().sum::<u64>() as f64 / timings.len() as f64
    }

    /// Print formatted results
    pub fn print_results(result: &BenchmarkResult) {
        println!("\n{}", "=".repeat(60));
        println!("  BENCHMARK RESULTS: {}", result.name);
        println!("{}", "=".repeat(60));
        println!("  Total Operations: {}", result.total_operations);
        println!("  ---");
        println!("  Update Operations:");
        println!("    Average: {:.2} ns", result.avg_update_ns);
        println!("    P50:     {} ns", result.p50_update_ns);
        println!("    P95:     {} ns", result.p95_update_ns);
        println!("    P99:     {} ns", result.p99_update_ns);
        println!("  ---");
        println!("  Get Best Bid:");
        println!("    Average: {:.2} ns", result.avg_best_bid_ns);
        println!("  ---");
        println!("  Get Best Ask:");
        println!("    Average: {:.2} ns", result.avg_best_ask_ns);
        println!("  ---");
        println!("  Get Spread:");
        println!("    Average: {:.2} ns", result.avg_spread_ns);
        println!("  ---");
        println!("  Random Reads:");
        println!("    Average: {:.2} ns", result.avg_random_read_ns);
        println!("{}\n", "=".repeat(60));
    }
}
