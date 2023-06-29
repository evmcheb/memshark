use std::collections::HashMap;
use ethers::prelude::gas_oracle::Cache;
use ethers::types::TxHash;

use ethers::{types::Transaction, utils::hex};

use super::Filter;

pub struct CacheFilter {
    seen: HashMap<TxHash, bool>,
}

impl CacheFilter{
    pub fn new() -> Self {
        // convert value to bytes
        Self { seen: HashMap::new() }
    }
}

impl Filter for CacheFilter{
    fn apply(&self, o: &Transaction) -> bool {
        // check if seen return true and add to cache
        if let Some(seen) = self.seen.get(&o.hash) {
            *seen
        } else {
            false
        }
    }
}
