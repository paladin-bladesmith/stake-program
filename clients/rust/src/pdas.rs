use solana_program::pubkey::Pubkey;

pub fn find_vault_pda(config: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["token-owner".as_bytes(), config.as_ref()], &crate::ID)
}

pub fn find_stake_pda(validator: &Pubkey, config: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "stake::state::stake".as_bytes(),
            validator.as_ref(),
            config.as_ref(),
        ],
        &crate::ID,
    )
}
