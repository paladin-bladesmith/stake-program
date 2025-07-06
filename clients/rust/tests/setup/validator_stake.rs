use paladin_stake_program_client::{
    accounts::ValidatorStake, instructions::InitializeValidatorStakeBuilder,
    pdas::find_validator_stake_pda,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction, vote::state::VoteState,
};

use crate::{
    get_account,
    setup::{get_duna_pda_from_vote, sign_duna_document},
};

use super::vote::create_vote_account;

pub struct ValidatorStakeManager {
    // Stake account.
    pub stake: Pubkey,
    // Stake authority.
    pub authority: Keypair,
    // Validator account.
    pub validator: Pubkey,
    // Duna document PDA.
    pub duna_doc: Pubkey,
    // Validator vote account.
    pub vote: Pubkey,
}

impl ValidatorStakeManager {
    pub async fn new(context: &mut ProgramTestContext, config: &Pubkey) -> Self {
        let authority = Keypair::new();
        let validator = Pubkey::new_unique();

        // Creates the validator vote account.
        let vote = create_vote_account(context, &validator, &authority.pubkey()).await;

        // Sign duna doc PDA
        let duna_doc = sign_duna_document(context, &authority.pubkey());

        // And a stake account.
        let stake = create_validator_stake(context, &vote, config).await;

        Self {
            stake,
            authority,
            validator,
            duna_doc,
            vote,
        }
    }

    pub async fn new_with_vote(
        context: &mut ProgramTestContext,
        config: &Pubkey,
        vote: Pubkey,
    ) -> Self {
        let authority = Keypair::new();
        let validator = Pubkey::new_unique();

        // Sign duna doc PDA
        let account = get_account!(context, vote);
        let vote_state =
            VoteState::deserialize(&account.data).expect("Failed to deserialize vote state");

        let duna_doc = sign_duna_document(context, &vote_state.authorized_withdrawer);

        // And a stake account.
        let stake = create_validator_stake(context, &vote, config).await;

        Self {
            stake,
            authority,
            validator,
            duna_doc,
            vote,
        }
    }
}

pub async fn create_validator_stake(
    context: &mut ProgramTestContext,
    vote: &Pubkey,
    config: &Pubkey,
) -> Pubkey {
    let (stake_pda, _) = find_validator_stake_pda(vote, config);

    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(ValidatorStake::LEN),
    );

    let initialize_ix = InitializeValidatorStakeBuilder::new()
        .config(*config)
        .validator_stake(stake_pda)
        .validator_vote(*vote)
        .duna_document_pda(get_duna_pda_from_vote(context, *vote).await)
        .instruction();

    context.get_new_latest_blockhash().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction_with_metadata(tx)
        .await
        .unwrap();

    stake_pda
}
