#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, Stake},
    errors::PaladinStakeProgramError,
    instructions::SlashBuilder,
    pdas::{find_stake_pda, find_vault_pda},
    NullableU64,
};
use setup::{
    config::ConfigManager,
    stake::StakeManager,
    token::{create_token_account, mint_to, TOKEN_ACCOUNT_EXTENSIONS},
};
use solana_program_test::{tokio, ProgramTest};
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
async fn slash() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.

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

    // And we set 50 tokens to the stake account.

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // When we slash 50 tokens.

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
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

    // And the slashed stake account has no tokens.

    let account = get_account!(context, stake_manager.stake);
    let stake = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.amount, 0);
}

#[tokio::test]
async fn fail_slash_with_zero_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.

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

    // And we set 50 tokens to the stake account.

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // When we try to slash 0 tokens.

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
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
async fn slash_with_no_staked_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts (stake account does not have tokens
    // staked).

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.

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

    // When we slash 50 tokens from the stake account without staked tokens.

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
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
async fn fail_slash_with_invalid_slash_authority() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.

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

    // And we set 50 tokens to the stake account.

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // When we try to slash with a "fake" slash authority.

    let fake_authority = Keypair::new();

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
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
async fn fail_slash_with_incorrect_vault_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.

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

    // And we set 50 tokens to the stake account.

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

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

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
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
async fn fail_slash_with_uninitialized_stake_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;

    // And we set 100 tokens to the vault account.

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

    // And an uninitialized stake account.

    let validator_vote = Pubkey::new_unique();
    let (stake, _) = find_stake_pda(&validator_vote, &config_manager.config);

    context.set_account(
        &stake,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; Stake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to slash with an uninitialized stake account.

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake)
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
async fn fail_slash_with_uninitialized_config_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.

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

    // And we set 50 tokens to the stake account.

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

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

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
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
async fn fail_slash_with_wrong_config_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.

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

    // And we set 100 tokens to the stake account.

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50
    stake_account.amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we create a new config account.

    let another_config_manager = ConfigManager::new(&mut context).await;

    // When we try to slash with the wrong config account.

    let slash_ix = SlashBuilder::new()
        .config(another_config_manager.config)
        .stake(stake_manager.stake)
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
async fn slash_using_inactive_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.

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

    // And we set 100 tokens to the stake account (50 staked and 50 inactive).

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50 and inactive amount to 50
    stake_account.amount = 50;
    stake_account.inactive_amount = 50;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // When we slash 100 tokens from the stake account.

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(100) // <- 100 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then all tokens are burned.

    let account = get_account!(context, config_manager.vault);
    let vault =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 0);

    // And the config account has no tokens delegated.

    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_delegated, 0);

    // And the slashed stake account has no tokens.

    let account = get_account!(context, stake_manager.stake);
    let stake = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.amount, 0);
    assert_eq!(stake.inactive_amount, 0);
}

#[tokio::test]
async fn fail_slash_with_insufficient_total_amount_delegated() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 50 tokens to the vault account.

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 50;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        50,
        0,
    )
    .await
    .unwrap();

    // And we set 100 tokens to the stake account.

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.amount = 100;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // When we try to slash 100 tokens from the stake account.

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(100) // <- 100 tokens
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
async fn slash_using_deactivating_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.

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

    // And we set 100 tokens to the stake account (50 staked, 25 inactive and
    // 25 deactivating).

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50, inactive amount to 25 and
    // deactivating amount to 25
    stake_account.amount = 50;
    stake_account.inactive_amount = 25;
    stake_account.deactivating_amount = 25;
    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;
    stake_account.deactivation_timestamp = NullableU64::from(timestamp as u64);

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // When we slash 100 tokens from the stake account.

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(100) // <- 100 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then all tokens are burned.

    let account = get_account!(context, config_manager.vault);
    let vault =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 0);

    // And the config account has no tokens delegated.

    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_delegated, 0);

    // And the slashed stake account has no tokens.

    let account = get_account!(context, stake_manager.stake);
    let stake = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.amount, 0);
    assert_eq!(stake.inactive_amount, 0);
    assert_eq!(stake.deactivating_amount, 0);
    assert!(stake.deactivation_timestamp.value().is_none());
}

#[tokio::test]
async fn slash_with_insufficient_stake_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.

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

    // And we set 75 tokens to the stake account (50 staked and 25 inactive).

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 50, inactive amount to 25
    stake_account.amount = 50;
    stake_account.inactive_amount = 25;

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // When we try to slash 100 tokens from the stake account.

    let slash_ix = SlashBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .slash_authority(config_manager.authority.pubkey())
        .vault(config_manager.vault)
        .mint(config_manager.mint)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(100) // <- 100 tokens
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then only 75 tokens are burned (25 tokens left).

    let account = get_account!(context, config_manager.vault);
    let vault =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 25);

    // And the config account has 25 tokens delegated.

    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_delegated, 25);

    // And the slashed stake account has no tokens.

    let account = get_account!(context, stake_manager.stake);
    let stake = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.amount, 0);
    assert_eq!(stake.inactive_amount, 0);
    assert_eq!(stake.deactivating_amount, 0);
}
