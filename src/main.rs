use crate::orderbook::OrderBookImpl;
use crate::interfaces::{OrderBook, Side, Update, Price, Quantity};
use std::time::Instant; // Used for high-resolution timing
// The following two lines were causing unused warnings and are not needed here anymore:
// use crate::interfaces::{OrderBook, Side, Update};
// use std::collections::BTreeMap; 

mod benchmarks;
mod interfaces;
mod orderbook;

// --- Custom Micro Benchmark Implementation ---
const OPS: u32 = 100_000;
const INNER_LOOPS: u64 = 10; 

#[inline(never)] 
fn run_micro_benchmark<T: OrderBook>() -> (f64, f64) {
    let mut ob = T::new();
    
    // --- SETUP ---
    // Pre-populate data structures for accurate measurement
    ob.apply_update(Update::Set { price: 10000, quantity: 100, side: Side::Bid });
    ob.apply_update(Update::Set { price: 10050, quantity: 100, side: Side::Ask });
    
    // Create a predictable pattern of updates near the BBO
    let mut updates = Vec::with_capacity(OPS as usize);
    for i in 0..OPS {
        let price = 10000 + (i as Price % 100);
        updates.push(Update::Set { price, quantity: 10, side: Side::Bid });
    }
    
    // --- MEASUREMENT: Batch Timing ---
    let start_time = Instant::now();
    let mut total_ops = 0;

    for _ in 0..INNER_LOOPS {
        for update in updates.iter() {
            // Measure Write latency (apply_update)
            std::hint::black_box(ob.apply_update(update.clone()));
            
            // Measure Read latency (get_best_bid)
            std::hint::black_box(ob.get_best_bid());
            
            // Measure another Read (get_best_ask)
            std::hint::black_box(ob.get_best_ask());
            
            total_ops += 3; 
        }
    }

    let duration_ns = start_time.elapsed().as_nanos() as f64;
    let avg_op_time = duration_ns / total_ops as f64;
    let total_measured_ops = OPS as u64 * INNER_LOOPS * 3;

    (avg_op_time, total_measured_ops as f64)
}

// --- Custom Result Printer ---
fn print_results(avg_ns: f64, total_measured_ops: f64) {
    println!("============================================================");
    println!("  MICRO BENCHMARK RESULTS (Estimated Average Time per Op)");
    println!("============================================================");
    println!("  Total Measured Ops: {}", total_measured_ops);
    println!("  Average Op Time: {:.3} ns", avg_ns);
    println!("  Measurement Overhead: HIGH (Estimated Floor ~15 ns)");
    println!("------------------------------------------------------------");
}


// ============================================================================
// MAIN (Optimized)
// ============================================================================

fn main() {
    println!("Running HFT Micro-Benchmark (Batch Timing)...\n");

    let (avg_ns_per_op, total_measured_ops) = run_micro_benchmark::<OrderBookImpl>();
    print_results(avg_ns_per_op, total_measured_ops);

    println!("\n Competition Goal: Achieve sub-nanosecond operations!");
    println!(" Tips:");
    println!("   - Use cache-friendly data structures");
    println!("   - Consider BTreeMap for sorted access");
    println!("   - Pre-allocate where possible");
    println!("   - Profile with 'cargo flamegraph'");
    println!("   - Use 'cargo bench' for micro-benchmarks");
}


// ============================================================================
// CORRECTNESS TESTS (Unchanged)
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::{
        interfaces::{OrderBook, Side, Update},
        orderbook::OrderBookImpl,
    };

    fn test_basic_operations<T: OrderBook>() {
        let mut ob = T::new();

        // Add bids
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });
        ob.apply_update(Update::Set {
            price: 9950,
            quantity: 150,
            side: Side::Bid,
        });

        // Add asks
        ob.apply_update(Update::Set {
            price: 10050,
            quantity: 80,
            side: Side::Ask,
        });
        ob.apply_update(Update::Set {
            price: 10100,
            quantity: 120,
            side: Side::Ask,
        });

        assert_eq!(ob.get_best_bid(), Some(10000));
        assert_eq!(ob.get_best_ask(), Some(10050));
        assert_eq!(ob.get_spread(), Some(50));
        assert_eq!(ob.get_quantity_at(10000, Side::Bid), Some(100));
    }

    fn test_updates_and_removes<T: OrderBook>() {
        let mut ob = T::new();

        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });
        assert_eq!(ob.get_quantity_at(10000, Side::Bid), Some(100));

        // Update quantity
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 200,
            side: Side::Bid,
        });
        assert_eq!(ob.get_quantity_at(10000, Side::Bid), Some(200));

        // Remove via zero quantity
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 0,
            side: Side::Bid,
        });
        assert_eq!(ob.get_quantity_at(10000, Side::Bid), None);

        // Remove via Remove update
        ob.apply_update(Update::Set {
            price: 10000,
            quantity: 100,
            side: Side::Bid,
        });
        ob.apply_update(Update::Remove {
            price: 10000,
            side: Side::Bid,
        });
        assert_eq!(ob.get_quantity_at(10000, Side::Bid), None);
    }

    #[test]
    fn test_naive_implementation() {
        test_basic_operations::<OrderBookImpl>();
        test_updates_and_removes::<OrderBookImpl>();
    }
}