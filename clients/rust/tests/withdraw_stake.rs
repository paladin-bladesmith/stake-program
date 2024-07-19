#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, Stake},
    instructions::WithdrawInactiveStakeBuilder,
    pdas::find_vault_pda,
};
use setup::{
    add_extra_account_metas_for_transfer,
    config::ConfigManager,
    rewards::{create_holder_rewards, RewardsManager},
    stake::StakeManager,
    token::{create_associated_token_account, mint_to},
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{signature::Signer, transaction::Transaction};
use spl_token_2022::{extension::StateWithExtensions, state::Account};

#[tokio::test]
async fn withdraw_stake() {
    let mut program_test = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    );
    program_test.add_program(
        "paladin_rewards_program",
        paladin_rewards_program_client::ID,
        None,
    );
    let mut context = program_test.start_with_context().await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set total amount delegated = 100 on the config account.

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 100;
    // "manually" update the config account data
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And we set the amount = 50 and inactive_account = 50 on the stake account.

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.amount = 50;
    stake_account.inactive_amount = 50;
    // "manually" update the stake account data
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault and destination accounts.

    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    let owner = context.payer.pubkey();
    let destination =
        create_associated_token_account(&mut context, &owner, &config_manager.mint).await;

    create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &destination,
    )
    .await;

    // When we withdraw the inactive amount from the stake account.

    let (vault_authority, _) = find_vault_pda(&config_manager.config);

    let mut withdraw_ix = WithdrawInactiveStakeBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .destination_token_account(destination)
        .mint(config_manager.mint)
        .stake_authority(stake_manager.authority.pubkey())
        .vault_authority(vault_authority)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- withdraw 50 tokens
        .instruction();

    add_extra_account_metas_for_transfer(
        &mut context,
        &mut withdraw_ix,
        &paladin_rewards_program_client::ID,
        &config_manager.vault,
        &config_manager.mint,
        &destination,
        &vault_authority,
        50, // <- withdraw 50 tokens
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the tokens should be withdrawn to the destination account.

    let account = get_account!(context, destination);
    let destination_account = StateWithExtensions::<Account>::unpack(&account.data).unwrap();

    assert_eq!(destination_account.base.amount, 50);

    // And the total delegated on the config should decrease 100 -> 50.

    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.token_amount_delegated, 50);

    // And the inactive amount on the stake account should be updated.

    let account = get_account!(context, stake_manager.stake);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.inactive_amount, 0);

    // And the vault account should have 50 tokens (decreased from 100).

    let account = get_account!(context, config_account.vault);
    let vault_account = StateWithExtensions::<Account>::unpack(&account.data).unwrap();
    assert_eq!(vault_account.base.amount, 50);
}
