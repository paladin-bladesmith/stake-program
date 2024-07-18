#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake_program_client::{
    accounts::{Config, Stake},
    errors::PaladinStakeProgramError,
    instructions::StakeTokensBuilder,
};
use setup::{
    add_extra_account_metas_for_transfer,
    config::ConfigManager,
    rewards::{create_holder_rewards, RewardsManager},
    stake::StakeManager,
    token::{create_token_account, mint_to, TOKEN_ACCOUNT_EXTENSIONS},
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::{extension::StateWithExtensions, state::Account};

#[tokio::test]
async fn stake_tokens() {
    let mut program_test = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    );
    program_test.add_program(
        "paladin_rewards_program",
        paladin_rewards_program_client::ID,
        None,
    );
    let mut context = program_test.start_with_context().await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts and mint 100 tokens.

    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &rewards_manager.token_account,
        100,
        0,
    )
    .await
    .unwrap();

    // And we create the holder rewards account for the vault account.

    create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we stake 50 tokens.

    let mut stake_ix = StakeTokensBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .validator_vote(stake_manager.vote)
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- stake 50 tokens
        .instruction();

    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &config_manager.vault,
        &rewards_manager.owner.pubkey(),
        50,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the tokens are staked.

    let account = get_account!(context, stake_manager.stake);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.amount, 50);

    // And the vault account has 50 tokens.

    let account = get_account!(context, config_manager.vault);
    let vault = StateWithExtensions::<Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 50);

    // And the config account has 50 tokens delegated (staked).

    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_delegated, 50);
}

#[tokio::test]
async fn fail_stake_tokens_with_wrong_vault_account() {
    let mut program_test = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    );
    program_test.add_program(
        "paladin_rewards_program",
        paladin_rewards_program_client::ID,
        None,
    );
    let mut context = program_test.start_with_context().await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts and mint 100 tokens.

    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &rewards_manager.token_account,
        100,
        0,
    )
    .await
    .unwrap();

    // And we create a fake vault token account.

    let wrong_vault = Keypair::new();
    create_token_account(
        &mut context,
        &config_manager.authority.pubkey(),
        &wrong_vault,
        &config_manager.mint,
        TOKEN_ACCOUNT_EXTENSIONS,
    )
    .await
    .unwrap();

    create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &wrong_vault.pubkey(),
    )
    .await;

    // When we try to stake tokens to the fake vault account.

    let mut stake_ix = StakeTokensBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .validator_vote(stake_manager.vote)
        .mint(config_manager.mint)
        .vault(wrong_vault.pubkey())
        .token_program(spl_token_2022::ID)
        .amount(50) // <- stake 50 tokens
        .instruction();

    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &wrong_vault.pubkey(),
        &rewards_manager.owner.pubkey(),
        50,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
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
async fn fail_stake_tokens_with_wrong_config_account() {
    let mut program_test = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    );
    program_test.add_program(
        "paladin_rewards_program",
        paladin_rewards_program_client::ID,
        None,
    );
    let mut context = program_test.start_with_context().await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts and mint 100 tokens.

    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &rewards_manager.token_account,
        100,
        0,
    )
    .await
    .unwrap();

    // And we create another config account.

    let another_config = ConfigManager::new(&mut context).await;

    create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we try to stake tokens using the wrong config account.

    let mut stake_ix = StakeTokensBuilder::new()
        .config(another_config.config)
        .stake(stake_manager.stake)
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .validator_vote(stake_manager.vote)
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .token_program(spl_token_2022::ID)
        .amount(50) // <- stake 50 tokens
        .instruction();

    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &config_manager.vault,
        &rewards_manager.owner.pubkey(),
        50,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
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
async fn fail_stake_tokens_with_zero_amount() {
    let mut program_test = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    );
    program_test.add_program(
        "paladin_rewards_program",
        paladin_rewards_program_client::ID,
        None,
    );
    let mut context = program_test.start_with_context().await;

    // Given a config and stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = StakeManager::new(&mut context, &config_manager.config).await;

    // And we initialize the holder rewards accounts and mint 100 tokens.

    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &rewards_manager.token_account,
        100,
        0,
    )
    .await
    .unwrap();

    // And we create the holder rewards account for the vault account.

    create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we try to stake 0 tokens.

    let mut stake_ix = StakeTokensBuilder::new()
        .config(config_manager.config)
        .stake(stake_manager.stake)
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .validator_vote(stake_manager.vote)
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .token_program(spl_token_2022::ID)
        .amount(0) // <- stake 50 tokens
        .instruction();

    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &config_manager.vault,
        &rewards_manager.owner.pubkey(),
        0,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
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
