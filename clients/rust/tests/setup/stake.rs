use solana_program_test::ProgramTestContext;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake::{
        instruction::{create_account, deactivate_stake, delegate_stake},
        state::{Authorized, Lockup, StakeStateV2},
    },
    transaction::Transaction,
};

pub async fn create_stake_account(
    context: &mut ProgramTestContext,
    stake: &Keypair,
    authorized: &Authorized,
    lockup: &Lockup,
    stake_amount: u64,
) -> u64 {
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(std::mem::size_of::<StakeStateV2>()) + stake_amount;

    context.get_new_latest_blockhash().await.unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &create_account(
            &context.payer.pubkey(),
            &stake.pubkey(),
            authorized,
            lockup,
            lamports,
        ),
        Some(&context.payer.pubkey()),
        &[&context.payer, stake],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction_with_metadata(transaction)
        .await
        .unwrap();

    lamports
}

pub async fn delegate_stake_account(
    context: &mut ProgramTestContext,
    stake_address: &Pubkey,
    vote: &Pubkey,
    authorized: &Keypair,
) {
    let transaction = Transaction::new_signed_with_payer(
        &[delegate_stake(stake_address, &authorized.pubkey(), vote)],
        Some(&context.payer.pubkey()),
        &[&context.payer, authorized],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
    warp_to_next_epoch(context).await;
}

pub async fn deactivate_stake_account(
    context: &mut ProgramTestContext,
    stake_address: &Pubkey,
    authority: &Keypair,
) {
    let transaction = Transaction::new_signed_with_payer(
        &[deactivate_stake(stake_address, &authority.pubkey())],
        Some(&context.payer.pubkey()),
        &[&context.payer, authority],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
    warp_to_next_epoch(context).await;
}

async fn warp_to_next_epoch(context: &mut ProgramTestContext) {
    let root = context.banks_client.get_root_slot().await.unwrap();
    let slots_per_epoch = context.genesis_config().epoch_schedule.slots_per_epoch;
    println!("WARPING: {root} -> {}", root + slots_per_epoch);
    context.warp_to_slot(root + slots_per_epoch).unwrap();
}
