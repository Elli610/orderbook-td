use crate::interfaces::{OrderBook, Price, Quantity, Side, Update};
use std::alloc::{alloc_zeroed, handle_alloc_error, Layout};

// ============================================================================
// CONFIGURATION
// ============================================================================
const CAP: usize = 65536;
const MASK: usize = CAP - 1;
const L1_SIZE: usize = CAP / 64;
const L2_SIZE: usize = L1_SIZE / 64;

// Helper pour allouer directement un tableau géant sur le Heap (plus rapide que vec! + unwrap)
fn alloc_heap_zeroed<T, const N: usize>() -> Box<[T; N]> {
    unsafe {
        let layout = Layout::new::<[T; N]>();
        let ptr = alloc_zeroed(layout) as *mut [T; N];
        if ptr.is_null() {
            handle_alloc_error(layout);
        }
        Box::from_raw(ptr)
    }
}

#[derive(Debug)]
#[repr(C, align(64))]
pub struct OrderBookImpl {
    best_bid: Price,
    best_ask: Price,
    total_bid_qty: Quantity,
    total_ask_qty: Quantity,

    root_bid: u64,
    root_ask: u64,

    bid_l2: [u64; L2_SIZE],
    ask_l2: [u64; L2_SIZE],

    bid_l1: Box<[u64; L1_SIZE]>,
    ask_l1: Box<[u64; L1_SIZE]>,

    bid_quantities: Box<[Quantity; CAP]>,
    ask_quantities: Box<[Quantity; CAP]>,
    
    bid_prices: Box<[Price; CAP]>,
    ask_prices: Box<[Price; CAP]>,
}

impl Default for OrderBookImpl {
    fn default() -> Self {
        Self {
            best_bid: i64::MIN,
            best_ask: i64::MAX,
            total_bid_qty: 0,
            total_ask_qty: 0,
            root_bid: 0,
            root_ask: 0,
            bid_l2: [0; L2_SIZE],
            ask_l2: [0; L2_SIZE],
            
            bid_l1: alloc_heap_zeroed(),
            ask_l1: alloc_heap_zeroed(),
            bid_quantities: alloc_heap_zeroed(),
            ask_quantities: alloc_heap_zeroed(),
            bid_prices: alloc_heap_zeroed(),
            ask_prices: alloc_heap_zeroed(),
        }
    }
}

impl OrderBook for OrderBookImpl {
    fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    fn apply_update(&mut self, update: Update) {
        match update {
            Update::Set { price, quantity, side } => {
                let idx = (price as usize) & MASK;
                match side {
                    Side::Bid => self.update_bid(idx, price, quantity),
                    Side::Ask => self.update_ask(idx, price, quantity),
                }
            }
            Update::Remove { price, side } => {
                let idx = (price as usize) & MASK;
                match side {
                    Side::Bid => self.update_bid(idx, price, 0),
                    Side::Ask => self.update_ask(idx, price, 0),
                }
            }
        }
    }

    #[inline(always)]
    fn get_spread(&self) -> Option<Price> {
        if self.best_bid == i64::MIN || self.best_ask == i64::MAX {
            None
        } else {
            Some(self.best_ask - self.best_bid)
        }
    }

    #[inline(always)]
    fn get_best_bid(&self) -> Option<Price> {
        if self.best_bid == i64::MIN { None } else { Some(self.best_bid) }
    }

    #[inline(always)]
    fn get_best_ask(&self) -> Option<Price> {
        if self.best_ask == i64::MAX { None } else { Some(self.best_ask) }
    }

    #[inline(always)]
    fn get_quantity_at(&self, price: Price, side: Side) -> Option<Quantity> {
        let idx = (price as usize) & MASK;
        match side {
            Side::Bid => {
                let q = unsafe { *self.bid_quantities.get_unchecked(idx) };
                if q > 0 { Some(q) } else { None }
            }
            Side::Ask => {
                let q = unsafe { *self.ask_quantities.get_unchecked(idx) };
                if q > 0 { Some(q) } else { None }
            }
        }
    }

    fn get_top_levels(&self, side: Side, n: usize) -> Vec<(Price, Quantity)> {
        let mut out = Vec::with_capacity(n);
        match side {
            Side::Bid => {
                if self.best_bid == i64::MIN { return out; }
                let start_idx = (self.best_bid as usize) & MASK;
                let mut idx = start_idx;
                loop {
                    let q = unsafe { *self.bid_quantities.get_unchecked(idx) };
                    if q > 0 {
                        out.push((unsafe { *self.bid_prices.get_unchecked(idx) }, q));
                        if out.len() >= n { break; }
                    }
                    if idx == 0 { idx = CAP - 1; } else { idx -= 1; }
                    if idx == start_idx { break; }
                }
            }
            Side::Ask => {
                if self.best_ask == i64::MAX { return out; }
                let start_idx = (self.best_ask as usize) & MASK;
                let mut idx = start_idx;
                loop {
                    let q = unsafe { *self.ask_quantities.get_unchecked(idx) };
                    if q > 0 {
                        out.push((unsafe { *self.ask_prices.get_unchecked(idx) }, q));
                        if out.len() >= n { break; }
                    }
                    if idx == CAP - 1 { idx = 0; } else { idx += 1; }
                    if idx == start_idx { break; }
                }
            }
        }
        out
    }

    #[inline(always)]
    fn get_total_quantity(&self, side: Side) -> Quantity {
        match side {
            Side::Bid => self.total_bid_qty,
            Side::Ask => self.total_ask_qty,
        }
    }
}

// ============================================================================
// BITWISE SEARCH ENGINE (LZCNT/TZCNT Optimized)
// ============================================================================
impl OrderBookImpl {
    #[inline(always)]
    fn update_bid(&mut self, idx: usize, price: Price, quantity: Quantity) {
        let old_qty = unsafe { *self.bid_quantities.get_unchecked(idx) };
        unsafe { *self.bid_quantities.get_unchecked_mut(idx) = quantity };
        unsafe { *self.bid_prices.get_unchecked_mut(idx) = price };

        let l1_idx = idx >> 6;
        let l1_bit = 1u64 << (idx & 63);

        if quantity == 0 {
            if old_qty > 0 {
                self.total_bid_qty -= old_qty;
                
                let l1_val = unsafe { *self.bid_l1.get_unchecked(l1_idx) } & !l1_bit;
                unsafe { *self.bid_l1.get_unchecked_mut(l1_idx) = l1_val };

                if l1_val == 0 {
                    let l2_idx = l1_idx >> 6;
                    let l2_bit = 1u64 << (l1_idx & 63);
                    let l2_val = unsafe { *self.bid_l2.get_unchecked(l2_idx) } & !l2_bit;
                    unsafe { *self.bid_l2.get_unchecked_mut(l2_idx) = l2_val };

                    if l2_val == 0 {
                        self.root_bid &= !(1u64 << l2_idx);
                    }
                }
                
                if price == self.best_bid {
                    self.find_new_best_bid();
                }
            }
        } else {
            self.total_bid_qty = self.total_bid_qty + quantity - old_qty;
            if old_qty == 0 {
                unsafe { *self.bid_l1.get_unchecked_mut(l1_idx) |= l1_bit };
                
                let l2_idx = l1_idx >> 6;
                let l2_bit = 1u64 << (l1_idx & 63);
                
                let l2_val = unsafe { *self.bid_l2.get_unchecked(l2_idx) };
                if (l2_val & l2_bit) == 0 {
                    unsafe { *self.bid_l2.get_unchecked_mut(l2_idx) = l2_val | l2_bit };
                    self.root_bid |= 1u64 << l2_idx;
                }
            }
            if price > self.best_bid {
                self.best_bid = price;
            }
        }
    }

    #[inline(always)]
    fn update_ask(&mut self, idx: usize, price: Price, quantity: Quantity) {
        let old_qty = unsafe { *self.ask_quantities.get_unchecked(idx) };
        unsafe { *self.ask_quantities.get_unchecked_mut(idx) = quantity };
        unsafe { *self.ask_prices.get_unchecked_mut(idx) = price };

        let l1_idx = idx >> 6;
        let l1_bit = 1u64 << (idx & 63);

        if quantity == 0 {
            if old_qty > 0 {
                self.total_ask_qty -= old_qty;
                
                let l1_val = unsafe { *self.ask_l1.get_unchecked(l1_idx) } & !l1_bit;
                unsafe { *self.ask_l1.get_unchecked_mut(l1_idx) = l1_val };

                if l1_val == 0 {
                    let l2_idx = l1_idx >> 6;
                    let l2_bit = 1u64 << (l1_idx & 63);
                    let l2_val = unsafe { *self.ask_l2.get_unchecked(l2_idx) } & !l2_bit;
                    unsafe { *self.ask_l2.get_unchecked_mut(l2_idx) = l2_val };

                    if l2_val == 0 {
                        self.root_ask &= !(1u64 << l2_idx);
                    }
                }
                
                if price == self.best_ask {
                    self.find_new_best_ask();
                }
            }
        } else {
            self.total_ask_qty = self.total_ask_qty + quantity - old_qty;
            if old_qty == 0 {
                unsafe { *self.ask_l1.get_unchecked_mut(l1_idx) |= l1_bit };
                
                let l2_idx = l1_idx >> 6;
                let l2_bit = 1u64 << (l1_idx & 63);
                
                let l2_val = unsafe { *self.ask_l2.get_unchecked(l2_idx) };
                if (l2_val & l2_bit) == 0 {
                    unsafe { *self.ask_l2.get_unchecked_mut(l2_idx) = l2_val | l2_bit };
                    self.root_ask |= 1u64 << l2_idx;
                }
            }
            if price < self.best_ask {
                self.best_ask = price;
            }
        }
    }

    #[inline(always)]
    fn find_new_best_bid(&mut self) {
        if self.root_bid == 0 {
            self.best_bid = i64::MIN;
            return;
        }
        
        let l2_idx = 63 - self.root_bid.leading_zeros() as usize;

        let l2_word = unsafe { *self.bid_l2.get_unchecked(l2_idx) };
        let l1_offset = 63 - l2_word.leading_zeros() as usize;
        let l1_idx = (l2_idx << 6) + l1_offset;

        let l1_word = unsafe { *self.bid_l1.get_unchecked(l1_idx) };
        let bit_offset = 63 - l1_word.leading_zeros() as usize;
        
        let final_idx = (l1_idx << 6) + bit_offset;
        self.best_bid = unsafe { *self.bid_prices.get_unchecked(final_idx) };
    }

    #[inline(always)]
    fn find_new_best_ask(&mut self) {
        if self.root_ask == 0 {
            self.best_ask = i64::MAX;
            return;
        }

        let l2_idx = self.root_ask.trailing_zeros() as usize;

        let l2_word = unsafe { *self.ask_l2.get_unchecked(l2_idx) };
        let l1_offset = l2_word.trailing_zeros() as usize;
        let l1_idx = (l2_idx << 6) + l1_offset;

        let l1_word = unsafe { *self.ask_l1.get_unchecked(l1_idx) };
        let bit_offset = l1_word.trailing_zeros() as usize;

        let final_idx = (l1_idx << 6) + bit_offset;
        self.best_ask = unsafe { *self.ask_prices.get_unchecked(final_idx) };
    }
    
    fn find_next_highest_active_idx(&self, start_idx: usize) -> Option<usize> {
        let start_l1_idx = start_idx >> 6;
        let current_bit = start_idx & 63;

        
        let mut word = unsafe { *self.bid_l1.get_unchecked(start_l1_idx) };
        
        let mask = (1u64 << current_bit).wrapping_sub(1);
        word &= mask; 
        
        if word != 0 {
            let bit_offset = 63 - word.leading_zeros() as usize;
            return Some((start_l1_idx << 6) + bit_offset);
        }

        
        for i in (0..start_l1_idx).rev() {
            let l1_word = unsafe { *self.bid_l1.get_unchecked(i) };
            if l1_word != 0 {
                let bit_offset = 63 - l1_word.leading_zeros() as usize;
                return Some((i << 6) + bit_offset);
            }
        }
        
        None
    }
    
    fn find_next_lowest_active_idx(&self, start_idx: usize) -> Option<usize> {
        let start_l1_idx = start_idx >> 6;
        let current_bit = start_idx & 63;
        
        
        let mut word = unsafe { *self.ask_l1.get_unchecked(start_l1_idx) };
        
        let mask = !((1u64 << (current_bit + 1)).wrapping_sub(1));
        word &= mask;
        
        if word != 0 {
            let bit_offset = word.trailing_zeros() as usize;
            return Some((start_l1_idx << 6) + bit_offset);
        }

        
        for i in (start_l1_idx + 1)..L1_SIZE {
            let l1_word = unsafe { *self.ask_l1.get_unchecked(i) };
            if l1_word != 0 {
                let bit_offset = l1_word.trailing_zeros() as usize;
                return Some((i << 6) + bit_offset);
            }
        }
        
        None
    }
}