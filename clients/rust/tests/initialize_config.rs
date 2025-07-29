#![cfg(feature = "test-sbf")]

mod setup;

use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::Config, errors::PaladinStakeProgramError, instructions::InitializeConfigBuilder,
    pdas::find_vault_pda,
};
use setup::token::create_mint;
use solana_program_test::tokio;
use solana_sdk::{
    account::Account,
    instruction::InstructionError,
    program_option::COption,
    program_pack::Pack,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::{Account as TokenAccount, Mint};

use crate::setup::{
    config::{create_ata, fund_account, get_duna_hash},
    rewards::RewardsManager,
    setup,
    token::mint_to_instruction,
};

#[tokio::test]
async fn initialize_config_with_mint_and_token() {
    let mut context = setup(&[]).await;

    // Given an empty config account with an associated vault and a mint.
    let config = Keypair::new();
    let mint_authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &mint_authority,
        Some(&mint_authority),
        6,
    )
    .await
    .unwrap();

    // Create vault DPA
    let (vault_pda, _) = find_vault_pda(&config.pubkey());
    let vault = get_associated_token_address(&vault_pda, &mint.pubkey());

    let rewards_manager = RewardsManager::new(&mut context, &mint.pubkey()).await;
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&vault_pda);

    // Fund vault pda
    fund_account(&mut context, &vault_pda, 0).await.unwrap();
    // Fund vault holder rewards
    fund_account(&mut context, &vault_holder_rewards, HolderRewards::LEN)
        .await
        .unwrap();
    // create vault ATA
    create_ata(&mut context, &vault_pda, &mint.pubkey())
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
        &paladin_stake_program::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .mint(mint.pubkey())
        .holder_rewards_pool(rewards_manager.pool)
        .holder_rewards_pool_token_account(rewards_manager.pool_token_account)
        .vault(vault)
        .vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .rewards_program(paladin_rewards_program_client::ID)
        .slash_authority(mint_authority)
        .config_authority(mint_authority)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
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
async fn fail_initialize_config_with_wrong_vault_pda() {
    let mut context = setup(&[]).await;

    // Given an empty config account and a mint.

    let config = Keypair::new();
    let wrong_authority = Keypair::new();

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &wrong_authority.pubkey(),
        Some(&wrong_authority.pubkey()),
        6,
    )
    .await
    .unwrap();

    let (vault_pda, _) = find_vault_pda(&wrong_authority.pubkey());
    let vault = get_associated_token_address(&vault_pda, &mint.pubkey());

    let (vault_holder_rewards, _) = HolderRewards::find_pda(&vault_pda);

    let rewards_manager = RewardsManager::new(&mut context, &mint.pubkey()).await;

    // Fund vault pda
    fund_account(&mut context, &vault_pda, 0).await.unwrap();
    // Fund vault holder rewards
    fund_account(&mut context, &vault_holder_rewards, HolderRewards::LEN)
        .await
        .unwrap();
    // create vault ATA
    create_ata(&mut context, &vault_pda, &mint.pubkey())
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
        .holder_rewards_pool(rewards_manager.pool)
        .holder_rewards_pool_token_account(rewards_manager.pool_token_account)
        .mint(mint.pubkey())
        .vault(vault)
        .vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .rewards_program(paladin_rewards_program_client::ID)
        .config_authority(wrong_authority.pubkey())
        .slash_authority(wrong_authority.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
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
    assert_custom_error!(err, PaladinStakeProgramError::IncorrectVaultPdaAccount);
}

#[tokio::test]
async fn fail_initialize_config_with_non_empty_token() {
    let mut context = setup(&[]).await;

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

    let (vault_pda, _) = find_vault_pda(&config.pubkey());
    let vault = get_associated_token_address(&vault_pda, &mint.pubkey());

    let (vault_holder_rewards, _) = HolderRewards::find_pda(&vault_pda);
    let rewards_manager = RewardsManager::new(&mut context, &mint.pubkey()).await;

    // And we mint a token.
    let mint_to_ix = mint_to_instruction(&mut context, &mint.pubkey(), &authority, &vault, 1)
        .await
        .unwrap();

    // Fund vault pda
    fund_account(&mut context, &vault_pda, 0).await.unwrap();
    // Fund vault holder rewards
    fund_account(&mut context, &vault_holder_rewards, HolderRewards::LEN)
        .await
        .unwrap();
    // create vault ATA
    create_ata(&mut context, &vault_pda, &mint.pubkey())
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
        .holder_rewards_pool(rewards_manager.pool)
        .holder_rewards_pool_token_account(rewards_manager.pool_token_account)
        .mint(mint.pubkey())
        .vault(vault)
        .vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .rewards_program(paladin_rewards_program_client::ID)
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
        .instruction();

    // When we try to initialize the config with a non-empty token account.
    let tx = Transaction::new_signed_with_payer(
        &[mint_to_ix, create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority, &config],
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
    let mut context = setup(&[]).await;

    // Given an empty config account and a mint.

    let config = Keypair::new();
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, &authority, Some(&authority), 0)
        .await
        .unwrap();

    let (vault_pda, _) = find_vault_pda(&config.pubkey());
    let vault = get_associated_token_address(&vault_pda, &mint.pubkey());

    let (vault_holder_rewards, _) = HolderRewards::find_pda(&vault_pda);

    let rewards_manager = RewardsManager::new(&mut context, &mint.pubkey()).await;

    let random_mint = Keypair::new();
    let create_mint_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &random_mint.pubkey(),
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Mint::LEN),
        Mint::LEN as u64,
        &spl_token::ID,
    );

    // Fund vault pda
    fund_account(&mut context, &vault_pda, 0).await.unwrap();
    // Fund vault holder rewards
    fund_account(&mut context, &vault_holder_rewards, HolderRewards::LEN)
        .await
        .unwrap();
    // create vault ATA
    create_ata(&mut context, &vault_pda, &mint.pubkey())
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
        .holder_rewards_pool(rewards_manager.pool)
        .holder_rewards_pool_token_account(rewards_manager.pool_token_account)
        .mint(random_mint.pubkey())
        .vault(vault)
        .vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .rewards_program(paladin_rewards_program_client::ID)
        .config_authority(authority)
        .slash_authority(authority)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
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
    let mut context = setup(&[]).await;

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

    let (vault_pda, _) = find_vault_pda(&config.pubkey());
    let vault = get_associated_token_address(&vault_pda, &mint.pubkey());

    let (vault_holder_rewards, _) = HolderRewards::find_pda(&vault_pda);

    let rewards_manager = RewardsManager::new(&mut context, &mint.pubkey()).await;

    // Fund vault pda
    fund_account(&mut context, &vault_pda, 0).await.unwrap();
    // Fund vault holder rewards
    fund_account(&mut context, &vault_holder_rewards, HolderRewards::LEN)
        .await
        .unwrap();
    // create vault ATA
    create_ata(&mut context, &vault_pda, &mint.pubkey())
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
            .minimum_balance(Config::LEN + 100),
        (Config::LEN + 100) as u64, // <-- wrong length
        &paladin_stake_program_client::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .holder_rewards_pool(rewards_manager.pool)
        .holder_rewards_pool_token_account(rewards_manager.pool_token_account)
        .mint(mint.pubkey())
        .vault(vault)
        .vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .rewards_program(paladin_rewards_program_client::ID)
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
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
    let mut context = setup(&[]).await;

    // Given an empty config account with an associated vault and a mint.

    let config = Keypair::new();
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, &authority, Some(&authority), 0)
        .await
        .unwrap();

    let (vault_pda, _) = find_vault_pda(&config.pubkey());
    let vault = get_associated_token_address(&vault_pda, &mint.pubkey());

    let (vault_holder_rewards, _) = HolderRewards::find_pda(&vault_pda);

    let rewards_manager = RewardsManager::new(&mut context, &mint.pubkey()).await;

    // Fund vault pda
    fund_account(&mut context, &vault_pda, 0).await.unwrap();
    // Fund vault holder rewards
    fund_account(&mut context, &vault_holder_rewards, HolderRewards::LEN)
        .await
        .unwrap();
    // create vault ATA
    create_ata(&mut context, &vault_pda, &mint.pubkey())
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

    // And we initialize a config.
    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .holder_rewards_pool(rewards_manager.pool)
        .holder_rewards_pool_token_account(rewards_manager.pool_token_account)
        .mint(mint.pubkey())
        .vault(vault)
        .vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .rewards_program(paladin_rewards_program_client::ID)
        .config_authority(authority)
        .slash_authority(authority)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
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
        .holder_rewards_pool(rewards_manager.pool)
        .holder_rewards_pool_token_account(rewards_manager.pool_token_account)
        .mint(mint.pubkey())
        .vault(vault)
        .vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .rewards_program(paladin_rewards_program_client::ID)
        .config_authority(authority)
        .slash_authority(authority)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
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
    let mut context = setup(&[]).await;

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

    let (vault_pda, _) = find_vault_pda(&config.pubkey());
    let vault = get_associated_token_address(&vault_pda, &mint.pubkey());

    let (vault_holder_rewards, _) = HolderRewards::find_pda(&vault_pda);

    let rewards_manager = RewardsManager::new(&mut context, &mint.pubkey()).await;

    // Fund vault pda
    fund_account(&mut context, &vault_pda, 0).await.unwrap();
    // Fund vault holder rewards
    fund_account(&mut context, &vault_holder_rewards, HolderRewards::LEN)
        .await
        .unwrap();
    // create vault ATA
    create_ata(&mut context, &vault_pda, &mint.pubkey())
        .await
        .unwrap();

    let account = get_account!(context, vault);
    let mut data = account.data;

    // And a token account with a delegate.
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
    context.set_account(&vault, &delegated_token_account.into());

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
        .holder_rewards_pool(rewards_manager.pool)
        .holder_rewards_pool_token_account(rewards_manager.pool_token_account)
        .mint(mint.pubkey())
        .vault(vault)
        .vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .rewards_program(paladin_rewards_program_client::ID)
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
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
    let mut context = setup(&[]).await;

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

    let (vault_pda, _) = find_vault_pda(&config.pubkey());
    let vault = get_associated_token_address(&vault_pda, &mint.pubkey());

    let (vault_holder_rewards, _) = HolderRewards::find_pda(&vault_pda);

    let rewards_manager = RewardsManager::new(&mut context, &mint.pubkey()).await;

    // Fund vault pda
    fund_account(&mut context, &vault_pda, 0).await.unwrap();
    // Fund vault holder rewards
    fund_account(&mut context, &vault_holder_rewards, HolderRewards::LEN)
        .await
        .unwrap();
    // create vault ATA
    create_ata(&mut context, &vault_pda, &mint.pubkey())
        .await
        .unwrap();

    let account = get_account!(context, vault);
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
    context.set_account(&vault, &closeable_token_account.into());

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
        .holder_rewards_pool(rewards_manager.pool)
        .holder_rewards_pool_token_account(rewards_manager.pool_token_account)
        .mint(mint.pubkey())
        .vault(vault)
        .vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .rewards_program(paladin_rewards_program_client::ID)
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
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
    let mut context = setup(&[]).await;

    // Given an empty config account with an associated vault and a mint.

    let config = Keypair::new();
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, &authority, Some(&authority), 0)
        .await
        .unwrap();

    let (vault_pda, _) = find_vault_pda(&config.pubkey());
    let vault = get_associated_token_address(&vault_pda, &mint.pubkey());

    let (vault_holder_rewards, _) = HolderRewards::find_pda(&vault_pda);

    let rewards_manager = RewardsManager::new(&mut context, &mint.pubkey()).await;

    // Fund vault pda
    fund_account(&mut context, &vault_pda, 0).await.unwrap();
    // Fund vault holder rewards
    fund_account(&mut context, &vault_holder_rewards, HolderRewards::LEN)
        .await
        .unwrap();
    // create vault ATA
    create_ata(&mut context, &vault_pda, &mint.pubkey())
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
        .holder_rewards_pool(rewards_manager.pool)
        .holder_rewards_pool_token_account(rewards_manager.pool_token_account)
        .mint(mint.pubkey())
        .vault(vault)
        .vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .rewards_program(paladin_rewards_program_client::ID)
        .config_authority(authority)
        .slash_authority(authority)
        .cooldown_time_seconds(1)
        .max_deactivation_basis_points(20_000) // <- invalid (200%)
        .sync_rewards_lamports(1_000_000)
        .duna_document_hash(get_duna_hash())
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
