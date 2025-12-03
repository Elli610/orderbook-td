// ============================================================================
// ORDERBOOK COMPETITION TEMPLATE
// ============================================================================
// Students: Implement the OrderBook trait in your own module
// The fastest implementation wins!
// Target: Sub-nanosecond operations where possible

/// Price is represented as an integer where 1 unit = 10^-4
/// Example: 12345 represents a price of 1.2345
pub type Price = i64;

/// Quantity in the orderbook
pub type Quantity = u64;

/// Side of the order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

/// Order book update operation
#[derive(Debug, Clone)]
pub enum Update {
    /// Add or update a price level (price, quantity, side)
    /// If quantity is 0, this level should be removed
    Set {
        price: Price,
        quantity: Quantity,
        side: Side,
    },

    /// Remove a price level completely
    Remove { price: Price, side: Side },
}

/// The main trait that students must implement
pub trait OrderBook: Send + Sync {
    /// Create a new orderbook instance
    fn new() -> Self
    where
        Self: Sized;

    /// Apply an update to the orderbook
    /// This is the HOT PATH - optimize heavily!
    fn apply_update(&mut self, update: Update);

    /// Get the current spread (best_ask - best_bid)
    /// Returns None if either side is empty
    /// This is also HOT PATH
    fn get_spread(&self) -> Option<Price>;

    /// Get the best bid price
    fn get_best_bid(&self) -> Option<Price>;

    /// Get the best ask price
    fn get_best_ask(&self) -> Option<Price>;

    /// Get quantity at a specific price level
    /// Returns None if the level doesn't exist
    fn get_quantity_at(&self, price: Price, side: Side) -> Option<Quantity>;

    /// Get the top N levels on a given side
    /// Returns Vec of (price, quantity) sorted by best prices first
    fn get_top_levels(&self, side: Side, n: usize) -> Vec<(Price, Quantity)>;

    /// Get total quantity across all levels for a side
    fn get_total_quantity(&self, side: Side) -> Quantity;
}
