#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::{Config, Stake},
    errors::PaladinStakeProgramError,
    instructions::HarvestHolderRewardsBuilder,
    pdas::{find_stake_pda, find_vault_pda},
};
use setup::{
    config::ConfigManager,
    rewards::{create_holder_rewards, create_holder_rewards_pool},
    stake::StakeManager,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn harvest_holder_rewards() {
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

    // Given a config account with 100 staked amount.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And 4 SOL rewards on its vault account.

    let mut account = get_account!(context, config_manager.vault);
    account.lamports = account.lamports.saturating_add(4_000_000_000);
    let rewards_lamports = account.lamports;
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a 50 staked amount.

    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we initialize the holder rewards accounts.

    let holder_rewards_pool = create_holder_rewards_pool(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    let holder_rewards = create_holder_rewards(
        &mut context,
        &holder_rewards_pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    let mut account = get_account!(context, holder_rewards);
    let mut holder_rewards_account = HolderRewards::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set last accumulated rewards per token to 40_000_000 (0.04 SOL)
    holder_rewards_account.last_accumulated_rewards_per_token = 40_000_000 * 1_000_000_000;

    account.data = holder_rewards_account.try_to_vec().unwrap();
    context.set_account(&holder_rewards, &account.into());

    // When we harvest the holder rewards.
    //
    // We are expecting the rewards to be 2 SOL.
    //
    // Calculation:
    //   - total staked: 100
    //   - holder rewards: 4 SOL
    //   - rewards per token: 4_000_000_000 / 100 = 40_000_000 (0.04 SOL)
    //   - rewards for 50 staked: 40_000_000 * 50 = 2_000_000_000 (2 SOL)

    let destination = Pubkey::new_unique();

    let harvest_holder_rewards_ix = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .holder_rewards(holder_rewards)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .stake_authority(stake_manager.authority.pubkey())
        .destination(destination)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the rewards.

    let account = get_account!(context, destination);
    assert_eq!(account.lamports, 2_000_000_000);

    // And there should be rewards left on the vault account.

    let account = get_account!(context, config_manager.vault);
    assert_eq!(
        account.lamports,
        rewards_lamports.saturating_sub(2_000_000_000)
    );

    // And the stake account last seen holder rewards per token is update.

    let account = get_account!(context, stake_manager.stake);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.last_seen_holder_rewards_per_token,
        40_000_000 * 1_000_000_000
    );

    // And the vault authority did not keep any lamports (the account should not exist).

    let account = context
        .banks_client
        .get_account(find_vault_pda(&config_manager.config).0)
        .await
        .unwrap();

    assert!(account.is_none());
}

#[tokio::test]
async fn harvest_holder_rewards_with_no_rewards_available() {
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

    // Given a config account with 100 staked amount.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And 4 SOL rewards on its vault account.

    let mut account = get_account!(context, config_manager.vault);
    account.lamports = account.lamports.saturating_add(4_000_000_000);
    let rewards_lamports = account.lamports;
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a no staked amount.

    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts.

    let holder_rewards_pool = create_holder_rewards_pool(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    let holder_rewards = create_holder_rewards(
        &mut context,
        &holder_rewards_pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    let mut account = get_account!(context, holder_rewards);
    let mut holder_rewards_account = HolderRewards::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set last accumulated rewards per token to 40_000_000 (0.04 SOL)
    holder_rewards_account.last_accumulated_rewards_per_token = 40_000_000 * 1_000_000_000;

    account.data = holder_rewards_account.try_to_vec().unwrap();
    context.set_account(&holder_rewards, &account.into());

    // And we harvest the holder rewards.

    let destination = Pubkey::new_unique();

    let harvest_holder_rewards_ix = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .holder_rewards(holder_rewards)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .stake_authority(stake_manager.authority.pubkey())
        .destination(destination)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination should not receive any rewards (account should not exist).

    let account = context.banks_client.get_account(destination).await.unwrap();
    assert!(account.is_none());

    // And the rewards should be on the vault account.

    let account = get_account!(context, config_manager.vault);
    assert_eq!(account.lamports, rewards_lamports);

    // And the stake account last seen holder rewards per token is not updated.

    let account = get_account!(context, stake_manager.stake);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.last_seen_holder_rewards_per_token, 0);
}

#[tokio::test]
async fn harvest_holder_rewards_after_harvesting() {
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

    // Given a config account with 100 staked amount.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And 4 SOL rewards on its vault account.

    let mut account = get_account!(context, config_manager.vault);
    account.lamports = account.lamports.saturating_add(4_000_000_000);
    let rewards_lamports = account.lamports;
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a 50 staked amount.

    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we initialize the holder rewards accounts.

    let holder_rewards_pool = create_holder_rewards_pool(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    let holder_rewards = create_holder_rewards(
        &mut context,
        &holder_rewards_pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    let mut account = get_account!(context, holder_rewards);
    let mut holder_rewards_account = HolderRewards::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set last accumulated rewards per token to 40_000_000 (0.04 SOL)
    holder_rewards_account.last_accumulated_rewards_per_token = 40_000_000 * 1_000_000_000;

    account.data = holder_rewards_account.try_to_vec().unwrap();
    context.set_account(&holder_rewards, &account.into());

    // And we harvest the holder rewards.

    let destination = Pubkey::new_unique();

    let harvest_holder_rewards_ix = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .holder_rewards(holder_rewards)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .stake_authority(stake_manager.authority.pubkey())
        .destination(destination)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, destination);
    assert_eq!(account.lamports, 2_000_000_000);

    // When we harvest the holder rewards again.

    let second_destination = Pubkey::new_unique();

    let harvest_holder_rewards_ix = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .holder_rewards(holder_rewards)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .stake_authority(stake_manager.authority.pubkey())
        .destination(second_destination)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the second destination account shoud have no rewards.

    let account = context
        .banks_client
        .get_account(second_destination)
        .await
        .unwrap();
    assert!(account.is_none());

    // And there should be rewards left on the vault account.

    let account = get_account!(context, config_manager.vault);
    assert_eq!(
        account.lamports,
        rewards_lamports.saturating_sub(2_000_000_000)
    );

    // And the stake account last seen holder rewards per token is update.

    let account = get_account!(context, stake_manager.stake);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.last_seen_holder_rewards_per_token,
        40_000_000 * 1_000_000_000
    );
}

#[tokio::test]
async fn fail_harvest_holder_rewards_with_wrong_authority() {
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

    // Given a config account with 100 staked amount.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And 4 SOL rewards on its vault account.

    let mut account = get_account!(context, config_manager.vault);
    account.lamports = account.lamports.saturating_add(4_000_000_000);
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a 50 staked amount.

    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we initialize the holder rewards accounts.

    let holder_rewards_pool = create_holder_rewards_pool(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    let holder_rewards = create_holder_rewards(
        &mut context,
        &holder_rewards_pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    let mut account = get_account!(context, holder_rewards);
    let mut holder_rewards_account = HolderRewards::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set last accumulated rewards per token to 40_000_000 (0.04 SOL)
    holder_rewards_account.last_accumulated_rewards_per_token = 40_000_000 * 1_000_000_000;

    account.data = holder_rewards_account.try_to_vec().unwrap();
    context.set_account(&holder_rewards, &account.into());

    // When we try to harvest the holder rewards with the wrong authority.

    let fake_authority = Keypair::new();
    let destination = Pubkey::new_unique();

    let harvest_holder_rewards_ix = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .holder_rewards(holder_rewards)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .stake_authority(fake_authority.pubkey())
        .destination(destination)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &fake_authority],
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
async fn fail_harvest_holder_rewards_with_uninitialized_config() {
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

    // Given a config account (which will be uninitialized) and a stake account.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts.

    let holder_rewards_pool = create_holder_rewards_pool(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    let holder_rewards = create_holder_rewards(
        &mut context,
        &holder_rewards_pool,
        &config_manager.mint,
        &config_manager.vault,
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

    // When we try to harvest the holder rewards with an uninitialized config account.

    let destination = Pubkey::new_unique();

    let harvest_holder_rewards_ix = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .holder_rewards(holder_rewards)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .stake_authority(stake_manager.authority.pubkey())
        .destination(destination)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder_rewards_ix],
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
async fn fail_harvest_holder_rewards_with_uninitialized_stake() {
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

    // Given a config account with 100 staked amount.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And 4 SOL rewards on its vault account.

    let mut account = get_account!(context, config_manager.vault);
    account.lamports = account.lamports.saturating_add(4_000_000_000);
    context.set_account(&config_manager.vault, &account.into());

    // And an uninitialized stake account.

    let validator_vote = Pubkey::new_unique();
    let (stake, _) = find_stake_pda(&validator_vote, &config_manager.config);

    context.set_account(
        &stake,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; Stake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // And we initialize the holder rewards accounts.

    let holder_rewards_pool = create_holder_rewards_pool(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    let holder_rewards = create_holder_rewards(
        &mut context,
        &holder_rewards_pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we try to harvest the holder rewards with an uninitialized stake account.

    let stake_authority = Keypair::new();
    let destination = Pubkey::new_unique();

    let harvest_holder_rewards_ix = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .stake(stake)
        .vault(config_manager.vault)
        .holder_rewards(holder_rewards)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .stake_authority(stake_authority.pubkey())
        .destination(destination)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &stake_authority],
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
async fn fail_harvest_holder_rewards_with_wrong_config() {
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

    // Given a config account with 100 staked amount.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And 4 SOL rewards on its vault account.

    let mut account = get_account!(context, config_manager.vault);
    account.lamports = account.lamports.saturating_add(4_000_000_000);
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a 50 staked amount.

    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we initialize the holder rewards accounts.

    let holder_rewards_pool = create_holder_rewards_pool(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    let holder_rewards = create_holder_rewards(
        &mut context,
        &holder_rewards_pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    let mut account = get_account!(context, holder_rewards);
    let mut holder_rewards_account = HolderRewards::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set last accumulated rewards per token to 40_000_000 (0.04 SOL)
    holder_rewards_account.last_accumulated_rewards_per_token = 40_000_000 * 1_000_000_000;

    account.data = holder_rewards_account.try_to_vec().unwrap();
    context.set_account(&holder_rewards, &account.into());

    // And we create a new config account.

    let another_config_manager = ConfigManager::new(&mut context).await;

    // When we try to harvest the holder rewards from the wrong config account.

    let destination = Pubkey::new_unique();

    let harvest_holder_rewards_ix = HarvestHolderRewardsBuilder::new()
        .config(another_config_manager.config)
        .stake(stake_manager.stake)
        .vault(another_config_manager.vault)
        .holder_rewards(holder_rewards)
        .vault_authority(find_vault_pda(&another_config_manager.config).0)
        .stake_authority(stake_manager.authority.pubkey())
        .destination(destination)
        .mint(another_config_manager.mint)
        .token_program(spl_token_2022::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder_rewards_ix],
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
async fn fail_harvest_holder_rewards_with_invalid_destination() {
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

    // Given a config account with 100 staked amount.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And 4 SOL rewards on its vault account.

    let mut account = get_account!(context, config_manager.vault);
    account.lamports = account.lamports.saturating_add(4_000_000_000);
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a 50 staked amount.

    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we initialize the holder rewards accounts.

    let holder_rewards_pool = create_holder_rewards_pool(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    let holder_rewards = create_holder_rewards(
        &mut context,
        &holder_rewards_pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    let mut account = get_account!(context, holder_rewards);
    let mut holder_rewards_account = HolderRewards::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set last accumulated rewards per token to 40_000_000 (0.04 SOL)
    holder_rewards_account.last_accumulated_rewards_per_token = 40_000_000 * 1_000_000_000;

    account.data = holder_rewards_account.try_to_vec().unwrap();
    context.set_account(&holder_rewards, &account.into());

    // When we try to harvest the holder rewards using the vault authority as destination.

    let vault_authority = find_vault_pda(&config_manager.config).0;
    let destination = vault_authority;

    let harvest_holder_rewards_ix = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .vault(config_manager.vault)
        .holder_rewards(holder_rewards)
        .vault_authority(vault_authority)
        .stake_authority(stake_manager.authority.pubkey())
        .destination(destination) // <- destination is the vault authority
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder_rewards_ix],
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
