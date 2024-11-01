use solana_program::pubkey::Pubkey;

pub fn find_vault_pda(config: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["token-owner".as_bytes(), config.as_ref()], &crate::ID)
}

pub fn find_sol_staker_stake_pda(stake_state: &Pubkey, config: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "sol_staker_stake".as_bytes(),
            stake_state.as_ref(),
            config.as_ref(),
        ],
        &crate::ID,
    )
}

pub fn find_sol_staker_authority_override_pda(
    original_authority: &Pubkey,
    config: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "sol_staker_authority_override".as_bytes(),
            original_authority.as_ref(),
            config.as_ref(),
        ],
        &crate::ID,
    )
}

pub fn find_validator_stake_pda(validator_vote: &Pubkey, config: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "validator_stake".as_bytes(),
            validator_vote.as_ref(),
            config.as_ref(),
        ],
        &crate::ID,
    )
}
