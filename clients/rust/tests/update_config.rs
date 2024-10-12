#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake_program_client::{
    accounts::Config,
    errors::PaladinStakeProgramError,
    instructions::{InitializeConfigBuilder, UpdateConfigBuilder},
    pdas::find_vault_pda,
    types::ConfigField,
};
use setup::token::{create_mint, create_token_account, MINT_EXTENSIONS, TOKEN_ACCOUNT_EXTENSIONS};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

#[tokio::test]
async fn update_cooldown_time_config() {
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

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &authority.pubkey(),
        Some(&authority.pubkey()),
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

    // And we create a config.

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
        .config_authority(authority.pubkey())
        .slash_authority(authority.pubkey())
        .mint(mint.pubkey())
        .vault(token.pubkey())
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
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.cooldown_time_seconds, 1);

    // When we update the cooldown time config.

    let ix = UpdateConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority.pubkey())
        .config_field(ConfigField::CooldownTimeSeconds(10)) // 10 seconds
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the cooldown time was updated.

    let account = get_account!(context, config.pubkey());
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.cooldown_time_seconds, 10);
    assert_eq!(config_account.max_deactivation_basis_points, 500);
}

#[tokio::test]
async fn update_max_deactivation_basis_points_config() {
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

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &authority.pubkey(),
        Some(&authority.pubkey()),
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

    // And we create a config.

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
        .config_authority(authority.pubkey())
        .slash_authority(authority.pubkey())
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1)
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
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.cooldown_time_seconds, 1);

    // When we update the max deactivation basis points config.

    let ix = UpdateConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority.pubkey())
        .config_field(ConfigField::MaxDeactivationBasisPoints(1000)) // 10%
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the max deactivation basis pointswas updated.

    let account = get_account!(context, config.pubkey());
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.cooldown_time_seconds, 1);
    assert_eq!(config_account.max_deactivation_basis_points, 1000);
}

#[tokio::test]
async fn update_sync_rewards_lamports() {
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

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &authority.pubkey(),
        Some(&authority.pubkey()),
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

    // And we create a config.

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
        .config_authority(authority.pubkey())
        .slash_authority(authority.pubkey())
        .mint(mint.pubkey())
        .vault(token.pubkey())
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
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.sync_rewards_lamports, 1_000_000);

    // When we update the sync rewards lamports.

    let ix = UpdateConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority.pubkey())
        .config_field(ConfigField::SyncRewardsLamports(200_000_000)) // 0.2 SOL
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the sync rewards lamports field was updated.

    let account = get_account!(context, config.pubkey());
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.cooldown_time_seconds, 1);
    assert_eq!(config_account.max_deactivation_basis_points, 500);
    assert_eq!(config_account.sync_rewards_lamports, 200_000_000);
}

#[tokio::test]
async fn fail_update_max_deactivation_basis_points_config_with_invalid_value() {
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

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &authority.pubkey(),
        Some(&authority.pubkey()),
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

    // And we create a config.

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
        .config_authority(authority.pubkey())
        .slash_authority(authority.pubkey())
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1)
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
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.cooldown_time_seconds, 1);

    // When we try to update the max deactivation basis points config with an
    // invalid value (200%).

    let ix = UpdateConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority.pubkey())
        .config_field(ConfigField::MaxDeactivationBasisPoints(20_000)) // 200%
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
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

    assert_instruction_error!(err, InstructionError::InvalidArgument);
}

#[tokio::test]
async fn fail_update_config_with_wrong_authority() {
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

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &authority.pubkey(),
        Some(&authority.pubkey()),
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

    // And we create a config.

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
        .config_authority(authority.pubkey())
        .slash_authority(authority.pubkey())
        .mint(mint.pubkey())
        .vault(token.pubkey())
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
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.cooldown_time_seconds, 1);

    // When we try to update the config with a wrong authority.
    let fake_authority = Keypair::new();

    let ix = UpdateConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(fake_authority.pubkey())
        .config_field(ConfigField::CooldownTimeSeconds(10)) // 10 seconds
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
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
async fn fail_update_config_non_existing() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an non-existing config account.

    let config = Keypair::new();
    let authority = Keypair::new();

    // And we try to update a non-existing config.

    let ix = UpdateConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority.pubkey())
        .config_field(ConfigField::CooldownTimeSeconds(10))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
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

    assert_instruction_error!(err, InstructionError::InvalidAccountOwner);
}

#[tokio::test]
async fn fail_update_with_uninitialized_config() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an empty config account.

    let config = Keypair::new();
    let authority = Keypair::new();

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

    // When we try to update the empty config.

    let update_ix = UpdateConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority.pubkey())
        .config_field(ConfigField::CooldownTimeSeconds(10))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, update_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config, &authority],
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
async fn fail_update_config_with_no_authority_set() {
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

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &authority.pubkey(),
        Some(&authority.pubkey()),
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

    // And we create a config.

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
        .config_authority(Pubkey::default()) // <- no authority
        .slash_authority(authority.pubkey())
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1)
        .max_deactivation_basis_points(500)
        .sync_rewards_lamports(1_000_000)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, config.pubkey());
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.cooldown_time_seconds, 1);

    // When we try to update the config when no authority is set

    let ix = UpdateConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority.pubkey())
        .config_field(ConfigField::CooldownTimeSeconds(10))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
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
