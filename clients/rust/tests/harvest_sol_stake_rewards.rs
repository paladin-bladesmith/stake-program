#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    instructions::HarvestSolStakerRewardsBuilder,
};
use setup::{
    calculate_stake_rewards_per_token, config::create_config, setup,
    sol_staker_stake::SolStakerStakeManager, validator_stake::ValidatorStakeManager,
};
use solana_program_test::tokio;
use solana_sdk::{
    account::{Account, AccountSharedData},
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
    config_account.token_amount_delegated = 130;
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
    stake_account.total_staked_token_amount = 65;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 65
    //   - lamports amount = 50
    stake_account.delegation.amount = 65;
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

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
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
        .destination(destination)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[harvest_stake_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the rewards.

    let account = get_account!(context, destination);
    assert_eq!(account.lamports, 100_000_000 + 13); // rent + rewards

    // And the stake account has the updated last seen stake rewards per token.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        200_000_000 // 0.2 * 1e9
    );
}
