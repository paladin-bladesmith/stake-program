#![cfg(feature = "test-sbf")]

mod setup;

use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake},
    errors::PaladinStakeProgramError,
    instructions::SolStakerStakeTokensBuilder,
};
use setup::{
    config::ConfigManager,
    setup,
    sol_staker_stake::SolStakerStakeManager,
    token::{create_token_account, mint_to},
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::tokio;
use solana_sdk::{
    account::AccountSharedData,
    instruction::InstructionError,
    program_pack::Pack,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::state::Account as TokenAccount;

use crate::setup::{rewards::create_holder_rewards, sign_duna_document};

#[tokio::test]
async fn sol_staker_stake_tokens_simple() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        config_manager.rewards_manager.owner.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_000 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        6_500_000_000,
    )
    .await
    .unwrap();

    // When we stake 6_500_000_000 tokens.
    //
    // - raw amount to be staked: 6_500_000_000
    // - current lamports staked: 5_000_000_000
    // - stake limit: 1.3 * 5_000_000_000 = 6_500_000_000

    let stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(6_500_000_000) // <- stake 6_500_000_000 tokens
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.rewards_manager.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the tokens are staked.
    let account = get_account!(context, sol_staker_staker_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.staked_amount, 6_500_000_000);

    // And the vault account has 0 tokens (because they are deposited into holder rewards program)
    let account = get_account!(context, config_manager.vault);
    let vault = TokenAccount::unpack(&account.data).unwrap();
    assert_eq!(vault.amount, 0);

    // And the config account has 6_500_000_000 tokens delegated (staked).
    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_effective, 6_500_000_000);

    // Assert rewards program pool have 6_500_000_000 balance
    let vault = get_account!(context, config_manager.rewards_manager.pool_token_account);
    let vault_token_account = TokenAccount::unpack(&vault.data).unwrap();
    assert_eq!(vault_token_account.amount, 6_500_000_000);

    // assert vault holder rewards account have 6_500_000_000 deposited
    let vault_holder_rewards_account = get_account!(context, config_manager.vault_holder_rewards);
    let vault_holder_rewards =
        HolderRewards::from_bytes(&vault_holder_rewards_account.data).unwrap();
    assert_eq!(vault_holder_rewards.deposited, 6_500_000_000)
}

#[tokio::test]
async fn fail_sol_staker_stake_tokens_with_wrong_vault_account() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        config_manager.rewards_manager.owner.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_000 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        6_500_000_000,
    )
    .await
    .unwrap();

    // And we create a fake vault token account.
    let wrong_vault = Keypair::new();
    let wrong_vault_holder_rewards = HolderRewards::find_pda(&wrong_vault.pubkey()).0;
    sign_duna_document(&mut context, &wrong_vault.pubkey());
    create_holder_rewards(
        &mut context,
        &config_manager.rewards_manager.pool,
        &config_manager.mint,
        wrong_vault.insecure_clone(),
    )
    .await;
    create_token_account(
        &mut context,
        &config_manager.mint_authority.pubkey(),
        &wrong_vault,
        &config_manager.mint,
    )
    .await
    .unwrap();

    // When we try to stake tokens to the fake vault account.
    let stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(wrong_vault.pubkey()) // <- wrong vault account
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(wrong_vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(6_500_000_000)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.rewards_manager.owner],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.
    assert_custom_error!(
        err,
        PaladinStakeProgramError::InvalidVaultHolderRewardsSeeds
    );
}

#[tokio::test]
async fn fail_sol_staker_stake_tokens_with_wrong_config_account() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        config_manager.rewards_manager.owner.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_000 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        6_500_000_000,
    )
    .await
    .unwrap();

    // And we create another config account.
    let another_config = ConfigManager::new(&mut context).await;

    // When we try to stake tokens using the wrong config account.
    let stake_ix = SolStakerStakeTokensBuilder::new()
        .config(another_config.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(6_500_000_000) // <- stake 6_500_000_000 tokens
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.rewards_manager.owner],
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
async fn fail_sol_staker_stake_tokens_with_zero_amount() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        config_manager.rewards_manager.owner.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_000 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        6_500_000_000,
    )
    .await
    .unwrap();

    // When we try to stake 0 tokens.
    let stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(0) // <- 0 tokens
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.rewards_manager.owner],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.
    assert_custom_error!(err, PaladinStakeProgramError::InvalidAmount);
}

#[tokio::test]
async fn fail_sol_staker_stake_tokens_with_uninitialized_stake_account() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        config_manager.rewards_manager.owner.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_000 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        6_500_000_000,
    )
    .await
    .unwrap();

    // And an uninitialized stake account.
    context.set_account(
        &sol_staker_staker_manager.stake,
        &AccountSharedData::from(solana_sdk::account::Account {
            lamports: 100_000_000,
            data: vec![5; SolStakerStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to stake the tokens to the uninitialized stake account.
    let stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(6_500_000_000)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.rewards_manager.owner],
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
async fn sol_staker_stake_tokens_with_insufficient_staked_sol_reduces_effective() {
    let mut context = setup(&[]).await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        config_manager.rewards_manager.owner.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_001 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        6_500_000_001,
    )
    .await
    .unwrap();

    // When we stake 6_500_000_000 tokens.
    //
    // - raw amount to be staked: 6_500_000_001
    // - current lamports staked: 5_000_000_000
    // - stake limit: 1.3 * 5_000_000_000 = 6_500_000_000
    //
    // Stake amount exceeds the stake limit: 6_500_000_001 > 6_500_000_000

    let stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(6_500_000_001) // <- stake 6_500_000_001 tokens
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.rewards_manager.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The staker's effective stake is less than the total stake.
    let staker_stake = get_account!(context, sol_staker_staker_manager.stake);
    let staker_stake = SolStakerStake::from_bytes(&staker_stake.data).unwrap();
    assert_eq!(staker_stake.delegation.staked_amount, 6_500_000_001);
    assert_eq!(staker_stake.delegation.effective_amount, 6_500_000_000);

    // Assert - The global effective stake is less than the total stake.
    let config = get_account!(context, config_manager.config);
    let config = Config::from_bytes(&config.data).unwrap();
    assert_eq!(config.token_amount_effective, 6_500_000_000);
}
