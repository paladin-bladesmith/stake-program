use paladin_stake_program_client::{
    accounts::Stake, instructions::InitializeStakeBuilder, pdas::find_stake_pda,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};

use super::vote::create_vote_account;

pub struct StakeManager {
    // Stake account.
    pub stake: Pubkey,
    // Stake authority.
    pub authority: Keypair,
    // Validator account.
    pub validator: Pubkey,
    // Validator vote account.
    pub vote: Pubkey,
}

impl StakeManager {
    pub async fn new(context: &mut ProgramTestContext, config: &Pubkey) -> Self {
        let mut manager = Self {
            stake: Pubkey::default(),
            authority: Keypair::new(),
            validator: Pubkey::new_unique(),
            vote: Pubkey::default(),
        };

        // Creates the validator vote account.

        manager.vote =
            create_vote_account(context, &manager.validator, &manager.authority.pubkey()).await;

        // And a stake account.

        manager.stake = create_stake(context, &manager.vote, config).await;

        let transfer_ix = system_instruction::transfer(
            &context.payer.pubkey(),
            &manager.stake,
            context
                .banks_client
                .get_rent()
                .await
                .unwrap()
                .minimum_balance(Stake::LEN),
        );

        let initialize_ix = InitializeStakeBuilder::new()
            .config(*config)
            .stake(manager.stake)
            .validator_vote(manager.vote)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[transfer_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        manager
    }
}

pub async fn create_stake(
    context: &mut ProgramTestContext,
    vote: &Pubkey,
    config: &Pubkey,
) -> Pubkey {
    let (stake_pda, _) = find_stake_pda(vote, config);

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
        .config(*config)
        .stake(stake_pda)
        .validator_vote(*vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    stake_pda
}
