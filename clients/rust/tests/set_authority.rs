#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::{InitializeConfigBuilder, SetAuthorityBuilder},
    pdas::find_vault_pda,
    types::AuthorityType,
};
use setup::{
    config::ConfigManager,
    setup,
    sol_staker_stake::SolStakerStakeManager,
    token::{create_mint, create_token_account, MINT_EXTENSIONS, TOKEN_ACCOUNT_EXTENSIONS},
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

#[tokio::test]
async fn set_config_authority_on_config() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given config account with an authority.

    let config_manager = ConfigManager::new(&mut context).await;
    let authority = config_manager.authority;

    // When we set a new authority on the config.

    let new_authority = Keypair::new();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(config_manager.config)
        .authority(authority.pubkey())
        .new_authority(new_authority.pubkey())
        .authority_type(AuthorityType::Config)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the authority is updated.

    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.authority, new_authority.pubkey().into());
}

#[tokio::test]
async fn set_slash_authority_on_config() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given config account with an slash authority.

    let config_manager = ConfigManager::new(&mut context).await;
    let slash_authority = config_manager.authority;

    // When we set a new slash authority on the config.

    let new_slash_authority = Keypair::new();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(config_manager.config)
        .authority(slash_authority.pubkey())
        .new_authority(new_slash_authority.pubkey())
        .authority_type(AuthorityType::Slash)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &slash_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the slash authority is updated.

    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        config_account.slash_authority,
        new_slash_authority.pubkey().into()
    );
}

#[tokio::test]
async fn fail_set_config_authority_with_wrong_authority() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given config account with an authority.

    let config_manager = ConfigManager::new(&mut context).await;

    // When we try to set a new authority with a wrong authority.

    let fake_authority = Keypair::new();
    let new_authority = Keypair::new();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(config_manager.config)
        .authority(fake_authority.pubkey())
        .new_authority(new_authority.pubkey())
        .authority_type(AuthorityType::Config)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
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
async fn fail_set_slash_authority_with_wrong_authority() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account.

    let config_manager = ConfigManager::new(&mut context).await;

    // When we try to set a new authority with a wrong authority.

    let fake_authority = Keypair::new();
    let new_authority = Keypair::new();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(config_manager.config)
        .authority(fake_authority.pubkey())
        .new_authority(new_authority.pubkey())
        .authority_type(AuthorityType::Slash)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
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
async fn fail_set_config_authority_when_authority_none() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an empty config account and a mint.

    let config = Keypair::new();
    let authority = Keypair::new();
    let authority_pubkey = authority.pubkey();

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &authority_pubkey,
        Some(&authority_pubkey),
        0,
        MINT_EXTENSIONS,
    )
    .await
    .unwrap();

    let token = Keypair::new();
    create_token_account(
        &mut context,
        &find_vault_pda(&config.pubkey()).0,
        &token,
        &mint.pubkey(),
        TOKEN_ACCOUNT_EXTENSIONS,
    )
    .await
    .unwrap();

    let create_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &config.pubkey(),
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Config::LEN),
        Config::LEN as u64,
        &paladin_stake_program_client::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(Pubkey::default()) // <- None
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // And we initialize a config.

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, config.pubkey());
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.authority, Pubkey::default().into());

    // When we try to set a new config authority when authority is None.

    let new_authority = Keypair::new();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(config.pubkey())
        .authority(authority_pubkey)
        .new_authority(new_authority.pubkey())
        .authority_type(AuthorityType::Config)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
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

    assert_custom_error!(err, PaladinStakeProgramError::AuthorityNotSet);
}

#[tokio::test]
async fn fail_set_slash_authority_when_authority_none() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an empty config account and a mint.

    let config = Keypair::new();
    let authority = Keypair::new();
    let authority_pubkey = authority.pubkey();

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &authority_pubkey,
        Some(&authority_pubkey),
        0,
        MINT_EXTENSIONS,
    )
    .await
    .unwrap();

    let token = Keypair::new();
    create_token_account(
        &mut context,
        &find_vault_pda(&config.pubkey()).0,
        &token,
        &mint.pubkey(),
        TOKEN_ACCOUNT_EXTENSIONS,
    )
    .await
    .unwrap();

    let create_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &config.pubkey(),
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Config::LEN),
        Config::LEN as u64,
        &paladin_stake_program_client::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority_pubkey)
        .slash_authority(Pubkey::default()) // <- None
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // And we initialize a config.

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, config.pubkey());
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.slash_authority, Pubkey::default().into());

    // When we try to set a new slash authority when slash authority is None.

    let new_slash_authority = Keypair::new();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(config.pubkey())
        .authority(authority_pubkey)
        .new_authority(new_slash_authority.pubkey())
        .authority_type(AuthorityType::Slash)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
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

    assert_custom_error!(err, PaladinStakeProgramError::AuthorityNotSet);
}

#[tokio::test]
async fn set_authority_on_validator_stake() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and validator stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // When we set a new authority on the stake account.

    let new_authority = Pubkey::new_unique();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(validator_stake_manager.stake)
        .authority(validator_stake_manager.authority.pubkey())
        .new_authority(new_authority)
        .authority_type(AuthorityType::Stake)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the stake authority is updated.

    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.authority, new_authority);
}

#[tokio::test]
async fn fail_set_authority_on_validator_stake_with_invalid_authority() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config and validator stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // When we try to set a new authority on the stake account with an invalid authority.

    let fake_authority = Keypair::new();
    let new_authority = Pubkey::new_unique();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(validator_stake_manager.stake)
        .authority(fake_authority.pubkey())
        .new_authority(new_authority)
        .authority_type(AuthorityType::Stake)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
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
async fn set_authority_on_sol_staker_stake() {
    let mut context = setup().await;

    // Given a config, validator stake and SOL staker stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    // When we set a new authority on the stake account.

    let new_authority = Pubkey::new_unique();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(sol_staker_stake_manager.stake)
        .authority(sol_staker_stake_manager.authority.pubkey())
        .new_authority(new_authority)
        .authority_type(AuthorityType::Stake)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the stake authority is updated.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.authority, new_authority);
}

#[tokio::test]
async fn fail_set_authority_on_sol_staker_stake_with_invalid_authority() {
    let mut context = setup().await;

    // Given a config, validator stake and SOL staker stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    // When we try to set a new authority on the stake account with an invalid authority.

    let fake_authority = Keypair::new();
    let new_authority = Pubkey::new_unique();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(sol_staker_stake_manager.stake)
        .authority(fake_authority.pubkey())
        .new_authority(new_authority)
        .authority_type(AuthorityType::Stake)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
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
