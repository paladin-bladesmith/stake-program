#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::SlashSolStakerStakeBuilder,
    pdas::{find_sol_staker_stake_pda, find_vault_pda},
    NullableU64,
};
use setup::{
    config::ConfigManager,
    setup,
    sol_staker_stake::SolStakerStakeManager,
    token::{create_token_account, mint_to, TOKEN_ACCOUNT_EXTENSIONS},
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::tokio;
use solana_sdk::{
    account::{Account, AccountSharedData},
    clock::Clock,
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::extension::StateWithExtensions;

#[tokio::test]
async fn slash_sol_staker_stake() {
    let mut context = setup().await;

    // Given a config account with 100 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 100 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.total_staked_token_amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with 50 tokens staked.

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
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we slash 50 tokens.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- slash 50 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the tokens are burned.

    let account = get_account!(context, config_manager.vault);
    let vault =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 50);

    // And the config account has 50 tokens delegated (staked).

    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_delegated, 50);

    // And the validator stake account has 50 staked tokens.

    let account = get_account!(context, validator_stake_manager.stake);
    let stake = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.total_staked_token_amount, 50);

    // And the slashed sol staker stake account has no delegated tokens.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.delegation.amount, 0);
}

#[tokio::test]
async fn fail_slash_sol_staker_stake_with_zero_amount() {
    let mut context = setup().await;

    // Given a config account with 100 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 100 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.total_staked_token_amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with 50 tokens staked.

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
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we try to slash 0 tokens.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(0) // <- 0 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
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
async fn slash_sol_staker_stake_with_no_staked_amount() {
    let mut context = setup().await;

    // Given a config account with 100 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 100 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.total_staked_token_amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with no tokens staked.

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    // When we slash 50 tokens from the sol staker stake account without staked tokens.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- 50 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the slash is successful but not token is burned.

    let account = get_account!(context, config_manager.vault);
    let vault =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 100);
}

#[tokio::test]
async fn fail_slash_sol_staker_stake_with_invalid_slash_authority() {
    let mut context = setup().await;

    // Given a config account with 100 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 100 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.total_staked_token_amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with 50 tokens staked.

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
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we try to slash with a "fake" slash authority.

    let fake_authority = Keypair::new();

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(fake_authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- slash 50 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
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
async fn fail_slash_sol_staker_stake_with_incorrect_vault_account() {
    let mut context = setup().await;

    // Given a config account with 100 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 100 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.total_staked_token_amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with 50 tokens staked.

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
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And we create a fake vault account with 100 tokens.

    let fake_vault_account = Keypair::new();
    create_token_account(
        &mut context,
        &config_manager.config,
        &fake_vault_account,
        &config_manager.mint,
        TOKEN_ACCOUNT_EXTENSIONS,
    )
    .await
    .unwrap();

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &fake_vault_account.pubkey(),
        100,
        0,
    )
    .await
    .unwrap();

    // When we try to slash with the "fake" vault account.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(fake_vault_account.pubkey())
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- slash 50 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
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
async fn fail_slash_sol_staker_stake_with_uninitialized_stake_account() {
    let mut context = setup().await;

    // Given a config account with 100 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 100 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.total_staked_token_amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And an uninitialized sol staker stake account.

    let stake_state = Pubkey::new_unique();
    let (stake, _) = find_sol_staker_stake_pda(&stake_state, &config_manager.config);

    context.set_account(
        &stake,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; SolStakerStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to slash with an uninitialized stake account.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- slash 50 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
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
async fn fail_slash_sol_staker_stake_with_uninitialized_config_account() {
    let mut context = setup().await;

    // Given a config account with 100 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 100 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.total_staked_token_amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with 50 tokens staked.

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
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

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

    // When we try to slash with an uninitialized config account.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- slash 50 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
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
async fn fail_slash_validator_stake_with_wrong_config_account() {
    let mut context = setup().await;

    // Given a config account with 100 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 100 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.total_staked_token_amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with 50 tokens staked.

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
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And we create a new config account.

    let another_config_manager = ConfigManager::new(&mut context).await;

    // When we try to slash with the wrong config account.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(another_config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(another_config_manager.authority.pubkey())
        .vault(another_config_manager.vault)
        .mint(another_config_manager.mint)
        .vault_authority(find_vault_pda(&another_config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- slash 50 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &another_config_manager.authority],
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
async fn fail_slash_sol_staker_stake_with_insufficient_total_amount_delegated() {
    let mut context = setup().await;

    // Given a config account with 100 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 200 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 200
    stake_account.total_staked_token_amount = 200;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with 200 tokens staked.

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
    // "manually" set the amount to 200
    stake_account.delegation.amount = 200;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we try to slash 200 tokens from the stake account.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(200) // <- 200 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error since there are not enough delegated tokens.

    assert_custom_error!(err, PaladinStakeProgramError::InvalidSlashAmount);
}

#[tokio::test]
async fn slash_sol_staker_stake_updating_deactivating_amount() {
    let mut context = setup().await;

    // Given a config account with 75 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 75;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100, // <- 100 tokens (25 are inactive)
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 75 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 75
    stake_account.total_staked_token_amount = 75;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with 50 tokens staked.

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
    // "manually" set the amount to 75, inactive amount to 25 and deactivating
    // amount to 25
    stake_account.delegation.amount = 75;
    stake_account.delegation.inactive_amount = 25;
    stake_account.delegation.deactivating_amount = 25;
    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;
    stake_account.delegation.deactivation_timestamp = NullableU64::from(timestamp as u64);

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we slash 75 tokens from the stake account.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(75) // <- 75 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then 75 tokens are burned.

    let account = get_account!(context, config_manager.vault);
    let vault =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 25);

    // And the config account has no tokens delegated.

    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_delegated, 0);

    // And the validator stake account has no tokens delegated.

    let account = get_account!(context, validator_stake_manager.stake);
    let stake = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.total_staked_token_amount, 0);

    // And the slashed sol staker stake account has no active/deactivating tokens.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.delegation.amount, 0);
    assert_eq!(stake.delegation.deactivating_amount, 0);
    assert!(stake.delegation.deactivation_timestamp.value().is_none());
    assert_eq!(stake.delegation.inactive_amount, 25);
}

#[tokio::test]
async fn slash_sol_staker_stake_with_insufficient_stake_amount() {
    let mut context = setup().await;

    // Given a config account with 900 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 900;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        1000, // <- 900 tokens + 100 inactive
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 500 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 500
    stake_account.total_staked_token_amount = 500;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with 500 tokens staked and 100 inactive tokens.

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
    // "manually" set the amount to 500
    stake_account.delegation.amount = 500;
    stake_account.delegation.inactive_amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we try to slash 600 tokens from the stake account.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(600) // <- 600 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then only 500 tokens are burned (500 tokens left).

    let account = get_account!(context, config_manager.vault);
    let vault =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 500);

    // And the config account has 400 tokens delegated.

    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_delegated, 400);

    // And the validator stake account has no tokens delegated.

    let account = get_account!(context, validator_stake_manager.stake);
    let stake = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.total_staked_token_amount, 0);

    // And the slashed stake account has no active tokens (100 inactive).

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.delegation.amount, 0);
    assert_eq!(stake.delegation.deactivating_amount, 0);
    assert_eq!(stake.delegation.inactive_amount, 100);
}

#[tokio::test]
async fn fail_slash_sol_staker_stake_with_invalid_validator_stake() {
    let mut context = setup().await;

    // Given a config account with 100 delegated tokens on its vault.

    let config_manager = ConfigManager::new(&mut context).await;

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // And a validator stake account with 100 total tokens staked.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.total_staked_token_amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a sol staker stake account with 50 tokens staked.

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
    // "manually" set the amount to 50
    stake_account.delegation.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And we create a new sol staker account.

    let second_sol_staker_stake = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await
    .stake;

    // When we try to slash with a sol staker as the validator stake account.

    let slash_ix = SlashSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(second_sol_staker_stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- slash 50 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::InvalidAccountData);
}
