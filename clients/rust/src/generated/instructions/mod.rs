//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

pub(crate) mod r#deactivate_stake;
pub(crate) mod r#distribute_rewards;
pub(crate) mod r#harvest_holder_rewards;
pub(crate) mod r#harvest_stake_rewards;
pub(crate) mod r#inactivate_stake;
pub(crate) mod r#initialize_config;
pub(crate) mod r#initialize_stake;
pub(crate) mod r#set_authority;
pub(crate) mod r#slash;
pub(crate) mod r#stake_tokens;
pub(crate) mod r#update_config;
pub(crate) mod r#withdraw_inactive_stake;

pub use self::{
    r#deactivate_stake::*, r#distribute_rewards::*, r#harvest_holder_rewards::*,
    r#harvest_stake_rewards::*, r#inactivate_stake::*, r#initialize_config::*,
    r#initialize_stake::*, r#set_authority::*, r#slash::*, r#stake_tokens::*, r#update_config::*,
    r#withdraw_inactive_stake::*,
};
