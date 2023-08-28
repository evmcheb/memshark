pub mod equality;
#[macro_use]
pub mod range;
pub mod calldata;
use ethers::types::Transaction;

pub trait Filter {
    fn apply(&self, tx: &Transaction) -> bool;
}

pub struct Filters {
    filters: Vec<Box<dyn Filter>>,
}

impl Filters {
    pub fn new() -> Self {
        Filters {
            filters: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, filter: Box<dyn Filter>) {
        self.filters.push(filter);
    }

    pub fn apply(&self, tx: &Transaction) -> bool {
        self.filters.iter().all(|filter| filter.apply(tx))
    }
}

pub fn add_optional_filter<T: 'static>(filters: &mut Filters, opt_value: Option<T>, factory: impl Fn(T) -> Box<dyn Filter>) {
    if let Some(value) = opt_value {
        filters.add_filter(factory(value));
    }
}

pub fn add_range_filter<T: 'static>(filters: &mut Filters, opt_gt: Option<T>, opt_lt: Option<T>, factory: impl Fn(Option<T>, Option<T>) -> Box<dyn Filter>) {
    if opt_gt.is_some() || opt_lt.is_some() {
        filters.add_filter(factory(opt_gt, opt_lt));
    }
}
