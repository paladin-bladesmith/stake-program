#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake::{
    accounts::{Config, Stake},
    instructions::InactivateStakeBuilder,
    NullableU64,
};
use setup::{config::create_config, stake::create_stake, vote::create_vote_account};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::Account,
    clock::Clock,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn inactivate_stake() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account (total amount delegated = 100)

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 100;

    let updated_config = Account {
        lamports: account.lamports,
        data: config_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&config, &updated_config.into());

    // And a stake account (amount = 100)

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    let stake_pda = create_stake(&mut context, &validator, &vote, &config).await;

    let account = get_account!(context, stake_pda);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.amount = 100;
    stake_account.deactivating_amount = 50;

    let mut timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;
    timestamp = timestamp.saturating_sub(config_account.cooldown_time_seconds);
    stake_account.deactivation_timestamp = NullableU64::from(timestamp as u64);

    let updated_stake = Account {
        lamports: account.lamports,
        data: stake_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&stake_pda, &updated_stake.into());

    // When we move the deactivated amount to inactive (50 tokens)

    let deactivate_ix = InactivateStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the inactivation should be successful.

    let account = get_account!(context, stake_pda);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.amount, 50);
    assert_eq!(stake_account.deactivating_amount, 0);
    assert_eq!(stake_account.inactive_amount, 50);

    // And the total delegated must decrease.

    let account = get_account!(context, config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(config_account.token_amount_delegated, 50);
}
