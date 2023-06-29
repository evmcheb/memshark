#[macro_use]
pub mod equality;
#[macro_use]
pub mod range;
pub mod calldata;
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

equality_filter!(FromFilter, from, Address);
equality_filter!(ToFilter, to, Option<Address>);
equality_filter!(NonceFilter, nonce, U256);
equality_filter!(ValueFilter, value, U256);
equality_filter!(TipFilter, max_priority_fee_per_gas, Option<U256>);
equality_filter!(GasPriceFilter, gas_price, Option<U256>);
equality_filter!(MaxFeeFilter, max_fee_per_gas, Option<U256>);

range_filter!(ValueRangeFilter, value, U256);
range_filter!(NonceRangeFilter, nonce, U256);
range_filter!(TipRangeFilter, max_priority_fee_per_gas, Option<U256>);
range_filter!(GasPriceRangeFilter, gas_price, Option<U256>);
range_filter!(MaxFeeRangeFilter, max_fee_per_gas, Option<U256>);