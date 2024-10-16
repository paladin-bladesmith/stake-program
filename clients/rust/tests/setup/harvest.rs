use solana_program_test::ProgramTestContext;
use solana_sdk::{
    account::{Account, AccountSharedData},
    pubkey::Pubkey,
};

pub fn setup_keeper(context: &mut ProgramTestContext) -> Pubkey {
    let keeper = Pubkey::new_unique();
    context.set_account(
        &keeper,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    keeper
}
