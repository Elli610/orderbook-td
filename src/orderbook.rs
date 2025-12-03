// ============================================================================
// REFERENCE IMPLEMENTATION (Naive, for correctness comparison)
// ============================================================================

use std::collections::HashMap;

use crate::interfaces::{OrderBook, Price, Quantity, Side, Update};

pub struct OrderBookImpl {}

impl OrderBook for OrderBookImpl {
    fn new() -> Self {
        panic!("Todo")
    }

    fn apply_update(&mut self, update: Update) {
        panic!("Todo")
    }

    fn get_spread(&self) -> Option<Price> {
        panic!("Todo")
    }

    fn get_best_bid(&self) -> Option<Price> {
        panic!("Todo")
    }

    fn get_best_ask(&self) -> Option<Price> {
        panic!("Todo")
    }

    fn get_quantity_at(&self, price: Price, side: Side) -> Option<Quantity> {
        panic!("Todo")
    }

    fn get_top_levels(&self, side: Side, n: usize) -> Vec<(Price, Quantity)> {
        panic!("Todo")
    }

    fn get_total_quantity(&self, side: Side) -> Quantity {
        panic!("Todo")
    }
}
