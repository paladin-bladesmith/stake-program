#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake::{
    accounts::{Config, Stake},
    errors::StakeError,
    instructions::{InitializeConfigBuilder, InitializeStakeBuilder, SetAuthorityBuilder},
    pdas::{find_stake_pda, find_vault_pda},
    types::AuthorityType,
};
use setup::{
    config::create_config,
    token::{create_mint, create_token_account, MINT_EXTENSIONS, TOKEN_ACCOUNT_EXTENSIONS},
    vote::create_vote_account,
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
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
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
        &paladin_stake::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
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
    assert_eq!(config_account.authority, authority_pubkey.into());

    // When we set a new authority on the config.

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
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the authority is updated.

    let account = get_account!(context, config.pubkey());
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.authority, new_authority.pubkey().into());
}

#[tokio::test]
async fn set_slash_authority_on_config() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
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
        &paladin_stake::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
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
    assert_eq!(config_account.authority, authority_pubkey.into());

    // When we set a new slash authority on the config.

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
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the slash authority is updated.

    let account = get_account!(context, config.pubkey());
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        config_account.slash_authority,
        new_slash_authority.pubkey().into()
    );
}

#[tokio::test]
async fn fail_set_config_authority_with_wrong_authority() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
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
        &paladin_stake::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
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
    assert_eq!(config_account.authority, authority_pubkey.into());

    // When we try to set a new authority with a wrong authority.

    let fake_authority = Keypair::new();
    let new_authority = Keypair::new();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(config.pubkey())
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

    assert_custom_error!(err, StakeError::InvalidAuthority);
}

#[tokio::test]
async fn fail_set_slash_authority_with_wrong_authority() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
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
        &paladin_stake::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority_pubkey)
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
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
    assert_eq!(config_account.authority, authority_pubkey.into());

    // When we try to set a new authority with a wrong authority.

    let fake_authority = Keypair::new();
    let new_authority = Keypair::new();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(config.pubkey())
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

    assert_custom_error!(err, StakeError::InvalidAuthority);
}

#[tokio::test]
async fn fail_set_config_authority_when_authority_none() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
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
        &paladin_stake::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(Pubkey::default()) // <- None
        .slash_authority(authority_pubkey)
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
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

    assert_custom_error!(err, StakeError::AuthorityNotSet);
}

#[tokio::test]
async fn fail_set_slash_authority_when_authority_none() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
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
        &paladin_stake::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority_pubkey)
        .slash_authority(Pubkey::default()) // <- None
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
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

    assert_custom_error!(err, StakeError::AuthorityNotSet);
}

#[tokio::test]
async fn set_authority_on_stake() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let withdraw_authority = Keypair::new();
    let validator_vote =
        create_vote_account(&mut context, &validator, &withdraw_authority.pubkey()).await;

    // And we initialize the stake account.

    let (stake_pda, _) = find_stake_pda(&validator_vote, &config);

    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Stake::LEN),
    );

    let initialize_ix = InitializeStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .validator_vote(validator_vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, stake_pda);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.authority, withdraw_authority.pubkey());

    // When we set a new authority on the stake account.

    let new_authority = Pubkey::new_unique();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(stake_pda)
        .authority(withdraw_authority.pubkey())
        .new_authority(new_authority)
        .authority_type(AuthorityType::Stake)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[set_authority_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &withdraw_authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the stake authority is updated.

    let account = get_account!(context, stake_pda);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.authority, new_authority);
}

#[tokio::test]
async fn fail_set_authority_on_stake_with_invalid_authority() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let withdraw_authority = Keypair::new();
    let validator_vote =
        create_vote_account(&mut context, &validator, &withdraw_authority.pubkey()).await;

    // And we initialize the stake account.

    let (stake_pda, _) = find_stake_pda(&validator_vote, &config);

    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Stake::LEN),
    );

    let initialize_ix = InitializeStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .validator_vote(validator_vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, stake_pda);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.authority, withdraw_authority.pubkey());

    // When we try to set a new authority on the stake account with an invalid authority.

    let fake_authority = Keypair::new();
    let new_authority = Pubkey::new_unique();

    let set_authority_ix = SetAuthorityBuilder::new()
        .account(stake_pda)
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

    assert_custom_error!(err, StakeError::InvalidAuthority);
}
