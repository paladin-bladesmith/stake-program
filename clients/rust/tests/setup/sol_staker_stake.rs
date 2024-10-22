use paladin_stake_program_client::{
    accounts::SolStakerStake, instructions::InitializeSolStakerStakeBuilder,
    pdas::find_sol_staker_stake_pda,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake::state::{Authorized, Lockup},
    system_instruction,
    transaction::Transaction,
};

use super::stake::{create_stake_account, delegate_stake_account};

pub struct SolStakerStakeManager {
    // Stake account.
    pub stake: Pubkey,
    // Stake authority.
    pub authority: Keypair,
    // SOL stake account.
    pub sol_stake: Pubkey,
}

impl SolStakerStakeManager {
    pub async fn new(
        context: &mut ProgramTestContext,
        config: &Pubkey,
        validator_stake: &Pubkey,
        validator_vote: &Pubkey,
        amount: u64,
    ) -> Self {
        // create the SOL stake and delegation

        let stake_state = Keypair::new();
        let authority = Keypair::new();

        create_stake_account(
            context,
            &stake_state,
            &Authorized::auto(&authority.pubkey()),
            &Lockup::default(),
            amount,
        )
        .await;

        if amount > 0 {
            delegate_stake_account(context, &stake_state.pubkey(), validator_vote, &authority)
                .await;
        }

        // create the sol staker stake account

        let stake =
            create_sol_staker_stake(context, &stake_state.pubkey(), validator_stake, config).await;

        Self {
            stake,
            authority,
            sol_stake: stake_state.pubkey(),
        }
    }
}

pub async fn create_sol_staker_stake(
    context: &mut ProgramTestContext,
    sol_stake: &Pubkey,
    validator_stake: &Pubkey,
    config: &Pubkey,
) -> Pubkey {
    let (stake_pda, _) = find_sol_staker_stake_pda(sol_stake, config);

    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(SolStakerStake::LEN),
    );

    let initialize_ix = InitializeSolStakerStakeBuilder::new()
        .config(*config)
        .sol_staker_stake(stake_pda)
        .validator_stake(*validator_stake)
        .sol_staker_native_stake(*sol_stake)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
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
