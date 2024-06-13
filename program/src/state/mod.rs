pub mod config;
pub mod stake;

pub use config::*;
pub use stake::*;

use bytemuck::{Pod, Zeroable};

/// The account types for the stake program.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AccountType {
    #[default]
    Uninitialized,
    Config,
    Stake,
}

impl From<u8> for AccountType {
    fn from(value: u8) -> Self {
        match value {
            0 => AccountType::Uninitialized,
            1 => AccountType::Config,
            2 => AccountType::Stake,
            _ => panic!("invalid key value: {value}"),
        }
    }
}

impl From<AccountType> for u8 {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Uninitialized => 0,
            AccountType::Config => 1,
            AccountType::Stake => 2,
        }
    }
}

unsafe impl Pod for AccountType {}

unsafe impl Zeroable for AccountType {}
