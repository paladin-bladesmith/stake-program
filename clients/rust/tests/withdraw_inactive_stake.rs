#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::WithdrawInactiveStakeBuilder,
    pdas::{find_validator_stake_pda, find_vault_pda},
};
use setup::{
    add_extra_account_metas_for_transfer,
    config::ConfigManager,
    rewards::{create_holder_rewards, RewardsManager},
    token::{
        create_associated_token_account, create_token_account, mint_to, TOKEN_ACCOUNT_EXTENSIONS,
    },
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::extension::StateWithExtensions;

#[tokio::test]
async fn withdraw_inactive_stake() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we set total amount delegated = 50 on the config account.

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 50;
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
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
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
    let destination_account =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data).unwrap();

    assert_eq!(destination_account.base.amount, 50);

    // And the total delegated on the config should not change (remains 50).

    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.token_amount_delegated, 50);

    // And the inactive amount on the stake account should be updated.

    let account = get_account!(context, stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.inactive_amount, 0);

    // And the vault account should have 50 tokens (decreased from 100).

    let account = get_account!(context, config_account.vault);
    let vault_account =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data).unwrap();
    assert_eq!(vault_account.base.amount, 50);
}

#[tokio::test]
async fn fail_withdraw_inactive_stake_without_inactive_stake() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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

    // And we set the amount = 100 and no inactive stake.

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.amount = 100;
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

    // When we try to withdraw the non-exitent inactive amount from the stake account.

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
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::NotEnoughInactivatedTokens);
}

#[tokio::test]
async fn fail_withdraw_inactive_stake_with_invalid_stake_authority() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
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

    // When we try to withdraw the inactive amount from the stake account
    // with the wrong stake authority.

    let fake_stake_authority = Keypair::new();
    let (vault_authority, _) = find_vault_pda(&config_manager.config);

    let mut withdraw_ix = WithdrawInactiveStakeBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .destination_token_account(destination)
        .mint(config_manager.mint)
        .stake_authority(fake_stake_authority.pubkey())
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
        &[&context.payer, &fake_stake_authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::InvalidAuthority);
}

#[tokio::test]
async fn fail_withdraw_inactive_stake_with_uninitialized_stake_account() {
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

    // And we set the amount = 50 and inactive_account = 50 on t// And an uninitialized stake account.

    let validator_vote = Pubkey::new_unique();
    let (stake, _) = find_validator_stake_pda(&validator_vote, &config_manager.config);

    context.set_account(
        &stake,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; ValidatorStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

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

    // When we try to withdraw from the uninitialized stake account.

    let (vault_authority, _) = find_vault_pda(&config_manager.config);

    let mut withdraw_ix = WithdrawInactiveStakeBuilder::new()
        .config(config_manager.config)
        .stake(stake)
        .vault(config_manager.vault)
        .destination_token_account(destination)
        .mint(config_manager.mint)
        .stake_authority(config_manager.authority.pubkey())
        .vault_authority(vault_authority)
        .token_program(spl_token_2022::ID)
        .amount(50)
        .instruction();

    add_extra_account_metas_for_transfer(
        &mut context,
        &mut withdraw_ix,
        &paladin_rewards_program_client::ID,
        &config_manager.vault,
        &config_manager.mint,
        &destination,
        &vault_authority,
        50,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::UninitializedAccount);
}

#[tokio::test]
async fn fail_withdraw_inactive_stake_with_uninitialized_config_account() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
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

    // And we uninitialize the config account.

    context.set_account(
        &config_manager.config,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; Config::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to withdraw from the uninitialized stake account.

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
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::UninitializedAccount);
}

#[tokio::test]
async fn fail_withdraw_inactive_stake_with_wrong_config_account() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
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

    // And we create a new config account.

    let another_config_manager = ConfigManager::new(&mut context).await;

    // When we try to withdraw using the wrong config account.

    let (vault_authority, _) = find_vault_pda(&another_config_manager.config);

    let mut withdraw_ix = WithdrawInactiveStakeBuilder::new()
        .config(another_config_manager.config)
        .stake(stake_manager.stake)
        .vault(another_config_manager.vault)
        .destination_token_account(destination)
        .mint(config_manager.mint)
        .stake_authority(stake_manager.authority.pubkey())
        .vault_authority(vault_authority)
        .token_program(spl_token_2022::ID)
        .amount(50)
        .instruction();

    add_extra_account_metas_for_transfer(
        &mut context,
        &mut withdraw_ix,
        &paladin_rewards_program_client::ID,
        &another_config_manager.vault,
        &config_manager.mint,
        &destination,
        &vault_authority,
        50,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_withdraw_inactive_stake_with_wrong_mint() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
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

    // And we create a new config account so we have a different mint account.

    let another_config_manager = ConfigManager::new(&mut context).await;

    let another_rewards_manager = RewardsManager::new(
        &mut context,
        &another_config_manager.mint,
        &another_config_manager.mint_authority,
    )
    .await;

    create_holder_rewards(
        &mut context,
        &another_rewards_manager.pool,
        &another_config_manager.mint,
        &another_config_manager.vault,
    )
    .await;

    // When we try to withdraw using the wrong mint account.

    let (vault_authority, _) = find_vault_pda(&config_manager.config);

    let mut withdraw_ix = WithdrawInactiveStakeBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .destination_token_account(destination)
        .mint(another_config_manager.mint) // <- wrong mint
        .stake_authority(stake_manager.authority.pubkey())
        .vault_authority(vault_authority)
        .token_program(spl_token_2022::ID)
        .amount(50)
        .instruction();

    add_extra_account_metas_for_transfer(
        &mut context,
        &mut withdraw_ix,
        &paladin_rewards_program_client::ID,
        &config_manager.vault,
        &another_config_manager.mint, // <- wrong mint
        &destination,
        &vault_authority,
        50,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::InvalidMint);
}

#[tokio::test]
async fn fail_withdraw_inactive_stake_with_wrong_vault_account() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
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

    // And we create a fake vault token account.

    let wrong_vault = Keypair::new();
    create_token_account(
        &mut context,
        &config_manager.authority.pubkey(),
        &wrong_vault,
        &config_manager.mint,
        TOKEN_ACCOUNT_EXTENSIONS,
    )
    .await
    .unwrap();

    create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &wrong_vault.pubkey(),
    )
    .await;

    // When we try to withdraw using the wrong (fake) vault account.

    let (vault_authority, _) = find_vault_pda(&config_manager.config);

    let mut withdraw_ix = WithdrawInactiveStakeBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(wrong_vault.pubkey()) // <- wrong vault
        .destination_token_account(destination)
        .mint(config_manager.mint)
        .stake_authority(stake_manager.authority.pubkey())
        .vault_authority(vault_authority)
        .token_program(spl_token_2022::ID)
        .amount(50)
        .instruction();

    add_extra_account_metas_for_transfer(
        &mut context,
        &mut withdraw_ix,
        &paladin_rewards_program_client::ID,
        &wrong_vault.pubkey(), // <- wrong vault
        &config_manager.mint,
        &destination,
        &vault_authority,
        50,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::IncorrectVaultAccount);
}

#[tokio::test]
async fn fail_withdraw_inactive_stake_with_vault_as_destination() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.amount = 50;
    stake_account.inactive_amount = 50;
    // "manually" update the stake account data
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault.

    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
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

    // When we try to withdraw using the vault as destination.

    let (vault_authority, _) = find_vault_pda(&config_manager.config);

    let mut withdraw_ix = WithdrawInactiveStakeBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .destination_token_account(config_manager.vault) // <- vault as destination
        .mint(config_manager.mint)
        .stake_authority(stake_manager.authority.pubkey())
        .vault_authority(vault_authority)
        .token_program(spl_token_2022::ID)
        .amount(50)
        .instruction();

    add_extra_account_metas_for_transfer(
        &mut context,
        &mut withdraw_ix,
        &paladin_rewards_program_client::ID,
        &config_manager.vault,
        &config_manager.mint,
        &config_manager.vault, // <- vault as destination
        &vault_authority,
        50,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::InvalidDestinationAccount);
}
