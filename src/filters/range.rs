use crate::filters::Filter;
use ethers::types::Transaction;
use ethers::types::U256;

macro_rules! range_filter {
    // For fields of type `Option<T>`.
    ($name:ident, $field:ident, Option<$t:ty>) => {
        pub struct $name {
            min: Option<$t>,
            max: Option<$t>,
        }
        impl $name {
            pub fn new(min: Option<$t>, max: Option<$t>) -> Self {
                Self { min, max }
            }
        }

        impl Filter for $name {
            fn apply(&self, o: &Transaction) -> bool {
                if let Some(value) = o.$field {
                    // check if the field is some
                    if let Some(min) = self.min {
                        if value < min {
                            return false;
                        }
                    }

                    if let Some(max) = self.max {
                        if value > max {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
        }
    };
    ($name:ident, $field:ident, $t:ty) => {
        pub struct $name {
            min: Option<$t>,
            max: Option<$t>,
        }
        impl $name {
            pub fn new(min: Option<$t>, max: Option<$t>) -> Self {
                println!("z {:?} {:?}", min, max);
                Self { min, max }
            }
        }

        impl Filter for $name {
            fn apply(&self, tx: &Transaction) -> bool {
                if let Some(min) = self.min {
                    if tx.$field < min {
                        return false;
                    }
                }
                if let Some(max) = self.max {
                    if tx.$field > max {
                        return false;
                    }
                }
                true
            }
        }
    };
}
range_filter!(ValueRangeFilter, value, U256);
range_filter!(NonceRangeFilter, nonce, U256);
range_filter!(TipRangeFilter, max_priority_fee_per_gas, Option<U256>);
range_filter!(GasPriceRangeFilter, gas_price, Option<U256>);
range_filter!(MaxFeeRangeFilter, max_fee_per_gas, Option<U256>);
