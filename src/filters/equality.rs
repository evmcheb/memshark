use super::Filter;
use ethers::types::{Address, Transaction, U256};

pub struct ToFilter {
    value: Address,
}
impl ToFilter {
    pub fn new(value: Address) -> Self {
        Self { value }
    }
}

impl Filter for ToFilter {
    fn apply(&self, o: &Transaction) -> bool {
        match &o.to {
            Some(addr) => *addr == self.value,
            None => self.value == Address::zero() // Match contract creations to 0x0
        }
    }
}

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
equality_filter!(NonceFilter, nonce, U256);
equality_filter!(ValueFilter, value, U256);
equality_filter!(TipFilter, max_priority_fee_per_gas, Option<U256>);
equality_filter!(GasPriceFilter, gas_price, Option<U256>);
equality_filter!(MaxFeeFilter, max_fee_per_gas, Option<U256>);
