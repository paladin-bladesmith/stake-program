#![cfg(feature = "test-sbf")]

mod setup;

use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::Config,
    errors::PaladinStakeProgramError,
    instructions::{InitializeConfigBuilder, SetAuthorityBuilder},
    pdas::find_vault_pda,
    types::AuthorityType,
};
use setup::{config::ConfigManager, setup, token::create_mint};
use solana_program_test::tokio;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

use crate::setup::{
    config::{create_ata, fund_account, get_duna_hash},
    rewards::RewardsManager,
};

#[tokio::test]
async fn set_config_authority_on_config() {
    let mut context = setup(&[]).await;

    // Given config account with an authority.

    let config_manager = ConfigManager::new(&mut context).await;
    let authority = config_manager.config_authority;

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
    let mut context = setup(&[]).await;

    // Given config account with an slash authority.

    let config_manager = ConfigManager::new(&mut context).await;
    let slash_authority = config_manager.config_authority;

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
    let mut context = setup(&[]).await;

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
    let mut context = setup(&[]).await;

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
        .config_authority(Pubkey::default()) // <- None
        .slash_authority(authority_pubkey)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
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
        .slash_authority(Pubkey::default()) // <- None
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .duna_document_hash(get_duna_hash())
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
