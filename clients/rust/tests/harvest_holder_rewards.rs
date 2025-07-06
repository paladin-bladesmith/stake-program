#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_rewards_program_client::accounts::{HolderRewards, HolderRewardsPool};
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::{
        HarvestHolderRewardsBuilder, HarvestSolStakerRewardsBuilder, HarvestValidatorRewardsBuilder,
    },
    pdas::{find_validator_stake_pda, find_vault_pda},
};
use setup::{
    config::ConfigManager,
    rewards::{create_holder_rewards, create_holder_rewards_pool},
    setup,
    sol_staker_stake::SolStakerStakeManager,
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    vote::state::VoteState,
};
use spl_token_2022::{extension::PodStateWithExtensionsMut, pod::PodAccount};

use crate::setup::sign_duna_document;

#[tokio::test]
async fn validator_stake_harvest_holder_rewards() {
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
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account with 100 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account with a 50 staked amount.
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // Set stake to 50.
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
    stake_account.delegation.effective_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // Sign DUNA document for validator stake account.
    let account = get_account!(context, stake_manager.vote);
    let vote_account = VoteState::deserialize(&account.data).unwrap();
    sign_duna_document(&mut context, &vote_account.authorized_withdrawer);

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

    // Setup pool state to enable claiming lamports.
    let mut account = get_account!(context, holder_rewards_pool);
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();
    holder_rewards_pool_state.accumulated_rewards_per_token =
        40_000_000 * 1_000_000_000_000_000_000;
    let rewards_lamports = 4_000_000_000;
    account.lamports = account.lamports + rewards_lamports;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&holder_rewards_pool, &account.into());

    // When we harvest the holder rewards.
    //
    // We are expecting the rewards to be 2 SOL.
    //
    // Calculation:
    //   - total staked: 100
    //   - holder rewards: 4 SOL
    //   - rewards per token: 4_000_000_000_000_000_000 / 100 = 40_000_000_000_000_000 (0.04 SOL)
    //   - rewards for 50 staked: 40_000_000_000_000_000 * 50 = 2_000_000_000_000_000_000 (2 SOL)

    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The authority account has the rewards.
    let account = get_account!(context, stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 2_000_000_000);

    // Assert - There should be rewards left on the config account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        rewards_lamports - 2_000_000_000 + rent.minimum_balance(Config::LEN),
    );

    // Assert - The stake account last seen holder rewards per token is update.
    let account = get_account!(context, stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        40_000_000 * 1_000_000_000_000_000_000
    );

    // Assert - The vault authority did not keep any lamports (the account should not exist).
    let account = context
        .banks_client
        .get_account(find_vault_pda(&config_manager.config).0)
        .await
        .unwrap();
    assert!(account.is_none());
}

#[tokio::test]
async fn validator_stake_harvest_holder_rewards_wrapped() {
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
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account with 100 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a 50 staked amount.
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.delegation.staked_amount = 50;
    stake_account.delegation.effective_amount = 50;
    // Set the stake account's last seen rate to `u128::MAX`.
    stake_account.delegation.last_seen_holder_rewards_per_token = u128::MAX;
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

    // Setup pool state to enable claiming lamports.
    let mut account = get_account!(context, holder_rewards_pool);
    let rewards_lamports = 4_000_000_000;
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();
    // Set the vault's last seen rate to 40_000_000 (0.04 SOL), simulating a
    // scenario where the rate has wrapped around `u128::MAX`.
    // If the holder's last seen rate is `u128::MAX`, the calculation should
    // still work with wrapped math.
    // We have to go just one below 0.04 SOL to cover the wrap around zero.
    holder_rewards_pool_state.accumulated_rewards_per_token =
        40_000_000 * 1_000_000_000_000_000_000 - 1;
    account.lamports = account.lamports + rewards_lamports;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&holder_rewards_pool, &account.into());

    let mut account = get_account!(context, holder_rewards);
    let mut holder_rewards_account = HolderRewards::from_bytes(account.data.as_ref()).unwrap();
    // Set the vault's last seen rate to 40_000_000 (0.04 SOL), simulating a
    // scenario where the rate has wrapped around `u128::MAX`.
    // If the holder's last seen rate is `u128::MAX`, the calculation should
    // still work with wrapped math.
    // We have to go just one below 0.04 SOL to cover the wrap around zero.
    holder_rewards_account.last_accumulated_rewards_per_token = u128::MAX;
    account.data = holder_rewards_account.try_to_vec().unwrap();
    context.set_account(&holder_rewards, &account.into());

    // When we harvest the holder rewards.
    //
    // We are expecting the rewards to be 2 SOL.
    //
    // Calculation:
    //   - total staked: 100
    //   - holder rewards: 4 SOL
    //   - rewards per token: 4_000_000_000_000_000_000 / 100 = 40_000_000_000_000_000 (0.04 SOL)
    //   - rewards for 50 staked: 40_000_000_000_000_000 * 50 = 2_000_000_000_000_000_000 (2 SOL)

    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the authority account has the rewards.
    let account = get_account!(context, stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 2_000_000_000);

    // And there should be rewards left on the vault account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        rewards_lamports - 2_000_000_000 + rent.minimum_balance(Config::LEN),
    );

    // And the stake account last seen holder rewards per token is update.

    let account = get_account!(context, stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        40_000_000 * 1_000_000_000_000_000_000 - 1,
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
async fn validator_stake_harvest_holder_rewards_with_no_rewards_available() {
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
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account with 100 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a no staked amount.
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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

    // Setup pool state to enable claiming lamports.
    let mut account = get_account!(context, holder_rewards_pool);
    let rewards_lamports = 4_000_000_000;
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();
    holder_rewards_pool_state.accumulated_rewards_per_token =
        40_000_000 * 1_000_000_000_000_000_000;
    account.lamports = account.lamports + rewards_lamports;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&holder_rewards_pool, &account.into());

    // And we harvest the holder rewards.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the stake authority should not receive any rewards (account should not exist).
    let account = context
        .banks_client
        .get_account(stake_manager.authority.pubkey())
        .await
        .unwrap();
    assert!(account.is_none());

    // And the rewards should be on the config account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        rewards_lamports + rent.minimum_balance(Config::LEN)
    );
}

#[tokio::test]
async fn validator_stake_harvest_holder_rewards_after_harvesting() {
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
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account with 100 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a 50 staked amount.
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.delegation.staked_amount = 50;
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

    // Setup rewards pool.
    let mut account = get_account!(context, holder_rewards_pool);
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();
    holder_rewards_pool_state.accumulated_rewards_per_token =
        40_000_000 * 1_000_000_000_000_000_000;
    let rewards_lamports = 4_000_000_000;
    account.lamports = account.lamports + rewards_lamports;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&holder_rewards_pool, &account.into());

    // And we harvest the holder rewards.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The authority account has the rewards.
    let account = get_account!(context, stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 2_000_000_000);

    // When we harvest the holder rewards again.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the authority should not have received additional rewards.
    let account = get_account!(context, stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 2_000_000_000);

    // And there should be rewards left on the vault account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        rewards_lamports - 2_000_000_000 + rent.minimum_balance(Config::LEN),
    );

    // And the stake account last seen holder rewards per token is update.
    let account = get_account!(context, stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        40_000_000 * 1_000_000_000_000_000_000
    );
}

#[tokio::test]
async fn validator_stake_fail_harvest_holder_rewards_with_wrong_authority() {
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
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account with a 50 staked amount.
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
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

    // Setup rewards pool.
    let mut account = get_account!(context, holder_rewards_pool);
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();
    holder_rewards_pool_state.accumulated_rewards_per_token =
        40_000_000 * 1_000_000_000_000_000_000;
    let rewards_lamports = 4_000_000_000;
    account.lamports = account.lamports + rewards_lamports;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&holder_rewards_pool, &account.into());

    // When we try to harvest the holder rewards with the wrong authority.
    let fake_authority = Keypair::new();
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(fake_authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
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
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // And an uninitialized stake account.
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
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .validator_stake(stake)
        .validator_stake_authority(Pubkey::new_unique())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
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
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a 50 staked amount.
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
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

    // Setup pool state to enable claiming lamports.
    let mut account = get_account!(context, holder_rewards_pool);
    let rewards_lamports = 4_000_000_000;
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();
    holder_rewards_pool_state.accumulated_rewards_per_token =
        40_000_000 * 1_000_000_000_000_000_000;
    account.lamports = account.lamports + rewards_lamports;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&holder_rewards_pool, &account.into());

    // And we create a new config account.
    let another_config_manager = ConfigManager::new(&mut context).await;

    // When we try to harvest the holder rewards from the wrong config account.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(another_config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&another_config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(another_config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
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
async fn sol_staker_stake_harvest_holder_rewards() {
    let mut context = setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account with 100 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // Add a validator stake account
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And a sol staker stake account with a 40 total staked amount.
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;
    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 40;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

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

    // Setup pool state to enable claiming lamports.
    let mut account = get_account!(context, holder_rewards_pool);
    let rewards_lamports = 4_000_000_000;
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();
    holder_rewards_pool_state.accumulated_rewards_per_token =
        40_000_000 * 1_000_000_000_000_000_000;
    account.lamports = account.lamports + rewards_lamports;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&holder_rewards_pool, &account.into());

    // When we harvest the holder rewards for the SOL staker stake account.
    //
    // We are expecting the rewards to be 1.6 SOL.
    //
    // Calculation:
    //   - total staked: 100
    //   - holder rewards: 4 SOL
    //   - rewards per token: 4_000_000_000_000_000_000 / 100 = 40_000_000_000_000_000 (0.04 SOL)
    //   - rewards for 40 staked: 40_000_000_000_000_000 * 40 = 1_600_000_000_000_000_000 (1.6 SOL)

    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_staker = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_staker],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the sol staker authority account has the rewards.
    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 1_600_000_000);

    // And there should be rewards left on the config account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        rewards_lamports - 1_600_000_000 + rent.minimum_balance(Config::LEN),
    );

    // And the stake account last seen holder rewards per token is update.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        40_000_000 * 1_000_000_000_000_000_000
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
async fn sol_staker_stake_harvest_holder_rewards_with_no_rewards_available() {
    let mut context = setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account with 100 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht no staked amount.
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And a sol staker stake account wiht no staked amount.
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

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

    // Setup rewards pool.
    let mut account = get_account!(context, holder_rewards_pool);
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();
    holder_rewards_pool_state.accumulated_rewards_per_token =
        40_000_000 * 1_000_000_000_000_000_000;
    let rewards_lamports = 4_000_000_000;
    account.lamports = account.lamports + rewards_lamports;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&holder_rewards_pool, &account.into());

    // And we harvest the holder rewards.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_staker = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_staker],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the sol staker stake authority should not receive any rewards (account should not exist).
    let account = context
        .banks_client
        .get_account(sol_staker_stake_manager.authority.pubkey())
        .await
        .unwrap();
    assert!(account.is_none());

    // And the rewards should be on the config account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        rewards_lamports + rent.minimum_balance(Config::LEN)
    );
}

#[tokio::test]
async fn sol_staker_stake_harvest_holder_rewards_after_harvesting() {
    let mut context = setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account with 100 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // And a stake account wiht a 50 staked amount.
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And a sol staker stake account wiht a 50 total staked amount.
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;
    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

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

    // Setup pool state to enable claiming lamports.
    let mut account = get_account!(context, holder_rewards_pool);
    let rewards_lamports = 4_000_000_000;
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();
    holder_rewards_pool_state.accumulated_rewards_per_token =
        40_000_000 * 1_000_000_000_000_000_000;
    account.lamports = account.lamports + rewards_lamports;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&holder_rewards_pool, &account.into());

    // And we harvest the holder rewards.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_staker = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_staker],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the sol staker authority account has the rewards.
    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 2_000_000_000);

    // When we harvest the holder rewards again.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_staker = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_staker],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // The authority has not received any additional rewards.
    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 2_000_000_000);

    // And there should be rewards left on the vault account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        rewards_lamports - 2_000_000_000 + rent.minimum_balance(Config::LEN)
    );

    // And the stake account last seen holder rewards per token is update.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        40_000_000 * 1_000_000_000_000_000_000
    );
}

#[tokio::test]
async fn sol_staker_stake_fail_harvest_holder_rewards_with_wrong_authority() {
    let mut context = setup(&[]).await;

    // Given a config account with 100 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Set vault token balance to match the total staked (100).
    let mut account = get_account!(context, config_manager.vault);
    let vault = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account.data).unwrap();
    vault.base.amount = 100.into();
    context.set_account(&config_manager.vault, &account.into());

    // And a validator stake account.
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And a sol staker stake account wiht a 50 total staked amount.
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;
    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

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

    // Setup pool state to enable claiming lamports.
    let mut account = get_account!(context, holder_rewards_pool);
    let rewards_lamports = 4_000_000_000;
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();
    holder_rewards_pool_state.accumulated_rewards_per_token =
        40_000_000 * 1_000_000_000_000_000_000;
    account.lamports = account.lamports + rewards_lamports;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&holder_rewards_pool, &account.into());

    // When we try to harvest the holder rewards with the wrong authority.
    let fake_authority = Keypair::new();
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(holder_rewards_pool)
        .vault_holder_rewards(holder_rewards)
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .mint(config_manager.mint)
        .token_program(spl_token_2022::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_staker = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(fake_authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_staker],
        Some(&context.payer.pubkey()),
        &[&context.payer],
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
