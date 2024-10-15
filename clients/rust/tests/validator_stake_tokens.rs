#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::ValidatorStakeTokensBuilder,
    pdas::find_validator_stake_pda,
};
use setup::{
    add_extra_account_metas_for_transfer,
    config::ConfigManager,
    rewards::{create_holder_rewards, RewardsManager},
    token::{create_token_account, mint_to, TOKEN_ACCOUNT_EXTENSIONS},
    validator_stake::ValidatorStakeManager,
    vote::create_vote_account,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::AccountSharedData,
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::{
    extension::{PodStateWithExtensions, StateWithExtensions},
    pod::PodAccount,
    state::Account,
};

#[tokio::test]
async fn validator_stake_tokens_simple() {
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

    // Given a config account and a validator stake with 50 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

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
    let holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we stake 65 tokens.
    //
    // - raw amount to be staked: 65
    // - current lamports staked: 50
    // - stake limit: 1.3 * 50 = 65

    let config_state = get_account!(context, config_manager.config);
    let config_state = Config::from_bytes(&config_state.data).unwrap();
    println!("{:?}", config_state);
    let mut stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(holder_rewards)
        .token_program(spl_token_2022::ID)
        .amount(65) // <- stake 65 tokens
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

    // Assert - The tokens are staked.
    let account = get_account!(context, stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.amount, 65);

    // Assert - The vault account has 50 tokens.
    let account = get_account!(context, config_manager.vault);
    let vault = StateWithExtensions::<Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 65);

    // Assert - The config account has 65 tokens delegated (staked).
    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_effective, 65);
}

#[tokio::test]
async fn fail_validator_stake_tokens_with_wrong_vault_account() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &wrong_vault.pubkey(),
    )
    .await;

    // When we try to stake tokens to the fake vault account.
    let mut stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(wrong_vault.pubkey())
        .vault_holder_rewards(holder_rewards)
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
async fn fail_validator_stake_tokens_with_wrong_config_account() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we try to stake tokens using the wrong config account.
    let mut stake_ix = ValidatorStakeTokensBuilder::new()
        .config(another_config.config) // <- wrong config account
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .vault_holder_rewards(holder_rewards)
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .token_program(spl_token_2022::ID)
        .amount(50)
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
async fn fail_validator_stake_tokens_with_zero_amount() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we try to stake 0 tokens.
    let mut stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(holder_rewards)
        .token_program(spl_token_2022::ID)
        .amount(0) // <- 0 tokens
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

#[tokio::test]
async fn fail_validator_stake_tokens_with_uninitialized_stake_account() {
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

    // Given a config.
    let config_manager = ConfigManager::new(&mut context).await;

    // And we initialize a holder rewards accounts and mint 100 tokens.
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
    let holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // And an uninitialized stake account.
    let validator_vote = create_vote_account(
        &mut context,
        &Pubkey::new_unique(),
        &config_manager.authority.pubkey(),
    )
    .await;
    let (stake_pda, _) = find_validator_stake_pda(&validator_vote, &config_manager.config);
    context.set_account(
        &stake_pda,
        &AccountSharedData::from(solana_sdk::account::Account {
            lamports: 100_000_000,
            data: vec![5; ValidatorStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to stake the tokens to the uninitialized stake account.
    let mut stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_pda)
        .validator_stake_authority(Pubkey::new_unique())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(holder_rewards)
        .token_program(spl_token_2022::ID)
        .amount(50)
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

    assert_instruction_error!(err, InstructionError::UninitializedAccount);
}

#[tokio::test]
async fn fail_validator_stake_tokens_without_staked_sol() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    let holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we try to stake 50 tokens on a validator stake account without staked SOL.
    let mut stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(holder_rewards)
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

    // Assert - The tokens should be staked but there should be no effective weight.
    let account = get_account!(context, stake_manager.stake);
    let account = ValidatorStake::from_bytes(&account.data).unwrap();
    assert_eq!(account.total_staked_lamports_amount, 0);
    assert_eq!(account.delegation.effective_amount, 0);
    assert_eq!(account.delegation.amount, 50);
}

#[tokio::test]
async fn validator_stake_tokens_with_insufficient_staked_sol_effective_capped() {
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

    // Given a config account and a validator stake with 2 SOL staked.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.total_staked_lamports_amount = 2_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

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
        2_600_000_001,
        0, // decimals
    )
    .await
    .unwrap();

    // And we create the holder rewards account for the vault account.
    let holder_rewards = create_holder_rewards(
        &mut context,
        &rewards_manager.pool,
        &config_manager.mint,
        &config_manager.vault,
    )
    .await;

    // When we try to stake 50 tokens on a validator stake account with only 2 SOL staked.
    //
    // - raw amount to be staked: 2_600_000_001
    // - current lamports (SOL) staked: 2_000_000_000
    // - stake limit: 1.3 * 2_000_000_000 = 2_600_000_000
    //
    // Stake amount exceeds the limit: 2_600_000_001 > 2_600_000_000

    let token = get_account!(context, rewards_manager.token_account);
    let token = PodStateWithExtensions::<PodAccount>::unpack(&token.data).unwrap();

    let mut stake_ix = ValidatorStakeTokensBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .source_token_account(rewards_manager.token_account)
        .token_account_authority(rewards_manager.owner.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(holder_rewards)
        .token_program(spl_token_2022::ID)
        .amount(2_600_000_001) // <- stake 2_600_000_001 tokens (raw value with 9 digits precision)
        .instruction();
    add_extra_account_metas_for_transfer(
        &mut context,
        &mut stake_ix,
        &paladin_rewards_program_client::ID,
        &rewards_manager.token_account,
        &config_manager.mint,
        &config_manager.vault,
        &rewards_manager.owner.pubkey(),
        2_600_000_001,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &[stake_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &rewards_manager.owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The tokens should be staked but their effective weight is capped.
    let account = get_account!(context, stake_manager.stake);
    let account = ValidatorStake::from_bytes(&account.data).unwrap();
    assert_eq!(
        account.total_staked_lamports_amount,
        stake_account.total_staked_lamports_amount
    );
    assert_eq!(account.delegation.effective_amount, 2_600_000_000);
    assert_eq!(account.delegation.amount, 2_600_000_001);
}
