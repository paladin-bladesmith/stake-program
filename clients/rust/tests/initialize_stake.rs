#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake::{accounts::Stake, instructions::InitializeStakeBuilder, pdas::find_stake_pda};
use setup::config::create_config;
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::Account,
    clock::Epoch,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    vote::state::VoteState,
};

#[tokio::test]
async fn initialize_config_with_mint_and_token() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let vote = Keypair::new().pubkey();
    let validator = Keypair::new().pubkey();

    let mut data = vec![0u8; VoteState::size_of()];
    // 0-4 is the version offset
    data[4..36].copy_from_slice(validator.as_ref()); // node_pubkey
    data[36..68].copy_from_slice(validator.as_ref()); // authorized_withdrawer

    let vote_account = Account {
        lamports: 5_000_000_000, // 5 SOL
        data,
        owner: solana_sdk::vote::program::ID,
        executable: false,
        rent_epoch: Epoch::default(),
    };

    context.set_account(&vote, &vote_account.into());

    // When we initialize the stake account.

    let (stake_pda, _) = find_stake_pda(&validator, &config);

    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Stake::LEN),
    );

    let initialize_ix = InitializeStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .validator_vote(vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then an account was created with the correct data.

    let account = get_account!(context, stake_pda);
    assert_eq!(account.data.len(), Stake::LEN);

    let account_data = account.data.as_ref();
    let stake_account = Stake::from_bytes(account_data).unwrap();
    assert_eq!(stake_account.validator, validator);
    assert_eq!(stake_account.authority, validator);
}
