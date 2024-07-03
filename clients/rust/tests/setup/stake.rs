use paladin_stake::{accounts::Stake, instructions::InitializeStakeBuilder, pdas::find_stake_pda};
use solana_program_test::ProgramTestContext;
use solana_sdk::{pubkey::Pubkey, signer::Signer, system_instruction, transaction::Transaction};

pub async fn create_stake(
    context: &mut ProgramTestContext,
    validator: &Pubkey,
    vote: &Pubkey,
    config: &Pubkey,
) -> Pubkey {
    let (stake_pda, _) = find_stake_pda(validator, config);

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
