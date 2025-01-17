#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::{Config, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::UnstakeTokensBuilder,
    pdas::{find_validator_stake_pda, find_vault_pda},
};
use setup::{
    add_extra_account_metas_for_transfer,
    config::{create_config, ConfigManager},
    pack_to_vec,
    rewards::RewardsManager,
    setup,
    validator_stake::create_validator_stake,
    vote::create_vote_account,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    clock::Clock,
    instruction::InstructionError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::state::{Account as SplAccount, AccountState};
use spl_transfer_hook_interface::get_extra_account_metas_address;

#[tokio::test]
async fn inactivate_validator_stake() {
    let mut context = setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(&mut context).await;
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Setup rewards pool.
    let rewards_manager = RewardsManager::new(
        &mut context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    // And a validator stake account (amount = 100).
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;
    let stake_pda = create_validator_stake(&mut context, &vote, &config_manager.config).await;

    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 100;
    stake_account.delegation.effective_amount = 100;
    stake_account.total_staked_lamports_amount = 100;
    let mut timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    timestamp = timestamp.saturating_sub(config_account.cooldown_time_seconds);
    stake_account.delegation.unstake_cooldown = timestamp;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

    // Setup the validator authorities receiving token account.
    let destination_token_account = Pubkey::new_unique();
    context.set_account(
        &destination_token_account,
        &Account {
            owner: spl_token_2022::ID,
            data: pack_to_vec(SplAccount {
                mint: config_manager.mint,
                owner: authority.pubkey(),
                amount: 0,
                delegate: COption::None,
                delegated_amount: 0,
                state: AccountState::Initialized,
                is_native: COption::None,
                close_authority: COption::None,
            }),
            lamports: Rent::default().minimum_balance(SplAccount::LEN),
            rent_epoch: u64::MAX,
            executable: false,
        }
        .into(),
    );

    // When we move the deactivated amount to inactive (50 tokens).
    let mut inactivate_ix = UnstakeTokensBuilder::new()
        .config(config_manager.config)
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .destination_token_account(destination_token_account)
        .amount(50)
        .instruction();
    add_extra_account_metas_for_transfer(
        &mut context,
        &mut inactivate_ix,
        &paladin_rewards_program_client::ID,
        &config_manager.vault,
        &config_manager.mint,
        &destination_token_account,
        &authority.pubkey(),
        50,
    )
    .await;
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The inactivation should be successful.
    let account = get_account!(context, stake_pda);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.staked_amount, 50);
    assert_eq!(stake_account.delegation.effective_amount, 50);
    assert_eq!(stake_account.delegation.unstake_cooldown, 0);

    // Assert - The total delegated on the config was updated.
    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.token_amount_effective, 50);

    // Assert - The authority token account now has 50 PAL.
    let account = get_account!(context, destination_token_account);
    let account = SplAccount::unpack(&account.data).unwrap();
    assert_eq!(account.amount, 50);
}

/*
#[tokio::test]
async fn fail_inactivate_validator_stake_with_no_deactivated_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (amount = 100).
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;
    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;
    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

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

    // When we try to inactivate the stake without any deactivated amount.
    let inactivate_ix = UnstakeTokensBuilder::new()
        .config(config)
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .vault_holder_rewards(vault_holder_rewards)
        .mint(config_manager.mint)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
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
    assert_custom_error!(err, PaladinStakeProgramError::NoDeactivatedTokens);
}

#[tokio::test]
async fn fail_inactivate_validator_stake_with_wrong_config() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (amount = 100).
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;
    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;
    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

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

    // And we create a second config.
    let wrong_config = create_config(&mut context).await;

    // When we try to inactivate the stake with the wrong config account.
    let inactivate_ix = UnstakeTokensBuilder::new()
        .config(wrong_config) // <- wrong config
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .vault_holder_rewards(vault_holder_rewards)
        .mint(config_manager.mint)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
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
    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_inactivate_validator_stake_with_uninitialized_stake_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account and a validator's vote account.
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;
    let validator = Pubkey::new_unique();

    // And an uninitialized validator stake account.
    let (stake_pda, _) = find_validator_stake_pda(&validator, &config);
    context.set_account(
        &stake_pda,
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

    // When we try to deactivate from an uninitialized stake account.
    let inactivate_ix = UnstakeTokensBuilder::new()
        .config(config)
        .stake(stake_pda)
        .stake_authority(Pubkey::new_unique())
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .vault_holder_rewards(vault_holder_rewards)
        .mint(config_manager.mint)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
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
    assert_instruction_error!(err, InstructionError::UninitializedAccount);
}

#[tokio::test]
async fn fail_inactivate_validator_stake_with_active_cooldown() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::with_args(
        &mut context,
        10,        /* cooldown 10 seconds */
        500,       /* basis points 5%     */
        1_000_000, /* sync rewards lamports 0.001 SOL */
    )
    .await;
    let config = config_manager.config;
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (amount = 100) with 50 tokens deactivated.
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;
    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;

    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 100;
    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;
    stake_account.delegation.unstake_cooldown = timestamp as u64 + 1;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

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

    // When we try to move the deactivated amount to inactive before the end of
    // the cooldown period.
    let inactivate_ix = UnstakeTokensBuilder::new()
        .config(config)
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .vault_holder_rewards(vault_holder_rewards)
        .mint(config_manager.mint)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
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
    assert_custom_error!(err, PaladinStakeProgramError::ActiveUnstakeCooldown);
}
*/
