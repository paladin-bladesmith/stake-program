#[tokio::test]
async fn fail_validator_stake_deactivate_stake_with_amount_greater_than_stake_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we try to deactivate an amount greater than the staked amount.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .amount(150)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::InsufficientStakeAmount);
}

#[tokio::test]
async fn validator_stake_deactivate_stake_with_zero_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we deactivate with zero amount.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .amount(0)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the deactivation should be cancelled.

    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.delegation.deactivating_amount, 0);
    assert!(stake_account
        .delegation
        .deactivation_timestamp
        .value()
        .is_none())
}

#[tokio::test]
async fn fail_validator_stake_deactivate_stake_with_maximum_deactivation_amount_exceeded() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we try to deactivate a greater amount than the maximum allowed.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .amount(100) // <- equivalent to 100% of the stake
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(
        err,
        PaladinStakeProgramError::MaximumDeactivationAmountExceeded
    );
}
