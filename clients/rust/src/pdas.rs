use solana_program::pubkey::Pubkey;

pub fn find_vault_pda(config: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["token-owner".as_bytes(), config.as_ref()], &crate::ID)
}

pub fn find_stake_pda(validator_vote: &Pubkey, config: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "stake::state::validator_stake".as_bytes(),
            validator_vote.as_ref(),
            config.as_ref(),
        ],
        &crate::ID,
    )
}
