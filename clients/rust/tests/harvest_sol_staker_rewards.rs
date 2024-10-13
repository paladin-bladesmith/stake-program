#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::HarvestSolStakerRewardsBuilder,
};
use setup::{
    calculate_stake_rewards_per_token, config::create_config, setup,
    sol_staker_stake::SolStakerStakeManager, validator_stake::ValidatorStakeManager,
};
use solana_program_test::tokio;
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};

#[tokio::test]
async fn harvest_sol_staker_rewards() {
    let mut context = setup().await;

    // Given a config account with 26 lamports rewards and 130 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);

    account.lamports += 26;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake and sol staker stake accounts with 65 staked tokens.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.amount = 65;
    stake_account.delegation.effective_amount = 65;
    stake_account.lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

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

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
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
    assert_eq!(account.lamports, 100_000_000 + 13); // rent + rewards

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
    let mut context = setup().await;

    // Given a config account with 26 lamports rewards and 130 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130;
    // Set the config account's current rewards per token, simulating a
    // scenario where the rate has wrapped around `u128::MAX`.
    // If the holder's last seen rate is `u128::MAX`, the calculation should
    // still work with wrapped math.
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);

    account.lamports += 26;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake and sol staker stake accounts with 65 staked tokens.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50;
    // Set the stake account's last seen rate to `u128::MAX`.
    stake_account.delegation.last_seen_stake_rewards_per_token = u128::MAX;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.amount = 65;
    stake_account.delegation.effective_amount = 65;
    stake_account.lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

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

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 100_000_000 + 13); // rent + rewards

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
    let mut context = setup().await;

    // Given a config account with no rewards and 130 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130;
    config_account.accumulated_stake_rewards_per_token = 0;

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
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.amount = 65;
    stake_account.delegation.effective_amount = 65;
    stake_account.lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

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
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
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
    let mut context = setup().await;

    // Given a config account with 26 lamports rewards and 130 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130;
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);

    account.lamports += 26;
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
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    //   - last seen stake rewards per token = config.accumulated_stake_rewards_per_token
    stake_account.delegation.amount = 65;
    stake_account.delegation.effective_amount = 65;
    stake_account.lamports_amount = 50;
    stake_account.delegation.last_seen_stake_rewards_per_token =
        config_account.accumulated_stake_rewards_per_token;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we harvest the stake rewards after the first harvest.
    //
    // The "first" harvest is sumulated by setting the last seen stake rewards per token to the
    // config accumulated stake rewards per token.

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
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
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

#[ignore = "Convert this into a harvest with mismatched SOL leads to sync"]
#[tokio::test]
async fn harvest_sol_staker_rewards_with_excess_rewards() {
    let mut context = setup().await;

    // Given a config account with 13_000_000_000 (13 SOL) lamports rewards and
    // 26_000_000_000 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    let config_rent_lamports = account.lamports;
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 26_000_000_000;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(13_000_000_000, 26_000_000_000);

    account.lamports += 13_000_000_000;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account and sol staker stake account with 13_000_000_000
    // staked tokens and 5_000_000_000 (5 SOL) staked lamports.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values
    stake_account.total_staked_lamports_amount = 5_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 13_000_000_000
    //   - lamports amount = 5_000_000_000 (5 SOL)
    stake_account.delegation.amount = 13_000_000_000;
    stake_account.delegation.effective_amount = 13_000_000_000;
    stake_account.lamports_amount = 5_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we harvest the stake rewards with excess rewards.
    //
    // We are expecting the rewards received by the SOL staker to be 3.25 SOL and the config
    // will retain 9.75 SOL (3.25 excess + 6.5 remaining rewards).
    //
    // Calculation:
    //
    //   [SOL staker stake account]
    //   - amount staked: 13_000_000_000
    //   - SOL stake amount: 5_000_000_000
    //   - stake limit: 1.3 * 5_000_000_000 = 6_500_000_000
    //
    //   [config account]
    //   - total staked: 26_000_000_000
    //   - stake rewards: 13_000_000_000
    //   - rewards per token: 13_000_000_000_000_000_000 / 26_000_000_000 = 0.5 SOL
    //
    //   [harvest]
    //   - rewards for 6_500_000_000 staked, since this is the stake limit:
    //     0.5 * 6_500_000_000 = 3_250_000_000 (3.25 SOL)
    //
    //   - the excess rewards are 3_250_000_000 (3.25 SOL), which goes "back" to
    //     the config account: 6.5 + 3.25 =  9_750_000_000 (9.75 SOL)
    //
    //   - and the excess is shared only with the remaining staked amount:
    //     26_000_000_000 - 13_000_000_000 = 13_000_000_000
    //
    //   - the rewards per token for the remaining staked amount:
    //     3_250_000_000 / 13_000_000_000 = 0.25 SOL
    //
    //   - the final accumulated stake rewards per token: 750_000_000 (0.5 + 0.25 SOL)

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
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the rewards.
    let account = get_account!(context, sol_staker_stake_manager.authority.pubkey());
    assert_eq!(account.lamports, 3_250_000_000);

    // And the stake account has the updated last seen stake rewards per token.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        750_000_000_000_000_000
    );

    // And the config account has the remaining rewards plus the excess rewards.

    let account = get_account!(context, config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        config_account.accumulated_stake_rewards_per_token,
        750_000_000_000_000_000
    );
    assert_eq!(
        account.lamports,
        config_rent_lamports.saturating_add(9_750_000_000)
    );
}

#[tokio::test]
async fn fail_harvest_sol_staker_rewards_with_wrong_authority() {
    let mut context = setup().await;

    // Given a config account with 26 lamports rewards and 130 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130;
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);

    account.lamports += 26;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake and sol staker stake accounts with 65 staked tokens.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.amount = 65;
    stake_account.delegation.effective_amount = 65;
    stake_account.lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we try harvest the stake rewards with the wrong authority.

    let fake_authority = Pubkey::new_unique();
    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .stake_authority(fake_authority) // <- wrong authority
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
    let mut context = setup().await;

    // Given a config account with 26 lamports rewards and 130 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130;
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);

    account.lamports += 26;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake and sol staker stake accounts with 65 staked tokens.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.amount = 65;
    stake_account.delegation.effective_amount = 65;
    stake_account.lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And we create another config account.

    let another_config = create_config(&mut context).await;

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
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
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
    let mut context = setup().await;

    // Given a config account with 26 lamports rewards and 130 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 130;
    config_account.accumulated_stake_rewards_per_token = calculate_stake_rewards_per_token(26, 130);

    account.lamports += 26;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake and sol staker stake accounts with 65 staked tokens.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - total staked token = 65
    //   - total staked lamports = 50
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
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

    let harvest_stake_rewards_ix = HarvestSolStakerRewardsBuilder::new()
        .config(config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
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
