
pub mod query {
    use crate::msg::{ValueResp};

    pub fn value() -> ValueResp {
        ValueResp {
            value: 0,
        }
    }

    pub fn increment(value: u64) -> ValueResp {
        ValueResp { value: value + 1 }
    }
}