pub mod config;
pub mod stake;

pub use config::*;
use solana_program::pubkey::Pubkey;
pub use stake::*;

#[inline(always)]
pub fn find_vault_pda(config: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["token-owner".as_bytes(), config.as_ref()], program_id)
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
