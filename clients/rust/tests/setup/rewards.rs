use paladin_rewards_program_client::{
    accounts::{HolderRewards, HolderRewardsPool},
    instructions::{InitializeHolderRewardsBuilder, InitializeHolderRewardsPoolBuilder},
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};

use super::token::create_associated_token_account;

pub struct RewardsManager {
    /// The rewards pool.
    pub pool: Pubkey,
    /// The holders rewards.
    pub holder_rewards: Pubkey,
    /// Owner.
    pub owner: Keypair,
    /// Token account.
    pub token_account: Pubkey,
}

impl RewardsManager {
    pub async fn new(
        context: &mut ProgramTestContext,
        mint: &Pubkey,
        mint_authority: &Keypair,
    ) -> Self {
        let pool = create_holder_rewards_pool(context, mint, mint_authority).await;
        let owner = Keypair::new();
        let token_account = create_associated_token_account(context, &owner.pubkey(), mint).await;
        let holder_rewards = create_holder_rewards(context, &pool, mint, &token_account).await;

        Self {
            pool,
            holder_rewards,
            owner,
            token_account,
        }
    }
}

pub async fn create_holder_rewards_pool(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    mint_authority: &Keypair,
) -> Pubkey {
    // Fund the rewards pool and extra account metas.

    let rent = context.banks_client.get_rent().await.unwrap();

    // rewards pool
    let (holder_rewards_pool, _) = HolderRewardsPool::find_pda(mint);

    // Initialize the holder rewards pool.

    let instructions = vec![
        system_instruction::transfer(
            &context.payer.pubkey(),
            &holder_rewards_pool,
            rent.minimum_balance(HolderRewardsPool::LEN),
        ),
        InitializeHolderRewardsPoolBuilder::new()
            .holder_rewards_pool(holder_rewards_pool)
            .mint(*mint)
            .instruction(),
    ];

    context.get_new_latest_blockhash().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer, mint_authority],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_metadata(tx)
        .await
        .unwrap();

    holder_rewards_pool
}

pub async fn create_holder_rewards(
    context: &mut ProgramTestContext,
    pool: &Pubkey,
    mint: &Pubkey,
    token_account: &Pubkey,
) -> Pubkey {
    let rent = context.banks_client.get_rent().await.unwrap();
    let (holder_rewards, _) = HolderRewards::find_pda(token_account);

    let instructions = vec![
        system_instruction::transfer(
            &context.payer.pubkey(),
            &holder_rewards,
            rent.minimum_balance(HolderRewards::LEN),
        ),
        InitializeHolderRewardsBuilder::new()
            .holder_rewards_pool(*pool)
            .holder_rewards(holder_rewards)
            .token_account(*token_account)
            .mint(*mint)
            .instruction(),
    ];

    context.get_new_latest_blockhash().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_metadata(tx)
        .await
        .unwrap();

    holder_rewards
}
