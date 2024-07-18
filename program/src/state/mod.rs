pub mod config;
pub mod stake;

pub use config::*;
pub use stake::*;

use solana_program::pubkey::Pubkey;

/// Defined the maximum valud for basis points (100%).
pub const MAX_BASIS_POINTS: u128 = 10_000;

#[inline(always)]
pub fn find_vault_pda(config: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["token-owner".as_bytes(), config.as_ref()], program_id)
}

#[inline(always)]
pub fn find_stake_pda(
    validator_vote: &Pubkey,
    config: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "stake::state::stake".as_bytes(),
            validator_vote.as_ref(),
            config.as_ref(),
        ],
        program_id,
    )
}

#[inline(always)]
pub fn get_stake_pda_signer_seeds<'a>(
    validator_vote: &'a Pubkey,
    config: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [
        "stake::state::stake".as_bytes(),
        validator_vote.as_ref(),
        config.as_ref(),
        bump_seed,
    ]
}
