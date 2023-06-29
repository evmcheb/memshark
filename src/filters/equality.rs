use ethers::types::{Address, U256, Transaction, U64, Block, TxHash};
use super::{Filter};

macro_rules! equality_filter {
    // For fields of type `Option<T>`.
    ($name:ident, $field:ident, Option<$t:ty>) => {
        pub struct $name {
            value: $t,
        }
        impl $name {
            pub fn new(value: $t) -> Self {
                Self { value }
            }
        }

        impl Filter for $name {
            fn apply(&self, o: &Transaction) -> bool {
                // If field is some, check if it's equal to the value.
                if let Some(value) = &o.$field {
                    *value == self.value
                } else {
                    false
                }
            }
        }
    };
    // For fields of type `T`.
    ($name:ident, $field:ident, $t:ty) => {
        pub struct $name {
            value: $t,
        }
        impl $name {
            pub fn new(value: $t) -> Self {
                Self { value }
            }
        }

        impl Filter for $name {
            fn apply(&self, o: &Transaction) -> bool {
                o.$field == self.value
            }
        }
    };
}
equality_filter!(FromFilter, from, Address);
equality_filter!(ToFilter, to, Option<Address>);
equality_filter!(NonceFilter, nonce, U256);
equality_filter!(ValueFilter, value, U256);
equality_filter!(TipFilter, max_priority_fee_per_gas, Option<U256>);
equality_filter!(GasPriceFilter, gas_price, Option<U256>);
equality_filter!(MaxFeeFilter, max_fee_per_gas, Option<U256>);