#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::UnstakeTokensBuilder,
    pdas::find_vault_pda,
};
use setup::{
    config::{create_config, ConfigManager},
    rewards::RewardsManager,
    setup,
    sol_staker_stake::SolStakerStakeManager,
    token::{create_associated_token_account, mint_to},
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::{
    account::{Account, AccountSharedData},
    clock::Clock,
    instruction::{AccountMeta, InstructionError},
    pubkey::Pubkey,
    signature::Signer,
    sysvar::SysvarId,
    transaction::Transaction,
};
use spl_token_2022::{extension::PodStateWithExtensions, pod::PodAccount};
use spl_transfer_hook_interface::get_extra_account_metas_address;

struct Fixture {
    config_manager: ConfigManager,
    config_account: Config,
    rewards_manager: RewardsManager,
    sol_staker_stake_manager: SolStakerStakeManager,
    destination_token_account: Pubkey,
}

async fn setup_fixture(context: &mut ProgramTestContext, active_cooldown: Option<u64>) -> Fixture {
    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(context).await;
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // Mint 100 tokens to the config vault.
    mint_to(
        context,
        &config_manager.mint,
        &config_manager.mint_authority,
        &config_manager.vault,
        100,
        0,
    )
    .await
    .unwrap();

    // Setup rewards pool.
    let rewards_manager = RewardsManager::new(
        context,
        &config_manager.mint,
        &config_manager.mint_authority,
    )
    .await;

    // And a validator stake account.
    let validator_stake_manager = ValidatorStakeManager::new(context, &config_manager.config).await;

    // And a SOL staker stake account (amount staked = 100).
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;
    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.staked_amount = 100;
    stake_account.delegation.effective_amount = 100;
    if let Some(cooldown) = active_cooldown {
        let clock: Clock = bincode::deserialize(&get_account!(context, Clock::id()).data).unwrap();
        stake_account.delegation.unstake_cooldown = clock.unix_timestamp as u64 + cooldown;
    }
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // Setup the stake authorities receiving token account.
    let destination_token_account = create_associated_token_account(
        context,
        &sol_staker_stake_manager.authority.pubkey(),
        &config_manager.mint,
    )
    .await;

    Fixture {
        config_manager,
        config_account,
        rewards_manager,
        sol_staker_stake_manager,
        destination_token_account,
    }
}

#[tokio::test]
async fn inactivate_sol_staker_stake_base() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        config_account,
        rewards_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context, None).await;

    // When we move the deactivated amount to inactive (50 tokens).
    let inactivate_ix = UnstakeTokensBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .destination_token_account(destination_token_account)
        .amount(5)
        .add_remaining_accounts(&[
            AccountMeta {
                pubkey: get_extra_account_metas_address(
                    &config_manager.mint,
                    &paladin_rewards_program_client::ID,
                ),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: rewards_manager.pool,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: config_manager.vault_holder_rewards,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: paladin_rewards_program_client::accounts::HolderRewards::find_pda(
                    &destination_token_account,
                )
                .0,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: paladin_rewards_program_client::ID,
                is_signer: false,
                is_writable: false,
            },
        ])
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The inactivation should be successful.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.staked_amount, 95);
    assert_eq!(stake_account.delegation.effective_amount, 95);
    let clock: Clock = bincode::deserialize(&get_account!(context, Clock::id()).data).unwrap();
    assert_eq!(
        stake_account.delegation.unstake_cooldown,
        clock.unix_timestamp as u64 + config_account.cooldown_time_seconds
    );

    // Assert - The destination token account received the amount.
    let account = get_account!(context, destination_token_account);
    let account = PodStateWithExtensions::<PodAccount>::unpack(&account.data).unwrap();
    assert_eq!(u64::from(account.base.amount), 5);

    // Assert - The total delegated on the config was updated.
    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.token_amount_effective, 95);
}

#[tokio::test]
async fn fail_inactivate_sol_staker_stake_with_wrong_config_for_vault() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context, None).await;

    // And we create a second config.
    let wrong_config = create_config(&mut context).await;

    // When we try to inactivate the stake with the wrong config account.
    let inactivate_ix = UnstakeTokensBuilder::new()
        .config(wrong_config) // <- wrong config
        .stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .destination_token_account(destination_token_account)
        .amount(5)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
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
async fn fail_inactivate_sol_staker_stake_with_wrong_config_for_stake() {
    let mut context = setup(&[]).await;
    let Fixture {
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context, None).await;

    // And we create a second config.
    let wrong_config = ConfigManager::new(&mut context).await;

    // When we try to inactivate the stake with the wrong config account.
    let inactivate_ix = UnstakeTokensBuilder::new()
        .config(wrong_config.config) // <- wrong config
        .stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .vault(wrong_config.vault)
        .vault_authority(find_vault_pda(&wrong_config.config).0)
        .vault_holder_rewards(wrong_config.vault_holder_rewards)
        .mint(wrong_config.mint)
        .destination_token_account(destination_token_account)
        .amount(5)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
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
async fn fail_inactivate_sol_stake_stake_with_uninitialized_stake_account() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context, None).await;

    // Uninitialize the stake account.
    context.set_account(
        &sol_staker_stake_manager.stake,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; SolStakerStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to deactivate from an uninitialized stake account.
    let inactivate_ix = UnstakeTokensBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .destination_token_account(destination_token_account)
        .amount(5)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
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
async fn fail_inactivate_sol_staker_stake_with_active_cooldown() {
    let mut context = setup(&[]).await;
    let Fixture {
        config_manager,
        sol_staker_stake_manager,
        destination_token_account,
        ..
    } = setup_fixture(&mut context, Some(1)).await;

    // When we try to move the deactivated amount to inactive before the end of
    // the cooldown period.
    let inactivate_ix = UnstakeTokensBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .vault(config_manager.vault)
        .vault_authority(find_vault_pda(&config_manager.config).0)
        .vault_holder_rewards(config_manager.vault_holder_rewards)
        .mint(config_manager.mint)
        .destination_token_account(destination_token_account)
        .amount(5)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
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
