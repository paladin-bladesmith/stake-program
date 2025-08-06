use paladin_rewards_program_client::{
    accounts::{HolderRewards, HolderRewardsPool},
    instructions::{InitializeHolderRewardsBuilder, InitializeHolderRewardsPoolBuilder},
};
use paladin_stake_program::state::find_duna_document_pda;
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

use crate::setup::{
    config::{create_ata, get_duna_hash},
    sign_duna_document,
};

use super::token::create_associated_token_account;

pub struct RewardsManager {
    /// The rewards pool.
    pub pool: Pubkey,
    /// The holders rewards.
    pub pool_token_account: Pubkey,
    // A owner / staker / user
    pub owner: Keypair,
    pub owner_token_account: Pubkey,
    pub owner_holder_rewards: Pubkey,
}

impl RewardsManager {
    pub async fn new(context: &mut ProgramTestContext, mint: &Pubkey, vault_pda: &Pubkey) -> Self {
        let (pool, pool_token_account) = create_holder_rewards_pool(context, mint, vault_pda).await;

        // Setup a user
        let owner = Keypair::new();
        let owner_token_account =
            create_associated_token_account(context, &owner.pubkey(), mint).await;
        sign_duna_document(context, &owner.pubkey());
        let owner_holder_rewards =
            create_holder_rewards(context, &pool, mint, owner.insecure_clone()).await;

        Self {
            pool,
            pool_token_account,
            owner,
            owner_token_account,
            owner_holder_rewards,
        }
    }
}

pub async fn create_holder_rewards_pool(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    vault_pda: &Pubkey,
) -> (Pubkey, Pubkey) {
    // Fund the rewards pool and extra account metas.
    let rent = context.banks_client.get_rent().await.unwrap();

    // rewards pool
    let (holder_rewards_pool, _) = HolderRewardsPool::find_pda(mint);
    let holder_rewards_pool_token_account =
        get_associated_token_address(&holder_rewards_pool, mint);
    create_ata(context, &holder_rewards_pool, &mint)
        .await
        .unwrap();
    let vault_holder_rewards = HolderRewards::find_pda(vault_pda).0;

    // Initialize the holder rewards pool.
    let instructions = vec![
        system_instruction::transfer(
            &context.payer.pubkey(),
            &holder_rewards_pool,
            rent.minimum_balance(HolderRewardsPool::LEN),
        ),
        system_instruction::transfer(
            &context.payer.pubkey(),
            &vault_holder_rewards,
            rent.minimum_balance(HolderRewards::LEN),
        ),
        InitializeHolderRewardsPoolBuilder::new()
            .holder_rewards_pool(holder_rewards_pool)
            .holder_rewards_pool_token_account(holder_rewards_pool_token_account)
            .mint(*mint)
            .duna_document_hash(get_duna_hash())
            .stake_vault_pda(*vault_pda)
            .vault_holder_rewards(vault_holder_rewards)
            .instruction(),
    ];

    context.get_new_latest_blockhash().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    (holder_rewards_pool, holder_rewards_pool_token_account)
}

pub async fn create_holder_rewards(
    context: &mut ProgramTestContext,
    pool: &Pubkey,
    mint: &Pubkey,
    owner: Keypair,
) -> Pubkey {
    let rent = context.banks_client.get_rent().await.unwrap();
    let (holder_rewards, _) = HolderRewards::find_pda(&owner.pubkey());
    let holder_rewards_pool_token_account = get_associated_token_address(&pool, &mint);
    let (duna_pda, _) = find_duna_document_pda(&owner.pubkey(), &get_duna_hash());

    let instructions = vec![
        system_instruction::transfer(
            &context.payer.pubkey(),
            &holder_rewards,
            rent.minimum_balance(HolderRewards::LEN),
        ),
        InitializeHolderRewardsBuilder::new()
            .holder_rewards_pool(*pool)
            .holder_rewards_pool_token_account(holder_rewards_pool_token_account)
            .holder_rewards(holder_rewards)
            .mint(*mint)
            .owner(owner.pubkey())
            .duna_document_pda(duna_pda)
            .instruction(),
    ];

    context.get_new_latest_blockhash().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    holder_rewards
}
