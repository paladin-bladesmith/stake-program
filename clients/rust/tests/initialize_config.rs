#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::Config, errors::PaladinStakeProgramError, instructions::InitializeConfigBuilder,
    pdas::find_vault_pda,
};
use setup::{
    setup_holder_rewards,
    token::{create_mint, create_token_account, mint_to},
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::Account,
    instruction::InstructionError,
    program_option::COption,
    program_pack::Pack,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::{Transaction, TransactionError},
};
use spl_token::state::{Account as TokenAccount, Mint};

#[tokio::test]
async fn initialize_config_with_mint_and_token() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an empty config account with an associated vault and a mint.
    let config = Keypair::new();
    let mint_authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &mint_authority,
        Some(&mint_authority),
        0,
    )
    .await
    .unwrap();

    let vault = Keypair::new();
    let vault_authority = find_vault_pda(&config.pubkey()).0;
    create_token_account(&mut context, &vault_authority, &vault, &mint.pubkey())
        .await
        .unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault_authority).await;

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
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .slash_authority(mint_authority)
        .config_authority(mint_authority)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then an account was created with the correct data.
    let account = get_account!(context, config.pubkey());
    assert_eq!(account.data.len(), Config::LEN);
    let config = Config::from_bytes(&account.data).unwrap();
    assert_eq!(config.accumulated_stake_rewards_per_token, 0);
    assert_eq!(config.token_amount_effective, 0);
}

#[tokio::test]
async fn fail_initialize_config_with_wrong_token_authority() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an empty config account and a mint.

    let config = Keypair::new();
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, &authority, Some(&authority), 0)
        .await
        .unwrap();

    let vault = Keypair::new();
    create_token_account(
        &mut context,
        &authority, // <-- wrong authority
        &vault,
        &mint.pubkey(),
    )
    .await
    .unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault.pubkey()).await;

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
        .config_authority(authority)
        .slash_authority(authority)
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // When we try to initialize the config with the wrong token authority.

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.
    assert_custom_error!(err, PaladinStakeProgramError::InvalidTokenOwner);
}

#[tokio::test]
async fn fail_initialize_config_with_non_empty_token() {
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
    )
    .await
    .unwrap();

    let vault = Keypair::new();
    create_token_account(
        &mut context,
        &find_vault_pda(&config.pubkey()).0,
        &vault,
        &mint.pubkey(),
    )
    .await
    .unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault.pubkey()).await;

    // And we mint a token.
    mint_to(&mut context, &mint.pubkey(), &authority, &vault.pubkey(), 1)
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
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // When we try to initialize the config with a non-empty token account.

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::AmountGreaterThanZero);
}

#[tokio::test]
async fn fail_initialize_config_with_unitialized_mint() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an empty config account and a mint.

    let config = Keypair::new();
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, &authority, Some(&authority), 0)
        .await
        .unwrap();

    let vault = Keypair::new();
    let vault_authority = find_vault_pda(&config.pubkey()).0;
    let rent = context.banks_client.get_rent().await.unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault_authority).await;
    create_token_account(&mut context, &vault_authority, &vault, &mint.pubkey())
        .await
        .unwrap();

    let random_mint = Keypair::new();
    let create_mint_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &random_mint.pubkey(),
        rent.minimum_balance(Mint::LEN),
        Mint::LEN as u64,
        &spl_token::ID,
    );

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
        .config_authority(authority)
        .slash_authority(authority)
        .mint(random_mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // When we try to initialize the config with an uninitialized mint.

    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix, create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &random_mint, &config],
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
async fn fail_initialize_config_with_wrong_account_length() {
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
    )
    .await
    .unwrap();

    let vault = Keypair::new();
    let vault_authority = find_vault_pda(&config.pubkey()).0;
    create_token_account(&mut context, &vault_authority, &vault, &mint.pubkey())
        .await
        .unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault_authority).await;

    let create_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &config.pubkey(),
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Config::LEN + 100),
        (Config::LEN + 100) as u64, // <-- wrong length
        &paladin_stake_program_client::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // When we try to initialize the config with an incorrectly-sized account.

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::InvalidAccountDataLength);
}

#[tokio::test]
async fn fail_initialize_config_with_initialized_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an empty config account with an associated vault and a mint.

    let config = Keypair::new();
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, &authority, Some(&authority), 0)
        .await
        .unwrap();

    let vault = Keypair::new();
    let vault_authority = find_vault_pda(&config.pubkey()).0;
    create_token_account(&mut context, &vault_authority, &vault, &mint.pubkey())
        .await
        .unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault_authority).await;

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

    // And we initialize a config.
    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority)
        .slash_authority(authority)
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, config.pubkey());
    assert_eq!(account.data.len(), Config::LEN);

    // When we try to initialize the config again.

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority)
        .slash_authority(authority)
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .cooldown_time_seconds(1)
        .max_deactivation_basis_points(500)
        .sync_rewards_lamports(1_000_000)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
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

    assert_instruction_error!(err, InstructionError::AccountAlreadyInitialized);
}

#[tokio::test]
async fn fail_initialize_config_with_token_delegate() {
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
    )
    .await
    .unwrap();

    // And a token account with a delegate.

    let vault = Keypair::new();
    let vault_authority = find_vault_pda(&config.pubkey()).0;
    create_token_account(&mut context, &vault_authority, &vault, &mint.pubkey())
        .await
        .unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault_authority).await;

    let account = get_account!(context, vault.pubkey());
    let mut data = account.data;

    let mut token_account = TokenAccount::unpack(&mut data).unwrap();
    // "manually" set the delegate
    token_account.delegate = COption::Some(mint.pubkey());
    token_account.delegated_amount = 1u64.into();

    let mut new_data = vec![0; TokenAccount::LEN];
    TokenAccount::pack(token_account, &mut new_data).unwrap();

    let delegated_token_account = Account {
        lamports: account.lamports,
        data: new_data,
        owner: account.owner,
        executable: false,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&vault.pubkey(), &delegated_token_account.into());

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
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // When we try to initialize the config with a delegated token account.

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::DelegateNotNone);
}

#[tokio::test]
async fn fail_initialize_config_with_token_close_authority() {
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
    )
    .await
    .unwrap();

    // And a token account with a close authority.
    let vault = Keypair::new();
    let vault_authority = find_vault_pda(&config.pubkey()).0;
    create_token_account(&mut context, &vault_authority, &vault, &mint.pubkey())
        .await
        .unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault_authority).await;

    let account = get_account!(context, vault.pubkey());
    let mut token_account = TokenAccount::unpack(&account.data).unwrap();
    // "manually" set the close authority
    token_account.close_authority = COption::Some(mint.pubkey());

    let mut new_data = vec![0; TokenAccount::LEN];
    TokenAccount::pack(token_account, &mut new_data).unwrap();

    let closeable_token_account = Account {
        lamports: account.lamports,
        data: new_data,
        owner: account.owner,
        executable: false,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&vault.pubkey(), &closeable_token_account.into());

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
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // When we try to initialize the config with a "closeable" token account.
    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::CloseAuthorityNotNone);
}

#[tokio::test]
async fn fail_initialize_config_with_invalid_max_deactivation_basis_points() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an empty config account with an associated vault and a mint.

    let config = Keypair::new();
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, &authority, Some(&authority), 0)
        .await
        .unwrap();

    let vault = Keypair::new();
    let vault_authority = find_vault_pda(&config.pubkey()).0;
    create_token_account(&mut context, &vault_authority, &vault, &mint.pubkey())
        .await
        .unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault_authority).await;

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
        .config_authority(authority)
        .slash_authority(authority)
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .cooldown_time_seconds(1)
        .max_deactivation_basis_points(20_000) // <- invalid (200%)
        .sync_rewards_lamports(1_000_000)
        .instruction();

    // When we try to initialize the config with an invalid max_deactivation_basis_points value.

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::InvalidArgument);
}

#[tokio::test]
async fn fail_initialize_config_with_invalid_holder_rewards() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an empty config account with an associated vault and a mint.
    let config = Keypair::new();
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, &authority, Some(&authority), 0)
        .await
        .unwrap();

    let vault = Keypair::new();
    let vault_authority = find_vault_pda(&config.pubkey()).0;
    create_token_account(&mut context, &vault_authority, &vault, &mint.pubkey())
        .await
        .unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault_authority).await;

    // Set some deposited amount which should be 0
    let mut vault_holder_rewards_account = get_account!(context, vault_holder_rewards);
    let mut vault_holder_rewards_state =
        HolderRewards::from_bytes(&vault_holder_rewards_account.data).unwrap();
    vault_holder_rewards_state.deposited = 1;
    vault_holder_rewards_account.data = vault_holder_rewards_state.try_to_vec().unwrap();
    context.set_account(&vault_holder_rewards, &vault_holder_rewards_account.into());

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
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .slash_authority(authority)
        .config_authority(authority)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.
    assert_custom_error!(err, PaladinStakeProgramError::InvalidHolderRewards);
}

#[tokio::test]
async fn fail_initialize_config_with_wrong_vault_seeds() {
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
    )
    .await
    .unwrap();

    let vault = Keypair::new();
    let vault_authority = find_vault_pda(&config.pubkey()).0;
    create_token_account(&mut context, &vault_authority, &vault, &mint.pubkey())
        .await
        .unwrap();
    let vault_holder_rewards = setup_holder_rewards(&mut context, &vault.pubkey()).await;

    let create_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &config.pubkey(),
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Config::LEN + 100),
        Config::LEN as u64,
        &paladin_stake_program_client::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(vault.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // When we try to initialize the config with an incorrectly-sized account.

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err()
        .unwrap();

    // Then we expect an error.
    assert_eq!(
        err,
        TransactionError::InstructionError(1, InstructionError::InvalidSeeds)
    );
}
