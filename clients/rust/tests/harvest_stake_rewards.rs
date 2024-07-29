#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::HarvestStakeRewardsBuilder,
    pdas::find_stake_pda,
};
use setup::{
    config::create_config, validator_stake::create_validator_stake, vote::create_vote_account,
};
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
        // Scaled by 1e9 to store 9 decimal places of precision.
        (rewards as u128)
            .checked_mul(1_000_000_000)
            .and_then(|product| product.checked_div(stake_amount as u128))
            .unwrap()
    }
}

#[tokio::test]
async fn harvest_stake_rewards() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account with 4 SOL rewards and 100 staked amount and
    // a validator's vote account.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(4_000_000_000, 100);

    account.lamports += 4_000_000_000;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account wiht a 50 staked amount.

    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;

    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

    // When we harvest the stake rewards.
    //
    // We are expecting the rewards to be 2 SOL.
    //
    // Calculation:
    //   - total staked: 100
    //   - stake rewards: 4 SOL
    //   - rewards per token: 4_000_000_000 / 100 = 40_000_000 (0.04 SOL)
    //   - rewards for 50 staked: 40_000_000 * 50 = 2_000_000_000 (2 SOL)

    let destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestStakeRewardsBuilder::new()
        .config(config)
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the rewards.

    let account = get_account!(context, destination);
    assert_eq!(account.lamports, 2_000_000_000);

    // And the stake account has the updated last seen stake rewards per token.
    let account = get_account!(context, stake_pda);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.last_seen_stake_rewards_per_token,
        40_000_000 * 1_000_000_000 // 0.04 * 1e9
    );
}

#[tokio::test]
async fn harvest_stake_rewards_with_no_rewards_available() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account with no rewards and 100 staked amount and
    // a validator's vote account.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account wiht a 50 staked amount.

    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;

    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

    // When we harvest the stake rewards with no rewards available.

    let destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestStakeRewardsBuilder::new()
        .config(config)
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the transaction succeeds but the destination account has no rewards.

    let account = context.banks_client.get_account(destination).await.unwrap();
    assert!(account.is_none());
}

#[tokio::test]
async fn harvest_stake_rewards_after_harvesting() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account with 4 SOL rewards and 100 staked amount and
    // a validator's vote account.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(4_000_000_000, 100);

    account.lamports += 4_000_000_000;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account wiht a 50 staked amount.

    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;

    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

    // And we harvest the stake rewards.
    //
    // We are expecting the rewards to be 2 SOL.
    //
    // Calculation:
    //   - total staked: 100
    //   - stake rewards: 4 SOL
    //   - rewards per token: 4_000_000_000 / 100 = 40_000_000 (0.04 SOL)
    //   - rewards for 50 staked: 40_000_000 * 50 = 2_000_000_000 (2 SOL)

    let first_destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestStakeRewardsBuilder::new()
        .config(config)
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .destination(first_destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, first_destination);
    assert_eq!(account.lamports, 2_000_000_000);

    // When we harvest the stake rewards again.
    //
    // We are expecting the rewards to be 0 SOL. There should still be 2 SOL of rewards
    // left on the config account.

    let second_destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestStakeRewardsBuilder::new()
        .config(config)
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .destination(second_destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has no rewards.

    let account = context
        .banks_client
        .get_account(second_destination)
        .await
        .unwrap();
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
async fn fail_harvest_stake_rewards_with_wrong_authority() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account with 4 SOL rewards and 100 staked amount and
    // a validator's vote account.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(4_000_000_000, 100);

    account.lamports += 4_000_000_000;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account wiht a 50 staked amount.

    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;

    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

    // When we try to harvest the stake rewards with the wrong authority.

    let fake_authority = Keypair::new();
    let destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestStakeRewardsBuilder::new()
        .config(config)
        .stake(stake_pda)
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
async fn fail_harvest_stake_rewards_with_uninitialized_config_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account.

    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;

    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

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

    let harvest_stake_rewards_ix = HarvestStakeRewardsBuilder::new()
        .config(config)
        .stake(stake_pda)
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
async fn fail_harvest_stake_rewards_with_uninitialized_stake_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(4_000_000_000, 100);

    account.lamports += 4_000_000_000;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();

    // And an uninitialized stake account.

    let (stake_pda, _) = find_stake_pda(&validator, &config);

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

    let harvest_stake_rewards_ix = HarvestStakeRewardsBuilder::new()
        .config(config)
        .stake(stake_pda)
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
async fn fail_harvest_stake_rewards_with_wrong_config_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account with 4 SOL rewards and 100 staked amount and
    // a validator's vote account.

    let config = create_config(&mut context).await;

    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(4_000_000_000, 100);

    account.lamports += 4_000_000_000;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account wiht a 50 staked amount.

    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;

    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

    // When we try to harvest the stake rewards with the wrong config account.

    let wrong_config = create_config(&mut context).await;
    let destination = Pubkey::new_unique();

    let harvest_stake_rewards_ix = HarvestStakeRewardsBuilder::new()
        .config(wrong_config)
        .stake(stake_pda)
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

    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}
