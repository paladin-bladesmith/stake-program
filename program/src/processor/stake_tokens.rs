use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::accounts::{Context, StakeTokensAccounts};

/// Stakes tokens with the given config.
///
/// Limited to the current amount of SOL staked to the validator.
///
/// NOTE: Anybody can stake tokens to a validator, but this does not work
/// like native staking, because the validator can take control of staked
/// tokens by deactivating and withdrawing.
///
/// 0. `[w]` Stake config account
/// 1. `[w]` Validator stake account
///     * PDA seeds: ['stake', validator, config_account]
/// 2. `[w]` Token Account
/// 3. `[s]` Owner or delegate of the token account
/// 4. `[]` Validator vote account
/// 3. `[]` Stake Token Mint
/// 4. `[]` Stake Token Vault, to hold all staked tokens.
///   Must be the token account on the stake config account
/// 5. `[]` Token program
/// 6.. Extra accounts required for the transfer hook
///
/// Instruction data: amount of tokens to stake, as a little-endian u64
pub fn process_stake_tokens(
    _program_id: &Pubkey,
    _ctx: Context<StakeTokensAccounts>,
    _amount: u64,
) -> ProgramResult {
    Ok(())
}
