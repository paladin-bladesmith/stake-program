mod generated;
mod hooked;
pub mod pdas;

pub use {
    generated::{programs::STAKE_ID as ID, *},
    hooked::*,
};
