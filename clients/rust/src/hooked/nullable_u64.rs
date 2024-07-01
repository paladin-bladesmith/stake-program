use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NullableU64(u64);

impl NullableU64 {
    pub fn value(&self) -> Option<u64> {
        if self.0 == u64::default() {
            None
        } else {
            Some(self.0)
        }
    }
}

impl From<Option<u64>> for NullableU64 {
    fn from(value: Option<u64>) -> Self {
        Self(value.unwrap_or_default())
    }
}

impl From<u64> for NullableU64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
