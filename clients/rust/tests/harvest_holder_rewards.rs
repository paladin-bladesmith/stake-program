#![cfg(feature = "test-sbf")]

mod setup;

use crate::setup::{config::create_ata, token::mint_to, REWARDS_PER_TOKEN_SCALING_FACTOR};
use borsh::BorshSerialize;
use paladin_rewards_program_client::accounts::{HolderRewards, HolderRewardsPool};
use paladin_stake_program_client::{
    accounts::{Config, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::{
        HarvestHolderRewardsBuilder, HarvestSolStakerRewardsBuilder,
        HarvestValidatorRewardsBuilder, SolStakerStakeTokensBuilder, ValidatorStakeTokensBuilder,
    },
};
use setup::{
    config::ConfigManager, setup, sol_staker_stake::SolStakerStakeManager,
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::{
    account::{Account, AccountSharedData},
    clock::Clock,
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    sysvar::SysvarId,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

struct Fixture {
    config_manager: ConfigManager,
    validator_stake_manager: ValidatorStakeManager,
    sol_staker_stake_manager: SolStakerStakeManager,
    destination_token_account: Pubkey,
}

async fn setup_fixture(context: &mut ProgramTestContext) -> Fixture {
    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(context).await;

    // And a validator stake account.
    let validator_stake_manager = ValidatorStakeManager::new(context, &config_manager.config).await;

    // Setup the stake authorities receiving token account.
    let destination_token_account = get_associated_token_address(
        &validator_stake_manager.authority.pubkey(),
        &config_manager.mint,
    );
    create_ata(
        context,
        &validator_stake_manager.authority.pubkey(),
        &config_manager.mint,
    )
    .await
    .unwrap();

    // Mint 100 tokens to the validator token account.
    mint_to(
        context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &destination_token_account,
        100,
    )
    .await
    .unwrap();

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    // Mint 100 tokens to the validator token account.
    mint_to(
        context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        100,
    )
    .await
    .unwrap();

    Fixture {
        config_manager,
        validator_stake_manager,
        sol_staker_stake_manager,
        destination_token_account,
    }
}

async fn stake_validator(
    context: &mut ProgramTestContext,
    config_manager: &ConfigManager,
    validator_stake_manager: &ValidatorStakeManager,
    destination_token_account: Pubkey,
    amount: u64,
    active_cooldown: Option<u64>,
) {
    // Stake
    let stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .validator_stake(validator_stake_manager.stake)
        .validator_stake_authority(validator_stake_manager.authority.pubkey())
        .source_token_account(destination_token_account)
        .source_token_account_authority(validator_stake_manager.authority.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(amount)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    if let Some(cooldown) = active_cooldown {
        let clock: Clock = bincode::deserialize(&get_account!(context, Clock::id()).data).unwrap();
        stake_account.delegation.unstake_cooldown = clock.unix_timestamp as u64 + cooldown;
    }
    stake_account.total_staked_lamports_amount = 1_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());
}

async fn stake_sol_staker(
    context: &mut ProgramTestContext,
    config_manager: &ConfigManager,
    sol_staker_stake_manager: &SolStakerStakeManager,
    amount: u64,
) {
    let stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(amount)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.rewards_manager.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();
}

async fn set_pool_rewards(
    context: &mut ProgramTestContext,
    config_manager: &ConfigManager,
    rewards_amount: u64,
    accumulated_rewards_per_token: Option<u128>,
) {
    let mut account = get_account!(context, config_manager.rewards_manager.pool);
    let mut holder_rewards_pool_state =
        HolderRewardsPool::from_bytes(account.data.as_ref()).unwrap();

    if let Some(accumulated_rewards_per_token) = accumulated_rewards_per_token {
        holder_rewards_pool_state.accumulated_rewards_per_token =
            accumulated_rewards_per_token * REWARDS_PER_TOKEN_SCALING_FACTOR;
    }

    account.lamports = account.lamports + rewards_amount;
    account.data = holder_rewards_pool_state.try_to_vec().unwrap();
    context.set_account(&config_manager.rewards_manager.pool, &account.into());
}

async fn get_rent(context: &ProgramTestContext, data_len: usize) -> u64 {
    context
        .banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(data_len)
}

#[tokio::test]
async fn validator_stake_harvest_holder_rewards() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        validator_stake_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context).await;

    // Validator stakes 100
    stake_validator(
        &mut context,
        &config_manager,
        &validator_stake_manager,
        destination_token_account,
        100,
        None,
    )
    .await;

    stake_sol_staker(
        &mut context,
        &config_manager,
        &sol_staker_stake_manager,
        100,
    )
    .await;

    // Setup pool state to enable claiming lamports.
    let rewards_lamports = 200_000_000_000;
    set_pool_rewards(&mut context, &config_manager, rewards_lamports, None).await;

    // When we harvest the holder rewards.
    //
    // We are expecting the rewards to be 2 SOL.
    //
    // Calculation:
    //   - total staked: 100
    //   - holder rewards: 4 SOL
    //   - rewards per token: 4_000_000_000_000_000_000 / 100 = 40_000_000_000_000_000 (0.04 SOL)
    //   - rewards for 50 staked: 40_000_000_000_000_000 * 50 = 2_000_000_000_000_000_000 (2 SOL)

    // First harvest holder rewards
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then harvest validator rewards
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .validator_stake(validator_stake_manager.stake)
        .validator_stake_authority(validator_stake_manager.authority.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The authority account has the rewards.
    let account = get_account!(context, validator_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, rewards_lamports / 2);

    // Assert - There should be rewards left on the config account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        (rewards_lamports / 2) + get_rent(&context, Config::LEN).await,
    );

    // Assert - The stake account last seen holder rewards per token is update.
    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        1_000_000_000 * REWARDS_PER_TOKEN_SCALING_FACTOR
    );

    // Assert - The vault authority did not keep any lamports.
    let account = context
        .banks_client
        .get_account(config_manager.vault_pda)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(account.lamports, get_rent(&context, 0).await);
}

#[tokio::test]
async fn validator_stake_harvest_holder_rewards_wrapped() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        validator_stake_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context).await;

    // Validator stakes 50
    stake_validator(
        &mut context,
        &config_manager,
        &validator_stake_manager,
        destination_token_account,
        50,
        None,
    )
    .await;

    // Sol staker stakes 50
    stake_sol_staker(&mut context, &config_manager, &sol_staker_stake_manager, 50).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // Set the stake account's last seen rate to `u128::MAX`.
    stake_account.delegation.last_seen_holder_rewards_per_token = u128::MAX;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // Set the vault's last seen rate to 40_000_000 (0.04 SOL), simulating a
    // scenario where the rate has wrapped around `u128::MAX`.
    // If the holder's last seen rate is `u128::MAX`, the calculation should
    // still work with wrapped math.
    // We have to go just one below 0.04 SOL to cover the wrap around zero.
    let rewards_lamports = 4_000_000_000;
    set_pool_rewards(&mut context, &config_manager, rewards_lamports, None).await;

    let mut account = get_account!(context, config_manager.vault_holder_rewards);
    let mut holder_rewards_account = HolderRewards::from_bytes(account.data.as_ref()).unwrap();
    // Set the vault's last seen rate to 40_000_000 (0.04 SOL), simulating a
    // scenario where the rate has wrapped around `u128::MAX`.
    // If the holder's last seen rate is `u128::MAX`, the calculation should
    // still work with wrapped math.
    // We have to go just one below 0.04 SOL to cover the wrap around zero.
    holder_rewards_account.last_accumulated_rewards_per_token = u128::MAX;
    account.data = holder_rewards_account.try_to_vec().unwrap();
    context.set_account(&config_manager.vault_holder_rewards, &account.into());

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
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .validator_stake(validator_stake_manager.stake)
        .validator_stake_authority(validator_stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the authority account has the rewards.
    let account = get_account!(context, validator_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 2_000_000_000);

    // And there should be rewards left on the config account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        rewards_lamports - 2_000_000_000 + get_rent(&context, Config::LEN).await,
    );

    // And the stake account last seen holder rewards per token is update.
    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        40_000_000 * REWARDS_PER_TOKEN_SCALING_FACTOR,
    );

    // And the vault PDA did not keep any lamports.
    let account = context
        .banks_client
        .get_account(config_manager.vault_pda)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(account.lamports, get_rent(&context, 0).await);
}

#[tokio::test]
async fn validator_stake_harvest_holder_rewards_with_no_rewards_available() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        validator_stake_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context).await;

    // Validator stakes 50
    stake_validator(
        &mut context,
        &config_manager,
        &validator_stake_manager,
        destination_token_account,
        50,
        None,
    )
    .await;

    // Sol staker stakes 50
    stake_sol_staker(&mut context, &config_manager, &sol_staker_stake_manager, 50).await;

    // Setup pool state to enable claiming lamports.
    let rewards_lamports = 4_000_000_000;
    set_pool_rewards(&mut context, &config_manager, rewards_lamports, None).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    validator_stake_account
        .delegation
        .last_seen_holder_rewards_per_token = 40_000_000 * REWARDS_PER_TOKEN_SCALING_FACTOR;
    account.data = validator_stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And we harvest the holder rewards.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .validator_stake(validator_stake_manager.stake)
        .validator_stake_authority(validator_stake_manager.authority.pubkey())
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
        .get_account(validator_stake_manager.authority.pubkey())
        .await
        .unwrap();
    assert!(account.is_none());

    // And the rewards should be on the config account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        rewards_lamports + get_rent(&context, Config::LEN).await
    );
}

#[tokio::test]
async fn validator_stake_harvest_holder_rewards_after_harvesting() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        validator_stake_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context).await;

    // Validator stakes 50
    stake_validator(
        &mut context,
        &config_manager,
        &validator_stake_manager,
        destination_token_account,
        50,
        None,
    )
    .await;

    // Sol staker stakes 50
    stake_sol_staker(&mut context, &config_manager, &sol_staker_stake_manager, 50).await;

    // Setup rewards pool.
    let rewards_lamports = 4_000_000_000;
    set_pool_rewards(&mut context, &config_manager, rewards_lamports, None).await;

    // And we harvest the holder rewards.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .validator_stake(validator_stake_manager.stake)
        .validator_stake_authority(validator_stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The authority account has the rewards.
    let account = get_account!(context, validator_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 2_000_000_000);

    // And we harvest the holder rewards.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .validator_stake(validator_stake_manager.stake)
        .validator_stake_authority(validator_stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_holder, harvest_validator],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the authority should not have received additional rewards.
    let account = get_account!(context, validator_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 2_000_000_000);

    // And there should be rewards left on the configs account.
    let account = get_account!(context, config_manager.config);
    assert_eq!(
        account.lamports,
        rewards_lamports - 2_000_000_000 + get_rent(&context, Config::LEN).await,
    );

    // And the stake account last seen holder rewards per token is update.
    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        40_000_000 * REWARDS_PER_TOKEN_SCALING_FACTOR
    );
}

#[tokio::test]
async fn validator_stake_fail_harvest_holder_rewards_with_wrong_authority() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        validator_stake_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context).await;

    // Validator stakes 50
    stake_validator(
        &mut context,
        &config_manager,
        &validator_stake_manager,
        destination_token_account,
        50,
        None,
    )
    .await;

    // Sol staker stakes 50
    stake_sol_staker(&mut context, &config_manager, &sol_staker_stake_manager, 50).await;

    // Setup rewards pool.
    let rewards_lamports = 4_000_000_000;
    set_pool_rewards(&mut context, &config_manager, rewards_lamports, None).await;

    // When we try to harvest the holder rewards with the wrong authority.
    let fake_authority = Keypair::new();
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .validator_stake(validator_stake_manager.stake)
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
    let mut context = setup(&[]).await;

    // Given a config account (which will be uninitialized) and a stake account.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
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
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        validator_stake_manager,
        ..
    } = setup_fixture(&mut context).await;

    // And an uninitialized stake account.
    context.set_account(
        &validator_stake_manager.stake,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; ValidatorStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to harvest the holder rewards with an uninitialized stake account.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .validator_stake(validator_stake_manager.stake)
        .validator_stake_authority(validator_stake_manager.authority.pubkey())
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
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        validator_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context).await;

    // Validator stakes 50
    stake_validator(
        &mut context,
        &config_manager,
        &validator_stake_manager,
        destination_token_account,
        50,
        None,
    )
    .await;

    // Setup pool state to enable claiming lamports.
    let rewards_lamports = 4_000_000_000;
    set_pool_rewards(&mut context, &config_manager, rewards_lamports, None).await;

    // And we create a new config account.
    let another_config_manager = ConfigManager::new(&mut context).await;

    // When we try to harvest the holder rewards from the wrong config account.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(another_config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(another_config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_validator = HarvestValidatorRewardsBuilder::new()
        .config(another_config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .validator_stake(validator_stake_manager.stake)
        .validator_stake_authority(validator_stake_manager.authority.pubkey())
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
    let Fixture {
        config_manager,
        validator_stake_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context).await;

    // Validator stakes 60
    stake_validator(
        &mut context,
        &config_manager,
        &validator_stake_manager,
        destination_token_account,
        60,
        None,
    )
    .await;

    // Sol staker stakes 40
    stake_sol_staker(&mut context, &config_manager, &sol_staker_stake_manager, 40).await;

    // Setup pool state to enable claiming lamports.
    let rewards_lamports = 4_000_000_000;
    set_pool_rewards(&mut context, &config_manager, rewards_lamports, None).await;

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
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_staker = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
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
        rewards_lamports - 1_600_000_000 + get_rent(&context, Config::LEN).await,
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
        .get_account(config_manager.vault_pda)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(account.lamports, get_rent(&context, 0).await);
}

#[tokio::test]
async fn sol_staker_stake_harvest_holder_rewards_with_no_rewards_available() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        validator_stake_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context).await;

    // Validator stakes 50
    stake_validator(
        &mut context,
        &config_manager,
        &validator_stake_manager,
        destination_token_account,
        50,
        None,
    )
    .await;

    // Sol staker stakes 50
    stake_sol_staker(&mut context, &config_manager, &sol_staker_stake_manager, 50).await;

    // And we harvest the holder rewards.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_staker = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
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
    assert_eq!(account.lamports, get_rent(&context, Config::LEN).await);
}

#[tokio::test]
async fn sol_staker_stake_harvest_holder_rewards_after_harvesting() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        validator_stake_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context).await;

    // Validator stakes 50
    stake_validator(
        &mut context,
        &config_manager,
        &validator_stake_manager,
        destination_token_account,
        50,
        None,
    )
    .await;

    // Sol staker stakes 50
    stake_sol_staker(&mut context, &config_manager, &sol_staker_stake_manager, 50).await;

    // Setup pool state to enable claiming lamports.
    let rewards_lamports = 4_000_000_000;
    set_pool_rewards(&mut context, &config_manager, rewards_lamports, None).await;

    // And we harvest the holder rewards.
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_staker = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
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
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_staker = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
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
        rewards_lamports - 2_000_000_000 + get_rent(&context, Config::LEN).await
    );

    // And the stake account last seen holder rewards per token is update.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        40_000_000 * REWARDS_PER_TOKEN_SCALING_FACTOR
    );
}

#[tokio::test]
async fn sol_staker_stake_fail_harvest_holder_rewards_with_wrong_authority() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        validator_stake_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context).await;

    // Validator stakes 50
    stake_validator(
        &mut context,
        &config_manager,
        &validator_stake_manager,
        destination_token_account,
        50,
        None,
    )
    .await;

    // Sol staker stakes 50
    stake_sol_staker(&mut context, &config_manager, &sol_staker_stake_manager, 50).await;

    // Setup pool state to enable claiming lamports.
    let rewards_lamports = 4_000_000_000;
    set_pool_rewards(&mut context, &config_manager, rewards_lamports, None).await;

    // Setup pool state to enable claiming lamports.
    let rewards_lamports = 4_000_000_000;
    set_pool_rewards(&mut context, &config_manager, rewards_lamports, None).await;

    // When we try to harvest the holder rewards with the wrong authority.
    let fake_authority = Keypair::new();
    let harvest_holder = HarvestHolderRewardsBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .token_program(spl_token::ID)
        .paladin_rewards_program(paladin_rewards_program_client::ID)
        .instruction();
    let harvest_staker = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
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
