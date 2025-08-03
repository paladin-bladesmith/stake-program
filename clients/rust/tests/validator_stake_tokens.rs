#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::{Config, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::ValidatorStakeTokensBuilder,
    pdas::find_validator_stake_pda,
};
use setup::{
    config::ConfigManager, rewards::create_holder_rewards, setup, token::mint_to,
    validator_stake::ValidatorStakeManager, vote::create_vote_account,
};
use solana_program_test::tokio;
use solana_sdk::{
    account::AccountSharedData,
    instruction::InstructionError,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as TokenAccount;

use crate::setup::{config::create_ata, sign_duna_document};

#[tokio::test]
async fn validator_stake_tokens_simple() {
    let mut context = setup(&[]).await;

    // Given a config account and a validator stake with 50 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we initialize the holder rewards accounts and mint 100 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        100,
    )
    .await
    .unwrap();

    // When we stake 65 tokens.
    //
    // - raw amount to be staked: 65
    // - current lamports staked: 50
    // - stake limit: 1.3 * 50 = 65
    let stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(65) // <- stake 65 tokens
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.rewards_manager.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The tokens are staked.
    let account = get_account!(context, stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.staked_amount, 65);

    // Assert - The vault account has 0 tokens. (they were deposited to holder rewards program)
    let account = get_account!(context, config_manager.vault);
    let vault = TokenAccount::unpack(&account.data).unwrap();
    assert_eq!(vault.amount, 0);

    // Assert - The config account has 65 tokens delegated (staked).
    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_effective, 65);

    // Assert rewards program pool have 65 balance
    let vault = get_account!(context, config_manager.rewards_manager.pool_token_account);
    let vault_token_account = TokenAccount::unpack(&vault.data).unwrap();
    assert_eq!(vault_token_account.amount, 65);

    // assert vault holder rewards account have 65 deposited
    let vault_holder_rewards_account = get_account!(context, config_manager.vault_holder_rewards);
    let vault_holder_rewards =
        HolderRewards::from_bytes(&vault_holder_rewards_account.data).unwrap();
    assert_eq!(vault_holder_rewards.deposited, 65)
}

#[tokio::test]
async fn fail_validator_stake_tokens_with_wrong_vault_holder_rewards_account() {
    let mut context = setup(&[]).await;

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts and mint 100 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        100,
    )
    .await
    .unwrap();

    // And we create a fake vault token account.
    let wrong_vault_pda = Keypair::new();
    sign_duna_document(&mut context, &wrong_vault_pda.pubkey());
    let wrong_vault = get_associated_token_address(&wrong_vault_pda.pubkey(), &config_manager.mint);
    let wrong_vault_holder_rewards = HolderRewards::find_pda(&wrong_vault_pda.pubkey()).0;
    create_holder_rewards(
        &mut context,
        &config_manager.rewards_manager.pool,
        &config_manager.mint,
        wrong_vault_pda.insecure_clone(),
    )
    .await;
    create_ata(
        &mut context,
        &wrong_vault_pda.pubkey(),
        &config_manager.mint,
    )
    .await
    .unwrap();

    // When we try to stake tokens to the fake vault account.
    let stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(wrong_vault)
        .vault_pda(wrong_vault_pda.pubkey())
        .vault_holder_rewards(wrong_vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(50) // <- stake 50 tokens
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
async fn fail_validator_stake_tokens_with_wrong_vault_pda_account() {
    let mut context = setup(&[]).await;

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts and mint 100 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        100,
    )
    .await
    .unwrap();

    // And we create a fake vault token account.
    let wrong_vault_pda = Keypair::new();

    // When we try to stake tokens to the fake vault account.
    let stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(wrong_vault_pda.pubkey())
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(50) // <- stake 50 tokens
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
    assert_custom_error!(err, PaladinStakeProgramError::IncorrectVaultPdaAccount);
}

#[tokio::test]
async fn fail_validator_stake_tokens_with_wrong_vault_token_account() {
    let mut context = setup(&[]).await;

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts and mint 100 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        100,
    )
    .await
    .unwrap();

    // And we create a fake vault token account.
    let wrong_vault_pda = Keypair::new();
    let wrong_vault = get_associated_token_address(&wrong_vault_pda.pubkey(), &config_manager.mint);
    create_ata(
        &mut context,
        &wrong_vault_pda.pubkey(),
        &config_manager.mint,
    )
    .await
    .unwrap();

    // When we try to stake tokens to the fake vault account.
    let stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(wrong_vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(50) // <- stake 50 tokens
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
    assert_custom_error!(err, PaladinStakeProgramError::IncorrectVaultAccount);
}

#[tokio::test]
async fn fail_validator_stake_tokens_with_wrong_config_account() {
    let mut context = setup(&[]).await;

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts and mint 100 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        100,
    )
    .await
    .unwrap();

    // And we create another config account.
    let another_config = ConfigManager::new(&mut context).await;

    // When we try to stake tokens using the wrong config account.
    let stake_ix = ValidatorStakeTokensBuilder::new()
        .config(another_config.config) // <- wrong config account
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(50)
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
async fn fail_validator_stake_tokens_with_zero_amount() {
    let mut context = setup(&[]).await;

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts and mint 100 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        100,
    )
    .await
    .unwrap();

    // When we try to stake 0 tokens.
    let stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
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
async fn fail_validator_stake_tokens_with_uninitialized_stake_account() {
    let mut context = setup(&[]).await;

    // Given a config.
    let config_manager = ConfigManager::new(&mut context).await;

    // And we initialize a holder rewards accounts and mint 100 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        100,
    )
    .await
    .unwrap();

    // And an uninitialized stake account.
    let validator_vote = create_vote_account(
        &mut context,
        &Pubkey::new_unique(),
        &config_manager.mint_authority.pubkey(),
    )
    .await;
    let (stake_pda, _) = find_validator_stake_pda(&validator_vote, &config_manager.config);
    context.set_account(
        &stake_pda,
        &AccountSharedData::from(solana_sdk::account::Account {
            lamports: 100_000_000,
            data: vec![5; ValidatorStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to stake the tokens to the uninitialized stake account.
    let stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .validator_stake(stake_pda)
        .validator_stake_authority(Pubkey::new_unique())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(50)
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
async fn fail_validator_stake_tokens_without_staked_sol() {
    let mut context = setup(&[]).await;

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts and mint 100 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        100,
    )
    .await
    .unwrap();

    // When we try to stake 50 tokens on a validator stake account without staked SOL.
    let stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(50) // <- stake 50 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.rewards_manager.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The tokens should be staked but there should be no effective weight.
    let account = get_account!(context, stake_manager.stake);
    let account = ValidatorStake::from_bytes(&account.data).unwrap();
    assert_eq!(account.total_staked_lamports_amount, 0);
    assert_eq!(account.delegation.effective_amount, 0);
    assert_eq!(account.delegation.staked_amount, 50);
}

#[tokio::test]
async fn validator_stake_tokens_with_insufficient_staked_sol_effective_capped() {
    let mut context = setup(&[]).await;

    // Given a config account and a validator stake with 2 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.total_staked_lamports_amount = 2_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we initialize the holder rewards accounts and mint 100 tokens.
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.rewards_manager.owner_token_account,
        2_600_000_001,
    )
    .await
    .unwrap();

    // When we try to stake 50 tokens on a validator stake account with only 2 SOL staked.
    //
    // - raw amount to be staked: 2_600_000_001
    // - current lamports (SOL) staked: 2_000_000_000
    // - stake limit: 1.3 * 2_000_000_000 = 2_600_000_000
    //
    // Stake amount exceeds the limit: 2_600_000_001 > 2_600_000_000

    let stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .holder_rewards_pool(config_manager.rewards_manager.pool)
        .holder_rewards_pool_token_account(config_manager.rewards_manager.pool_token_account)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(config_manager.rewards_manager.owner_token_account)
        .source_token_account_authority(config_manager.rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_pda(config_manager.vault_pda)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .token_program(spl_token::ID)
        .rewards_program(paladin_rewards_program_client::ID)
        .amount(2_600_000_001) // <- stake 2_600_000_001 tokens (raw value with 9 digits precision)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.rewards_manager.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The tokens should be staked but their effective weight is capped.
    let account = get_account!(context, stake_manager.stake);
    let account = ValidatorStake::from_bytes(&account.data).unwrap();
    assert_eq!(
        account.total_staked_lamports_amount,
        stake_account.total_staked_lamports_amount
    );
    assert_eq!(account.delegation.effective_amount, 2_600_000_000);
    assert_eq!(account.delegation.staked_amount, 2_600_000_001);
}
