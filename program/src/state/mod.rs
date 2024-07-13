pub mod config;
pub mod stake;

pub use config::*;
use solana_program::{
    program_error::ProgramError,
    pubkey::{Pubkey, PubkeyError},
};
pub use stake::*;

const REWARDS_PER_TOKEN_SCALING_FACTOR: u128 = 1_000_000_000; // 1e9

#[inline(always)]
pub fn find_vault_pda(config: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["token-owner".as_bytes(), config.as_ref()], program_id)
}

#[inline(always)]
pub fn create_vault_pda(
    config: &Pubkey,
    bump: u8,
    program_id: &Pubkey,
) -> Result<Pubkey, PubkeyError> {
    Pubkey::create_program_address(
        &["token-owner".as_bytes(), config.as_ref(), &[bump]],
        program_id,
    )
}

#[inline(always)]
pub fn get_vault_pda_signer_seeds<'a>(config: &'a Pubkey, bump_seed: &'a [u8]) -> [&'a [u8]; 3] {
    ["token-owner".as_bytes(), config.as_ref(), bump_seed]
}

#[inline(always)]
pub fn find_stake_pda(validator: &Pubkey, config: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "stake::state::stake".as_bytes(),
            validator.as_ref(),
            config.as_ref(),
        ],
        program_id,
    )
}

#[inline(always)]
pub fn get_stake_pda_signer_seeds<'a>(
    validator: &'a Pubkey,
    config: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [
        "stake::state::stake".as_bytes(),
        validator.as_ref(),
        config.as_ref(),
        bump_seed,
    ]
}

/// Calculate the eligible rewards for a given token account balance.
///
/// This uses the same logic as the Rewards program.
pub fn calculate_eligible_rewards(
    current_accumulated_rewards_per_token: u128,
    last_accumulated_rewards_per_token: u128,
    token_account_balance: u64,
) -> Result<u64, ProgramError> {
    // Calculation:
    //   (current_accumulated_rewards_per_token
    //     - last_accumulated_rewards_per_token)
    //   * token_account_balance
    let marginal_rate = current_accumulated_rewards_per_token
        .checked_sub(last_accumulated_rewards_per_token)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if marginal_rate == 0 {
        Ok(0)
    } else {
        // Scaled by 1e9 to store 9 decimal places of precision.
        marginal_rate
            .checked_mul(token_account_balance as u128)
            .and_then(|product| product.checked_div(REWARDS_PER_TOKEN_SCALING_FACTOR))
            .and_then(|product| product.try_into().ok())
            .ok_or(ProgramError::ArithmeticOverflow)
    }
}
