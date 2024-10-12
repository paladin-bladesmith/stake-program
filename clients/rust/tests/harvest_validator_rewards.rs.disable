#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::HarvestValidatorRewardsBuilder,
    pdas::find_validator_stake_pda,
};
use setup::{config::create_config, validator_stake::ValidatorStakeManager};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

fn calculate_stake_rewards_per_token(rewards: u64, stake_amount: u64) -> u128 {
    if stake_amount == 0 {
        0
    } else {
        // Calculation: rewards / stake_amount
        //
        // Scaled by 1e18 to store 18 decimal places of precision.
        (rewards as u128)
            .checked_mul(1_000_000_000_000_000_000)
            .and_then(|product| product.checked_div(stake_amount as u128))
            .unwrap()
    }
}

#[tokio::test]
async fn harvest_validator_rewards() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

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

    // And a validator stake account wiht a 65 staked amount.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked amount to 65, SOL stake amount to 50
    // (stake maximum limit is 1.3 * 50 = 65)
    stake_account.delegation.amount = 65;
    stake_account.delegation.effective_amount = 65;
    stake_account.total_staked_lamports_amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we harvest the stake rewards.
    //
    // We are expecting the rewards to be 13 lamports.
    //
    // Calculation:
    //   - total staked: 130
    //   - stake rewards: 26 lamports
    //   - rewards per token: 26 / 130 = 0.2
    //   - validator stake amount: 65 (limit 1.3 * 50 = 65)
    //   - rewards for 65 staked: 0.2 * 65 = 13 lamports

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let harvest_stake_rewards_ix = HarvestValidatorRewardsBuilder::new()
        .config(config)
        .validator_stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the rewards.

    let account = get_account!(context, destination);
    assert_eq!(account.lamports, 100_000_000 + 13); // rent + rewards

    // And the stake account has the updated last seen stake rewards per token.
    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        200_000_000_000_000_000 // 0.2 * 1e18
    );
}

#[tokio::test]
async fn harvest_validator_rewards_wrapped() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

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

    // And a validator stake account wiht a 65 staked amount.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked amount to 65, SOL stake amount to 50
    // (stake maximum limit is 1.3 * 50 = 65)
    stake_account.delegation.amount = 65;
    stake_account.delegation.effective_amount = 65;
    stake_account.total_staked_lamports_amount = 50;
    // Set the stake account's last seen rate to `u128::MAX`.
    stake_account.delegation.last_seen_stake_rewards_per_token = u128::MAX;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we harvest the stake rewards.
    //
    // We are expecting the rewards to be 13 lamports.
    //
    // Calculation:
    //   - total staked: 130
    //   - stake rewards: 26 lamports
    //   - rewards per token: 26 / 130 = 0.2
    //   - validator stake amount: 65 (limit 1.3 * 50 = 65)
    //   - rewards for 65 staked: 0.2 * 65 = 13 lamports

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let harvest_stake_rewards_ix = HarvestValidatorRewardsBuilder::new()
        .config(config)
        .validator_stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the rewards.

    let account = get_account!(context, destination);
    assert_eq!(account.lamports, 100_000_000 + 13); // rent + rewards

    // And the stake account has the updated last seen stake rewards per token.
    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        200_000_000_000_000_000 // 0.2 * 1e18
    );
}

#[tokio::test]
async fn harvest_validator_rewards_with_no_rewards_available() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account with 100 staked amount and no rewards accumulated.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 100;
    config_account.accumulated_stake_rewards_per_token = 0;

    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account wiht a 50 staked amount.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;
    stake_account.delegation.effective_amount = 50;
    stake_account.total_staked_lamports_amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we harvest the stake rewards with no rewards available.

    let destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestValidatorRewardsBuilder::new()
        .config(config)
        .validator_stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction succeeds but the destination account has no rewards.

    let account = context.banks_client.get_account(destination).await.unwrap();
    assert!(account.is_none());
}

#[tokio::test]
async fn harvest_validator_rewards_after_harvesting() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account with 4 SOL rewards and 100 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 100;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(4_000_000_000, 100);

    // only 2 SOL left since we are simulating that 2 SOL were already harvested
    account.lamports += 2_000_000_000;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account wiht a 50 staked amount, which was entitled to
    // 2 SOL rewards.
    //
    // We simulate that the rewards were already harvested by setting the value of
    // last_seen_stake_rewards_per_token to the expected rewards_per_token value (0.04 * 1e18).

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;
    stake_account.delegation.effective_amount = 50;
    stake_account.total_staked_lamports_amount = 50;
    // same as the current value on the config
    stake_account.delegation.last_seen_stake_rewards_per_token =
        config_account.accumulated_stake_rewards_per_token;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we harvest the stake rewards when there are no rewards available.
    //
    // We are expecting the rewards to be 0 SOL.

    let destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestValidatorRewardsBuilder::new()
        .config(config)
        .validator_stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has no rewards.

    let account = context.banks_client.get_account(destination).await.unwrap();
    assert!(account.is_none());

    // And the config account still has 2 SOL of rewards.

    let account = get_account!(context, config);
    let config_rent_exempt = context
        .banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(Config::LEN);
    assert_eq!(
        account.lamports,
        2_000_000_000u64.checked_add(config_rent_exempt).unwrap()
    );
}

#[tokio::test]
async fn fail_harvest_validator_rewards_with_wrong_authority() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account with 4 SOL rewards and 100 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 100;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(4_000_000_000, 100);

    account.lamports += 4_000_000_000;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account wiht a 50 staked amount.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;
    stake_account.delegation.effective_amount = 50;
    stake_account.total_staked_lamports_amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we try to harvest the stake rewards with the wrong authority.

    let fake_authority = Keypair::new();
    let destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestValidatorRewardsBuilder::new()
        .config(config)
        .validator_stake(validator_stake_manager.stake)
        .stake_authority(fake_authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
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
async fn fail_harvest_validator_rewards_with_uninitialized_config_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and validator stake accounts.

    let config = create_config(&mut context).await;
    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    // And we uninitialize the config account.

    context.set_account(
        &config,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; Config::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to harvest stake rewards from an uninitialized config account.

    let destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestValidatorRewardsBuilder::new()
        .config(config)
        .validator_stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
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
async fn fail_harvest_validator_rewards_with_uninitialized_stake_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and an uninitialized stake account.

    let config = create_config(&mut context).await;
    let (stake_pda, _) = find_validator_stake_pda(&Pubkey::new_unique(), &config);

    context.set_account(
        &stake_pda,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; ValidatorStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to harvest stake rewards from an uninitialized stake account.

    let destination = Pubkey::new_unique();
    let authority = Keypair::new();

    let harvest_stake_rewards_ix = HarvestValidatorRewardsBuilder::new()
        .config(config)
        .validator_stake(stake_pda)
        .stake_authority(authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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
async fn failharvest_validator_rewards_with_wrong_config_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account with 4 SOL rewards and 100 staked amount.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_effective = 100;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(4_000_000_000, 100);

    account.lamports += 4_000_000_000;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account wiht a 50 staked amount.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;
    stake_account.delegation.effective_amount = 50;
    stake_account.total_staked_lamports_amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we try to harvest the stake rewards with the wrong config account.

    let wrong_config = create_config(&mut context).await;
    let destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestValidatorRewardsBuilder::new()
        .config(wrong_config)
        .validator_stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
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

#[ignore = "This needs to be replaced by a sync test"]
#[tokio::test]
async fn harvest_validator_rewards_with_excess_rewards() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account with 13 SOL rewards and 26_000_000_000 staked amount.

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

    // And a validator stake account wiht a 13_000_000_000 staked amount.

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // - token amount staked: 13_000_000_000
    // - current lamports staked: 5_000_000_000
    stake_account.delegation.amount = 13_000_000_000;
    stake_account.delegation.effective_amount = 13_000_000_000;
    stake_account.total_staked_lamports_amount = 5_000_000_000;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we harvest the stake rewards with excess rewards.
    //
    // We are expecting the rewards received by the staker to be 1 SOL and the config
    // will retain 3 SOL.
    //
    // Calculation:
    //
    //   [validator stake account]
    //   - token amount staked: 13_000_000_000
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
    let destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestValidatorRewardsBuilder::new()
        .config(config)
        .validator_stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the corrent amount of rewards.

    let account = get_account!(context, destination);
    assert_eq!(account.lamports, 3_250_000_000);

    // And the stake account has the updated last seen stake rewards per token.

    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
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
