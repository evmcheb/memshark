pub mod equality;
#[macro_use]
pub mod range;
pub mod calldata;
pub mod cache;
use ethers::types::{Transaction, Address, U256};

pub trait Filter {
    fn apply(&self, tx: &Transaction) -> bool;
}

pub struct Filters {
    filters: Vec<Box<dyn Filter>>,
}

impl Filters {
    pub fn new() -> Self {
        Filters { filters: Vec::new() }
    }

    pub fn add_filter(&mut self, filter: Box<dyn Filter>) {
        self.filters.push(filter);
    }

    pub fn apply(&self, tx: &Transaction) -> bool {
        self.filters.iter().all(|filter| filter.apply(tx))
    }
}

