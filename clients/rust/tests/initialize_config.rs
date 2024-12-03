#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake_program_client::{
    accounts::Config, errors::PaladinStakeProgramError, instructions::InitializeConfigBuilder,
    pdas::find_vault_pda,
};
use setup::token::{
    create_mint, create_token_account, mint_to, MINT_EXTENSIONS, TOKEN_ACCOUNT_EXTENSIONS,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token_2022::{
    extension::{
        memo_transfer::instruction::enable_required_transfer_memos, ExtensionType,
        PodStateWithExtensionsMut,
    },
    pod::{PodAccount, PodCOption},
};

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
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(
        &mut context,
        &mint,
        &authority,
        Some(&authority),
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
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .slash_authority(authority)
        .config_authority(authority)
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // When we create a config.

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
    create_mint(
        &mut context,
        &mint,
        &authority,
        Some(&authority),
        0,
        MINT_EXTENSIONS,
    )
    .await
    .unwrap();

    let token = Keypair::new();
    create_token_account(
        &mut context,
        &authority, // <-- wrong authority
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
        .config_authority(authority)
        .slash_authority(authority)
        .mint(mint.pubkey())
        .vault(token.pubkey())
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

    // And we mint a token.

    mint_to(
        &mut context,
        &mint.pubkey(),
        &authority,
        &token.pubkey(),
        1,
        0,
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
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(token.pubkey())
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
async fn fail_initialize_config_without_transfer_hook() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an empty config account and a mint without a transfer hook.

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
        &[], // <-- no transfer hook
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
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // When we try to initialize the config with the mint without a transfer hook.

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

    assert_custom_error!(err, PaladinStakeProgramError::MissingTransferHook);
}

#[tokio::test]
async fn fail_initialize_config_with_unitialized_mint() {
    let context = ProgramTest::new(
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
    let token = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();

    let create_mint_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &mint.pubkey(),
        rent.minimum_balance(Config::LEN),
        Config::LEN as u64,
        &spl_token_2022::ID,
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
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .sync_rewards_lamports(1_000_000) // 0.001 SOL
        .instruction();

    // When we try to initialize the config with an uninitialized mint.

    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix, create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint, &config],
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
            .minimum_balance(Config::LEN + 100),
        (Config::LEN + 100) as u64, // <-- wrong length
        &paladin_stake_program_client::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(token.pubkey())
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
    create_mint(
        &mut context,
        &mint,
        &authority,
        Some(&authority),
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

    // And we initialize a config.

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority)
        .slash_authority(authority)
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
    assert_eq!(account.data.len(), Config::LEN);

    // When we try to initialize the config again.

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority)
        .slash_authority(authority)
        .mint(mint.pubkey())
        .vault(token.pubkey())
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
        MINT_EXTENSIONS,
    )
    .await
    .unwrap();

    // And a token account with a delegate.

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

    let account = get_account!(context, token.pubkey());
    let mut data = account.data;

    let token_account = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut data).unwrap();
    // "manually" set the delegate
    token_account.base.delegate = PodCOption::some(mint.pubkey());
    token_account.base.delegated_amount = 1u64.into();

    let delegated_token_account = Account {
        lamports: account.lamports,
        data,
        owner: account.owner,
        executable: false,
        rent_epoch: account.rent_epoch,
    };
    let account_shared_data: AccountSharedData = delegated_token_account.into();
    context.set_account(&token.pubkey(), &account_shared_data);

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
        .vault(token.pubkey())
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
        MINT_EXTENSIONS,
    )
    .await
    .unwrap();

    // And a token account with a close authority.

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

    let account = get_account!(context, token.pubkey());
    let mut data = account.data;

    let token_account = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut data).unwrap();
    // "manually" set the close authority
    token_account.base.close_authority = PodCOption::some(mint.pubkey());

    let closeable_token_account = Account {
        lamports: account.lamports,
        data,
        owner: account.owner,
        executable: false,
        rent_epoch: account.rent_epoch,
    };
    let account_shared_data: AccountSharedData = closeable_token_account.into();
    context.set_account(&token.pubkey(), &account_shared_data);

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
        .vault(token.pubkey())
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
async fn fail_initialize_config_with_invalid_token_extensions() {
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

    // And a token account with invalid Memo extensions.

    let token = Keypair::new();
    let owner = Keypair::new();

    create_token_account(
        &mut context,
        &owner.pubkey(),
        &token,
        &mint.pubkey(),
        &[
            ExtensionType::MemoTransfer, // <- invalid extension
            ExtensionType::TransferHookAccount,
        ],
    )
    .await
    .unwrap();

    let enable_memo_ix =
        enable_required_transfer_memos(&spl_token_2022::ID, &token.pubkey(), &owner.pubkey(), &[])
            .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[enable_memo_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, token.pubkey());
    let mut data = account.data;

    let token_account = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut data).unwrap();
    // "manually" set the owner of the token account
    token_account.base.owner = find_vault_pda(&config.pubkey()).0;

    let memo_token_account = Account {
        lamports: account.lamports,
        data,
        owner: account.owner,
        executable: false,
        rent_epoch: account.rent_epoch,
    };
    let account_shared_data: AccountSharedData = memo_token_account.into();
    context.set_account(&token.pubkey(), &account_shared_data);

    // When we try to initialize the config with a memo-enabled token account.

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

    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::InvalidTokenAccountExtension);
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
    create_mint(
        &mut context,
        &mint,
        &authority,
        Some(&authority),
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
        .config_authority(authority)
        .slash_authority(authority)
        .mint(mint.pubkey())
        .vault(token.pubkey())
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
