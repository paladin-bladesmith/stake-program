#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::HarvestSolStakerRewardsBuilder,
};
use setup::{
    calculate_stake_rewards_per_token,
    config::{create_config, ConfigManager},
    harvest::setup_keeper,
    setup,
    sol_staker_stake::SolStakerStakeManager,
    stake::{create_stake_account, deactivate_stake_account, delegate_stake_account},
    validator_stake::ValidatorStakeManager,
    vote::add_vote_account,
    SWAD,
};
use solana_program_test::tokio;
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    stake::state::{Authorized, Lockup},
    transaction::Transaction,
};

#[tokio::test]
async fn harvest_sol_staker_rewards() {
    let mut context = setup(&[]).await;

    // Given a config account with 26 lamports rewards and 130 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;

    // And a validator stake and sol staker stake accounts with 65 staked tokens.
    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        50 * SWAD,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.active_amount = 65 * SWAD;
    stake_account.delegation.effective_amount = 65 * SWAD;
    stake_account.lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we harvest the stake rewards.
    //
    // We are expecting the rewards to be 13 lamports.
    //
    // Calculation:
    //   - total staked: 130
    //   - stake rewards: 26 lamports
    //   - rewards per token: 26 / 130 = 0.2
    //   - sol staker stake token amount: 65 (limit 1.3 * 50 = 65)
    //   - rewards for 65 staked: 0.2 * 65 = 13 lamports

    // Set the starting authority balance.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Accrue some global rewards.
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);
    account.lamports += 26 * SWAD;
    config_account.lamports_last = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config)
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
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the stake authority account has the rewards.
    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 100_000_000 + 13 * SWAD); // rent + rewards

    // And the stake account has the updated last seen stake rewards per token.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        200_000_000_000_000_000 // 0.2 * 1e18
    );
}

#[tokio::test]
async fn harvest_sol_staker_rewards_wrapped() {
    let mut context = setup(&[]).await;

    // Given a config account with 26 lamports rewards and 130 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;

    // And a validator stake and sol staker stake accounts with 65 staked tokens.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        50 * SWAD,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50 * SWAD;
    // Set the stake account's last seen rate to `u128::MAX`.
    stake_account.delegation.last_seen_stake_rewards_per_token = u128::MAX;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.active_amount = 65 * SWAD;
    stake_account.delegation.effective_amount = 65 * SWAD;
    stake_account.lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we harvest the stake rewards.
    //
    // We are expecting the rewards to be 13 lamports.
    //
    // Calculation:
    //   - total staked: 130
    //   - stake rewards: 26 lamports
    //   - rewards per token: 26 / 130 = 0.2
    //   - sol staker stake token amount: 65 (limit 1.3 * 50 = 65)
    //   - rewards for 65 staked: 0.2 * 65 = 13 lamports

    // Set the starting authority balance.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Accrue some global rewards.
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130 * SWAD;
    // Set the config account's current rewards per token, simulating a
    // scenario where the rate has wrapped around `u128::MAX`.
    // If the holder's last seen rate is `u128::MAX`, the calculation should
    // still work with wrapped math.
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);
    account.lamports += 26 * SWAD;
    config_account.lamports_last = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config)
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
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 100_000_000 + 13 * SWAD); // rent + rewards

    // And the stake account has the updated last seen stake rewards per token.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        200_000_000_000_000_000 // 0.2 * 1e18 - 1
    );
}

#[tokio::test]
async fn harvest_sol_staker_rewards_with_no_rewards_available() {
    let mut context = setup(&[]).await;

    // Given a config account with no rewards and 130 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130 * SWAD;
    config_account.accumulated_stake_rewards_per_token = 0;
    let expected_config_lamports = account.lamports;
    config_account.lamports_last = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake and sol staker stake accounts with 65 staked tokens.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        50 * SWAD,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.active_amount = 65 * SWAD;
    stake_account.delegation.effective_amount = 65 * SWAD;
    stake_account.lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Set the starting authority balance.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config)
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
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 100_000_000); // only rent

    // And the config account lamports are the same.
    let account = get_account!(context, config);
    assert_eq!(account.lamports, expected_config_lamports);
}

#[tokio::test]
async fn harvest_sol_staker_rewards_after_harvesting() {
    let mut context = setup(&[]).await;

    // Given a config account with 26 lamports rewards and 130 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 130 * SWAD;
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);
    account.lamports += 26 * SWAD;
    config_account.lamports_last = account.lamports;
    let expected_config_lamports = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake and sol staker stake accounts with 65 staked tokens.
    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        50 * SWAD,
    )
    .await;
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    //   - last seen stake rewards per token = config.accumulated_stake_rewards_per_token
    stake_account.delegation.active_amount = 65 * SWAD;
    stake_account.delegation.effective_amount = 65 * SWAD;
    stake_account.lamports_amount = 50 * SWAD;
    stake_account.delegation.last_seen_stake_rewards_per_token =
        config_account.accumulated_stake_rewards_per_token;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we harvest the stake rewards after the first harvest.
    //
    // The "first" harvest is sumulated by setting the last seen stake rewards per token to the
    // config accumulated stake rewards per token.

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Make the authority account rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config)
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
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 100_000_000); // only rent

    // And the config account lamports are the same.
    let account = get_account!(context, config);
    assert_eq!(account.lamports, expected_config_lamports);
}

#[tokio::test]
async fn fail_harvest_sol_staker_rewards_with_wrong_authority() {
    let mut context = setup(&[]).await;

    // Given a config account with 26 lamports rewards and 130 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130 * SWAD;
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);
    account.lamports += 26 * SWAD;
    config_account.lamports_last = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake and sol staker stake accounts with 65 staked tokens.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        50 * SWAD,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.active_amount = 65 * SWAD;
    stake_account.delegation.effective_amount = 65 * SWAD;
    stake_account.lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try harvest the stake rewards with the wrong authority.

    let fake_authority = Pubkey::new_unique();
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(fake_authority) // <- wrong authority
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
async fn fail_harvest_sol_staker_rewards_with_wrong_config_account() {
    let mut context = setup(&[]).await;

    // Given a config account with 26 lamports rewards and 130 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130 * SWAD;
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);
    account.lamports += 26 * SWAD;
    config_account.lamports_last = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake and sol staker stake accounts with 65 staked tokens.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        50 * SWAD,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.active_amount = 65 * SWAD;
    stake_account.delegation.effective_amount = 65 * SWAD;
    stake_account.lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And we create another config account.

    let another_config = create_config(&mut context).await;

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Set the starting authority balance.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(another_config)
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
        &[harvest_stake_rewards_ix],
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
    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_harvest_sol_staker_rewards_with_uninitialized_stake_account() {
    let mut context = setup(&[]).await;

    // Given a config account with 26 lamports rewards and 130 staked amount.
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130 * SWAD;
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);
    account.lamports += 26 * SWAD;
    config_account.lamports_last = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake and sol staker stake accounts with 65 staked tokens.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50 * SWAD;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        50 * SWAD,
    )
    .await;

    // Uninitialize the sol staker account.
    context.set_account(
        &sol_staker_stake_manager.stake,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; SolStakerStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // Set the starting authority balance.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config)
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
        &[harvest_stake_rewards_ix],
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
async fn harvest_sol_stake_when_deactivating() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And the SOL staker stake and validator stake accounts are correctly synced.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 5_000_000_000);
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        5_000_000_000
    );

    // And we deactivate the stake.
    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we sync the SOL stake after deactivating the SOL stake.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the SOL amounts are correctly synced (0 SOL staked).
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 0);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);
}

#[tokio::test]
async fn harvest_sol_stake_when_inactive() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And the SOL staker stake and validator stake accounts are correctly synced.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 5_000_000_000);
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        5_000_000_000
    );

    // Deactivate the stake account.
    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we sync the SOL stake after deactivating the SOL stake.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the SOL amounts are correctly synced (0 SOL staked).
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 0);
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);
}

#[tokio::test]
async fn sync_sol_stake_when_effective() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And the SOL staker stake and validator stake accounts are correctly synced.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 5_000_000_000);
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        5_000_000_000
    );

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the SOL amounts are correctly synced (5 SOL staked).
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 5_000_000_000);
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        5_000_000_000
    );
}

#[tokio::test]
async fn sync_sol_stake_when_activating() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the SOL amounts are correctly synced (5 SOL staked).
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 5_000_000_000);
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        5_000_000_000
    );
}

#[tokio::test]
async fn fail_sync_sol_stake_with_wrong_config_account() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we create another config account.
    let another_config = ConfigManager::new(&mut context).await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to sync the SOL stake with the wrong config account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(another_config.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_sync_sol_stake_with_wrong_sol_stake_account() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we create another SOL stake account.

    let another_sol_stake = Keypair::new();
    create_stake_account(
        &mut context,
        &another_sol_stake,
        &Authorized::auto(&Pubkey::new_unique()),
        &Lockup::default(),
        0,
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to sync the SOL stake with the wrong config account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(another_sol_stake.pubkey())
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
    assert_custom_error!(err, PaladinStakeProgramError::IncorrectSolStakeAccount);
}

#[tokio::test]
async fn fail_sync_sol_stake_with_wrong_validator_stake() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we another validator stake account.
    let another_validator_stake = ValidatorStakeManager::new(&mut context, &config_manager.config)
        .await
        .stake;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Deactivate the stake.
    deactivate_stake_account(
        &mut context,
        &sol_staker_stake_manager.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // When we try to sync with the wrong validator stake account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(another_validator_stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_sync_sol_stake_with_uninitialized_config() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
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

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to sync with a unitialized config account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
async fn fail_sync_sol_stake_with_uninitialized_validator_stake() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we uninitialize the validator stake account.
    context.set_account(
        &validator_stake_manager.stake,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; ValidatorStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Deactivate the stake.
    deactivate_stake_account(
        &mut context,
        &sol_staker_stake_manager.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // When we try to sync with a unitialized validator stake account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
async fn fail_sync_sol_stake_with_invalid_sol_stake_view_program() {
    let fake_sol_stake_view_program = Pubkey::new_unique();
    let mut context = setup(&[(
        "paladin_sol_stake_view_program",
        fake_sol_stake_view_program,
    )])
    .await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to sync with a fake sol stake view program.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(fake_sol_stake_view_program) // <- fake sol stake view program
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
    assert_instruction_error!(err, InstructionError::IncorrectProgramId);
}

#[tokio::test]
async fn sync_sol_stake_sol_stake_redelegate_to_non_pal() {
    let mut context = setup(&[]).await;
    let second_vote = add_vote_account(&mut context, &Pubkey::new_unique(), &Pubkey::new_unique());

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked
    // with the first vote account.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And the SOL staker stake and validator stake accounts are correctly synced.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 5_000_000_000);
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        5_000_000_000
    );

    // Deactivate the stake.
    deactivate_stake_account(
        &mut context,
        &sol_staker_stake_manager.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // And we delegate the stake to a second vote account.
    delegate_stake_account(
        &mut context,
        &sol_staker_stake_manager.sol_stake,
        &second_vote,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we sync the SOL stake after the SOL stake has been delegated to a different
    // vote account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(
            paladin_stake_program_client::pdas::find_validator_stake_pda(
                &second_vote,
                &config_manager.config,
            )
            .0,
        )
        .current_validator_stake_authority(Pubkey::new_unique())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the SOL amounts are correctly synced (0 SOL staked).
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 0);
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);
    let config = get_account!(context, config_manager.config);
    let config = Config::from_bytes(&config.data).unwrap();
    assert_eq!(config.token_amount_effective, 0);
}

#[tokio::test]
async fn harvest_sync_rewards_base() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And there is 1 SOL for stake rewards on the config.

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 1_300_000_000;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(1_300_000_000, 1_300_000_000);
    account.lamports += 1_300_000_000; // 1 SOL
    config_account.lamports_last = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And the SOL staker stake has 1_300_000_000 tokens staked.

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 1_300_000_000
    //   - lamports amount = 1_000_000_000
    stake_account.delegation.active_amount = 1_300_000_000;
    stake_account.delegation.effective_amount = 1_300_000_000;
    stake_account.lamports_amount = 1_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 1_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );

    // And we deactivate the stake.
    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we harvest rewards for syncing the SOL stake after deactivating the SOL stake.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the sync rewards.
    let account = get_account!(context, keeper);
    assert_eq!(account.lamports, 100_000_000 + 1_000_000); // rent + rewards

    // The authority account gets the remaining rewards.
    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 100_000_000 + 1_299_000_000); // rent + rewards

    // Then the SOL amounts are correctly synced (0 SOL staked).
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 0);

    // And the last seen stake rewards per token on the SOL staker stake account
    // was updated correctly.
    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        config_account.accumulated_stake_rewards_per_token,
    );
}

#[tokio::test]
async fn harvest_sync_rewards_wrapped() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And there is 1 SOL for stake rewards on the config.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 1_300_000_000;
    // Set the config account's current rewards per token, simulating a
    // scenario where the rate has wrapped around `u128::MAX`.
    // If the holder's last seen rate is `u128::MAX`, the calculation should
    // still work with wrapped math.
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(1_300_000_000, 1_300_000_000);
    account.lamports += 1_300_000_000; // 1 SOL
    config_account.lamports_last = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And the SOL staker stake has 1_300_000_000 tokens staked.

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 1_300_000_000
    //   - lamports amount = 1_000_000_000
    stake_account.delegation.effective_amount = 1_300_000_000;
    stake_account.lamports_amount = 1_000_000_000;
    // Set the stake account's last seen rate to `u128::MAX`.
    stake_account.delegation.last_seen_stake_rewards_per_token = u128::MAX;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 1_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );

    // And we deactivate the stake.

    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we harvest rewards for syncing the SOL stake after deactivating the SOL stake.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Keeper account has the sync rewards.
    let account = get_account!(context, keeper);
    assert_eq!(account.lamports, 100_000_000 + 1_000_000); // rent + rewards

    // Assert - Authority account has the remaining rewards.
    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 100_000_000 + 1_299_000_000); // rent + rewards

    // Assert - Validator lamports are 0.
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);

    // Assert - Staker lamports are 0.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 0);

    // Assert - Validator rewards per taken match config.
    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account
            .delegation
            .last_seen_stake_rewards_per_token,
        config_account.accumulated_stake_rewards_per_token,
    );

    // Assert - Staker rewards per taken match config.
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        config_account.accumulated_stake_rewards_per_token,
    );
}

#[tokio::test]
async fn harvest_sync_rewards_without_rewards() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And there is 1.3 SOL for stake rewards on the config.

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 1_300_000_000;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(1_300_000_000, 1_300_000_000);
    account.lamports += 1_300_000_000; // 1.3 SOL
    config_account.lamports_last = account.lamports;
    let expected_config_lamports = account.lamports;

    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 1_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );

    // And we deactivate the stake.

    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we harvest rewards for syncing the SOL stake after deactivating the SOL stake
    // on a SOL staker stake account with no rewards.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account does not have the sync rewards.
    let account = get_account!(context, keeper);
    assert_eq!(account.lamports, 100_000_000); // only rent

    // And the SOL amounts are correctly synced (0 SOL staked).
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 0);
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        config_account.accumulated_stake_rewards_per_token
    );

    // And the config lamports amount is the same.
    let account = get_account!(context, config_manager.config);
    assert_eq!(account.lamports, expected_config_lamports);
}

#[tokio::test]
async fn harvest_sync_rewards_with_closed_sol_stake_account() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And there is 1 SOL for stake rewards on the config.

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 1_300_000_000;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(1_300_000_000, 1_300_000_000);
    account.lamports += 1_300_000_000; // 1 SOL
    config_account.lamports_last = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And the SOL staker stake has 1_300_000_000 tokens staked.

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 1_300_000_000
    //   - lamports amount = 1_000_000_000
    stake_account.delegation.active_amount = 1_300_000_000;
    stake_account.delegation.effective_amount = 1_300_000_000;
    stake_account.lamports_amount = 1_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 1_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );

    // And we close the SOL stake account.
    context.set_account(
        &stake_account.sol_stake,
        &AccountSharedData::from(Account {
            data: vec![],
            lamports: 0,
            owner: solana_sdk::stake::program::ID,
            ..Default::default()
        }),
    );

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we harvest rewards for syncing the SOL stake after closing the SOL stake account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The keeper account has the sync rewards.
    let account = get_account!(context, keeper);
    assert_eq!(account.lamports, 100_000_000 + 1_000_000); // rent + rewards

    // Assert - The stake authority has the remaining rewards.
    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 100_000_000 + 1_299_000_000); // rent + rewards

    // Assert - Validator lamports are 0.
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);

    // Assert - Staker lamports are 0.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 0);

    // Assert - Validator rewards per taken match config.
    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account
            .delegation
            .last_seen_stake_rewards_per_token,
        config_account.accumulated_stake_rewards_per_token,
    );

    // Assert - Staker rewards per taken match config.
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        config_account.accumulated_stake_rewards_per_token,
    );
}

#[tokio::test]
async fn harvest_sync_rewards_with_capped_sync_rewards() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.
    let config_manager = ConfigManager::with_args(&mut context, 1, 500, 1_300_000_001).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And there is 1_300_000_000 lamports for stake rewards on the config.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 1_300_000_000;
    // add 200_000_000 (0.2) to the accumulated stake rewards per token to simulate
    // a previous reward
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(1_300_000_000, 1_300_000_000) + 200_000_000;
    account.lamports += 1_300_000_000;
    config_account.lamports_last = account.lamports;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And the SOL staker stake has 1_300_000_000 tokens staked.
    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 1_300_000_000
    //   - lamports amount = 1_000_000_000
    stake_account.delegation.active_amount = 1_300_000_000;
    stake_account.delegation.effective_amount = 1_300_000_000;
    // set the last seen stake rewards per token to 200_000_000
    stake_account.delegation.last_seen_stake_rewards_per_token = 200_000_000;
    stake_account.lamports_amount = 1_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 1_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );

    // And we deactivate the stake.

    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
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
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The keeper account has the sync rewards.
    let account = get_account!(context, keeper);
    assert_eq!(account.lamports, 100_000_000 + 1_300_000_000); // rent + rewards

    // Assert - The stake authority has the remaining rewards.
    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 100_000_000); // rent + rewards

    // Assert - Validator lamports are 0.
    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);

    // Assert - Staker lamports are 0.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.lamports_amount, 0);

    // Assert - Validator rewards per taken match config.
    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account
            .delegation
            .last_seen_stake_rewards_per_token,
        config_account.accumulated_stake_rewards_per_token,
    );

    // Assert - Staker rewards per taken match config.
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        config_account.accumulated_stake_rewards_per_token,
    );
}

#[tokio::test]
async fn fail_harvest_sync_rewards_with_wrong_sol_stake_account() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And we create another SOL stake account.

    let another_sol_stake = Keypair::new();
    let authority = Keypair::new();

    create_stake_account(
        &mut context,
        &another_sol_stake,
        &Authorized::auto(&authority.pubkey()),
        &Lockup::default(),
        0, // <- no SOL staked
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to harvest rewards for syncing the SOL stake with the wrong SOL stake account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(another_sol_stake.pubkey()) // <- wrong SOL stake account
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
    assert_custom_error!(err, PaladinStakeProgramError::IncorrectSolStakeAccount);
}

#[tokio::test]
async fn fail_harvest_sync_rewards_with_wrong_previous_validator_stake_account() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And we another validator stake account.
    let another_validator_stake =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Deactivate the stake.
    deactivate_stake_account(
        &mut context,
        &sol_staker_stake_manager.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // Re-delegate the stake.
    delegate_stake_account(
        &mut context,
        &sol_staker_stake_manager.sol_stake,
        &another_validator_stake.vote,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // When we try to harvest rewards for syncing the SOL stake with the wrong validator stake account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(another_validator_stake.stake) // <- new validator stake (we need previous)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(another_validator_stake.stake)
        .current_validator_stake_authority(another_validator_stake.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_harvest_sync_rewards_with_wrong_current_validator_stake_account() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.
    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And we another validator stake account.
    let another_validator_stake =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Deactivate the stake.
    deactivate_stake_account(
        &mut context,
        &sol_staker_stake_manager.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // Re-delegate the stake.
    delegate_stake_account(
        &mut context,
        &sol_staker_stake_manager.sol_stake,
        &another_validator_stake.vote,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // When we try to harvest rewards for syncing the SOL stake with the wrong validator stake account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake) // <- old validator stake (we need the new)
        .current_validator_stake_authority(another_validator_stake.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_harvest_sync_rewards_with_wrong_config_account() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And we create another config account.

    let another_config = ConfigManager::new(&mut context).await.config;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to harvest rewards for syncing the SOL stake with the wrong config account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(another_config) // <- invalid config
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_harvest_sync_rewards_with_invalid_sol_stake_view_program() {
    let fake_sol_stake_view_program = Pubkey::new_unique();
    let mut context = setup(&[(
        "paladin_sol_stake_view_program",
        fake_sol_stake_view_program,
    )])
    .await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to harvest rewards for syncing the SOL stake with an invalid sol stake
    // view program.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(fake_sol_stake_view_program) // <- invalid sol stake view program
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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

    assert_instruction_error!(err, InstructionError::IncorrectProgramId);
}

#[tokio::test]
async fn fail_harvest_sync_rewards_with_wrong_vault_holder_rewards() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // Ensure the authority is rent exempt.
    context.set_account(
        &sol_staker_stake_manager.authority.pubkey(),
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    // Setup the keeper account.
    let keeper = setup_keeper(&mut context);

    // Setup a holder rewards account with 0 accrued rewards.
    let rent = context.banks_client.get_rent().await.unwrap();
    let holder_rewards = HolderRewards::find_pda(&config_manager.vault).0;
    context.set_account(
        &holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Setup another (wrong) holder rewards.
    let wrong_holder_rewards = HolderRewards::find_pda(&Pubkey::new_unique()).0;
    context.set_account(
        &wrong_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to harvest rewards for syncing the SOL stake with the wrong config account.
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config_manager.config)
        .vault_holder_rewards(wrong_holder_rewards)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .sol_staker_native_stake(sol_staker_stake_manager.sol_stake)
        .previous_validator_stake(validator_stake_manager.stake)
        .previous_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .current_validator_stake(validator_stake_manager.stake)
        .current_validator_stake_authority(validator_stake_manager.authority.pubkey())
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .keeper_recipient(Some(keeper))
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}
