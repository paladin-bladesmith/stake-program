#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake},
    errors::PaladinStakeProgramError,
    instructions::SolStakerStakeTokensBuilder,
};
use setup::{
    add_extra_account_metas_for_transfer,
    config::ConfigManager,
    rewards::{create_holder_rewards, RewardsManager},
    setup,
    sol_staker_stake::SolStakerStakeManager,
    token::{create_token_account, mint_to, TOKEN_ACCOUNT_EXTENSIONS},
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::tokio;
use solana_sdk::{
    account::AccountSharedData,
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::{extension::StateWithExtensions, state::Account};

#[tokio::test]
async fn sol_staker_stake_tokens_simple() {
    let mut context = setup().await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_000 tokens.
    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &rewards_manager.token_account,
        6_500_000_000,
        0,
    )
    .await
    .unwrap();

    // And we create the holder rewards account for the vault account.
    let vault_holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we stake 6_500_000_000 tokens.
    //
    // - raw amount to be staked: 6_500_000_000
    // - current lamports staked: 5_000_000_000
    // - stake limit: 1.3 * 5_000_000_000 = 6_500_000_000

    let mut stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
        .token_program(spl_token_2022::ID)
        .amount(6_500_000_000) // <- stake 6_500_000_000 tokens
        .instruction();
    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &config_manager.vault,
        &rewards_manager.owner.pubkey(),
        50,
    )
    .await;
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the tokens are staked.
    let account = get_account!(context, sol_staker_staker_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.active_amount, 6_500_000_000);

    // And the vault account has 6_500_000_000 tokens.
    let account = get_account!(context, config_manager.vault);
    let vault = StateWithExtensions::<Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 6_500_000_000);

    // And the config account has 6_500_000_000 tokens delegated (staked).
    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_effective, 6_500_000_000);
}

#[tokio::test]
async fn fail_sol_staker_stake_tokens_with_wrong_vault_account() {
    let mut context = setup().await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_000 tokens.
    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &rewards_manager.token_account,
        6_500_000_000,
        0,
    )
    .await
    .unwrap();

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
    let vault_holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &wrong_vault.pubkey(),
    )
    .await;

    // When we try to stake tokens to the fake vault account.
    let mut stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(wrong_vault.pubkey()) // <- wrong vault account
        .vault_holder_rewards(vault_holder_rewards)
        .token_program(spl_token_2022::ID)
        .amount(6_500_000_000)
        .instruction();
    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &wrong_vault.pubkey(),
        &rewards_manager.owner.pubkey(),
        6_500_000_000,
    )
    .await;
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
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
async fn fail_sol_staker_stake_tokens_with_wrong_config_account() {
    let mut context = setup().await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_000 tokens.
    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &rewards_manager.token_account,
        6_500_000_000,
        0,
    )
    .await
    .unwrap();

    // And we create the holder rewards account for the vault account.
    let vault_holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // And we create another config account.
    let another_config = ConfigManager::new(&mut context).await;

    // When we try to stake tokens using the wrong config account.
    let mut stake_ix = SolStakerStakeTokensBuilder::new()
        .config(another_config.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
        .token_program(spl_token_2022::ID)
        .amount(6_500_000_000) // <- stake 6_500_000_000 tokens
        .instruction();
    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &config_manager.vault,
        &rewards_manager.owner.pubkey(),
        50,
    )
    .await;
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
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
    let mut context = setup().await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_000 tokens.
    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &rewards_manager.token_account,
        6_500_000_000,
        0,
    )
    .await
    .unwrap();

    // And we create the holder rewards account for the vault account.
    let vault_holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we try to stake 0 tokens.
    let mut stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
        .token_program(spl_token_2022::ID)
        .amount(0) // <- 0 tokens
        .instruction();
    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &config_manager.vault,
        &rewards_manager.owner.pubkey(),
        50,
    )
    .await;
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
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
    let mut context = setup().await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_000 tokens.
    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &rewards_manager.token_account,
        6_500_000_000,
        0,
    )
    .await
    .unwrap();

    // And we create the holder rewards account for the vault account.
    let vault_holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

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
    let mut stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
        .token_program(spl_token_2022::ID)
        .amount(6_500_000_000)
        .instruction();
    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &config_manager.vault,
        &rewards_manager.owner.pubkey(),
        50,
    )
    .await;
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
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
    let mut context = setup().await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And we initialize the holder rewards accounts and mint 6_500_000_001 tokens.
    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &rewards_manager.token_account,
        6_500_000_001,
        0,
    )
    .await
    .unwrap();

    // And we create the holder rewards account for the vault account.
    let vault_holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we stake 6_500_000_000 tokens.
    //
    // - raw amount to be staked: 6_500_000_001
    // - current lamports staked: 5_000_000_000
    // - stake limit: 1.3 * 5_000_000_000 = 6_500_000_000
    //
    // Stake amount exceeds the stake limit: 6_500_000_001 > 6_500_000_000

    let mut stake_ix = SolStakerStakeTokensBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .sol_staker_stake_authority(sol_staker_staker_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
        .token_program(spl_token_2022::ID)
        .amount(6_500_000_001) // <- stake 6_500_000_001 tokens
        .instruction();
    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &config_manager.vault,
        &rewards_manager.owner.pubkey(),
        6_500_000_001,
    )
    .await;
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The staker's effective stake is less than the total stake.
    let staker_stake = get_account!(context, sol_staker_staker_manager.stake);
    let staker_stake = SolStakerStake::from_bytes(&staker_stake.data).unwrap();
    assert_eq!(staker_stake.delegation.active_amount, 6_500_000_001);
    assert_eq!(staker_stake.delegation.effective_amount, 6_500_000_000);

    // Assert - The global effective stake is less than the total stake.
    let config = get_account!(context, config_manager.config);
    let config = Config::from_bytes(&config.data).unwrap();
    assert_eq!(config.token_amount_effective, 6_500_000_000);
}
