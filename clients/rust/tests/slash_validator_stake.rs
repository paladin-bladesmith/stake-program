#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::{Config, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::SlashValidatorStakeBuilder,
    pdas::{find_validator_stake_pda, find_vault_pda},
};
use setup::{
    config::ConfigManager,
    token::{create_token_account, mint_to, TOKEN_ACCOUNT_EXTENSIONS},
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::extension::StateWithExtensions;

#[tokio::test]
async fn slash_validator_stake() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
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

    // And we set 50 tokens to the validator stake account.
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
    stake_account.delegation.effective_amount = 50;
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        rent_sponsor: Pubkey::default(),
        rent_debt: 0,
        minimum_balance: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we slash 50 tokens.
    let slash_ix = SlashValidatorStakeBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .slash_authority(config_manager.authority.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
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
    assert_eq!(config.token_amount_effective, 50);

    // And the slashed validator stake account has no tokens.
    let account = get_account!(context, stake_manager.stake);
    let stake = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.delegation.staked_amount, 0);
}

#[tokio::test]
async fn fail_slash_validator_stake_with_zero_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
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

    // And we set 50 tokens to the validator stake account.
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
    stake_account.delegation.effective_amount = 50;
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        rent_sponsor: Pubkey::default(),
        rent_debt: 0,
        minimum_balance: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to slash 0 tokens.
    let slash_ix = SlashValidatorStakeBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .slash_authority(config_manager.authority.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
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
async fn slash_validator_stake_with_no_staked_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and stake accounts (stake account does not have tokens
    // staked).
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
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

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        rent_sponsor: Pubkey::default(),
        rent_debt: 0,
        minimum_balance: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we slash 50 tokens from the validator stake account without staked tokens.
    let slash_ix = SlashValidatorStakeBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .slash_authority(config_manager.authority.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
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
async fn fail_slash_validator_stake_with_invalid_slash_authority() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
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

    // And we set 50 tokens to the validator stake account.
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
    stake_account.delegation.effective_amount = 50;
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        rent_sponsor: Pubkey::default(),
        rent_debt: 0,
        minimum_balance: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to slash with a "fake" slash authority.
    let fake_authority = Keypair::new();
    let slash_ix = SlashValidatorStakeBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .slash_authority(fake_authority.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
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
async fn fail_slash_validator_stake_with_incorrect_vault_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
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

    // And we set 50 tokens to the validator stake account.
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
    stake_account.delegation.effective_amount = 50;
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        rent_sponsor: Pubkey::default(),
        rent_debt: 0,
        minimum_balance: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

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
    let slash_ix = SlashValidatorStakeBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .slash_authority(config_manager.authority.pubkey())
        .mint(config_manager.mint)
        .vault(fake_vault_account.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
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
async fn fail_slash_validator_stake_with_uninitialized_stake_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;

    // And we set 100 tokens to the vault account.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
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

    // And an uninitialized validator stake account.
    let validator_vote = Pubkey::new_unique();
    let (stake, _) = find_validator_stake_pda(&validator_vote, &config_manager.config);
    context.set_account(
        &stake,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; ValidatorStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        rent_sponsor: Pubkey::default(),
        rent_debt: 0,
        minimum_balance: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to slash with an uninitialized stake account.
    let slash_ix = SlashValidatorStakeBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake)
        .validator_stake_authority(Pubkey::new_unique())
        .slash_authority(config_manager.authority.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
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
async fn fail_slash_validator_stake_with_uninitialized_config_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
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

    // And we set 50 tokens to the validator stake account.
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
    stake_account.delegation.effective_amount = 50;
    stake_account.total_staked_lamports_amount = 50;
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

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        rent_sponsor: Pubkey::default(),
        rent_debt: 0,
        minimum_balance: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to slash with an uninitialized config account.
    let slash_ix = SlashValidatorStakeBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .slash_authority(config_manager.authority.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
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
async fn fail_slash_validator_stake_with_wrong_config_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we set 100 tokens to the vault account.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
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

    // And we set 100 tokens to the validator stake account.
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 50;
    stake_account.delegation.effective_amount = 50;
    stake_account.total_staked_lamports_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        rent_sponsor: Pubkey::default(),
        rent_debt: 0,
        minimum_balance: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // And we create a new config account.
    let another_config_manager = ConfigManager::new(&mut context).await;

    // When we try to slash with the wrong config account.
    let slash_ix = SlashValidatorStakeBuilder::new()
        .config(another_config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .slash_authority(another_config_manager.authority.pubkey())
        .mint(another_config_manager.mint)
        .vault(another_config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
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
async fn fail_slash_validator_stake_with_insufficient_total_amount_delegated() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we set 50 tokens to the vault account.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 50;
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

    // And we set 100 tokens to the validator stake account.
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 100;
    stake_account.delegation.effective_amount = 100;
    stake_account.total_staked_lamports_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        rent_sponsor: Pubkey::default(),
        rent_debt: 0,
        minimum_balance: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to slash 100 tokens from the stake account.
    let slash_ix = SlashValidatorStakeBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .slash_authority(config_manager.authority.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
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
    assert_custom_error!(err, 1);
}

#[tokio::test]
async fn slash_validator_stake_with_insufficient_stake_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and validator stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we set 1000 tokens to the vault account.
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 900;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());
    mint_to(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        1000, // <- 900 tokens delegated (100 are inactive)
        0,
    )
    .await
    .unwrap();

    // And we set 500 active tokens and 100 inactive.
    let mut account = get_account!(context, stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 500;
    stake_account.delegation.effective_amount = 500;
    stake_account.total_staked_lamports_amount = 500;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        rent_sponsor: Pubkey::default(),
        rent_debt: 0,
        minimum_balance: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to slash 600 tokens from the stake account.
    let slash_ix = SlashValidatorStakeBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_manager.stake)
        .validator_stake_authority(stake_manager.authority.pubkey())
        .slash_authority(config_manager.authority.pubkey())
        .mint(config_manager.mint)
        .vault(config_manager.vault)
        .vault_holder_rewards(vault_holder_rewards)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .token_program(spl_token_2022::ID)
        .amount(600) // <- 600 tokens
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[slash_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then only 500 tokens are burned (500 tokens left).
    let account = get_account!(context, config_manager.vault);
    let vault =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data).unwrap();
    assert_eq!(vault.base.amount, 500);

    // And the config account has 400 tokens delegated.
    let account = get_account!(context, config_manager.config);
    let config = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config.token_amount_effective, 400);

    // And the slashed stake account has no active tokens (100 inactive).
    let account = get_account!(context, stake_manager.stake);
    let stake = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake.delegation.staked_amount, 0);
}
