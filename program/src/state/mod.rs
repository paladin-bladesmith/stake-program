pub mod config;
pub mod stake;

pub use config::*;
use solana_program::pubkey::Pubkey;
pub use stake::*;

#[inline(always)]
pub fn find_vault_pda(config: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["token-owner".as_bytes(), config.as_ref()], program_id)
}
