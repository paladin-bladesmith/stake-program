//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use crate::hooked::NullableAddress;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    pub discriminator: [u8; 8],
    pub authority: NullableAddress,
    pub slash_authority: NullableAddress,
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub vault: Pubkey,
    pub cooldown_time_seconds: u64,
    pub token_amount_delegated: u64,
    pub sync_rewards_lamports: u64,
    pub accumulated_stake_rewards_per_token: u128,
    pub max_deactivation_basis_points: u16,
    pub vault_authority_bump: u8,
    pub padding: [u8; 5],
}

impl Config {
    pub const LEN: usize = 144;

    #[inline(always)]
    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut data = data;
        Self::deserialize(&mut data)
    }
}

impl<'a> TryFrom<&solana_program::account_info::AccountInfo<'a>> for Config {
    type Error = std::io::Error;

    fn try_from(
        account_info: &solana_program::account_info::AccountInfo<'a>,
    ) -> Result<Self, Self::Error> {
        let mut data: &[u8] = &(*account_info.data).borrow();
        Self::deserialize(&mut data)
    }
}

#[cfg(feature = "anchor")]
impl anchor_lang::AccountDeserialize for Config {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        Ok(Self::deserialize(buf)?)
    }
}

#[cfg(feature = "anchor")]
impl anchor_lang::AccountSerialize for Config {}

#[cfg(feature = "anchor")]
impl anchor_lang::Owner for Config {
    fn owner() -> Pubkey {
        crate::PALADIN_STAKE_PROGRAM_ID
    }
}

#[cfg(feature = "anchor-idl-build")]
impl anchor_lang::IdlBuild for Config {}

#[cfg(feature = "anchor-idl-build")]
impl anchor_lang::Discriminator for Config {
    const DISCRIMINATOR: [u8; 8] = [0; 8];
}
