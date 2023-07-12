use regex::Regex;

use ethers::{types::Transaction, utils::hex};

use super::Filter;

pub struct SigFilter {
    value: [u8;4],
}
impl SigFilter{
    pub fn new(value: [u8;4]) -> Self {
        // convert value to bytes
        Self { value }
    }
}

impl Filter for SigFilter{
    fn apply(&self, o: &Transaction) -> bool {
        o.input.starts_with(&self.value)
    }
}

pub struct DataFilter {
    value: Vec<u8>,
}
impl DataFilter{
    pub fn new(value: Vec<u8>) -> Self {
        // convert value to bytes
        Self { value }
    }
}

impl Filter for DataFilter {
    fn apply(&self, o: &Transaction) -> bool {
        o.input == self.value
    }
}

pub struct RegexFilter {
    re: Regex
}
impl RegexFilter {
    pub fn new(s: &String) -> Self {
        Self { re: Regex::new(s).unwrap() }
    }
}
impl Filter for RegexFilter {
    fn apply(&self, o: &Transaction) -> bool {
        self.re.is_match(&hex::encode(&o.input))
    }
}
