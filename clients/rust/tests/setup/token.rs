#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{
    instruction::Instruction, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    signature::Keypair, signer::Signer, system_instruction, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::{
    instruction::{initialize_account3, initialize_mint, mint_to as spl_mint_to},
    state::{Account as TokenAccount, Mint},
};

pub async fn create_associated_token_account(
    context: &mut ProgramTestContext,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Pubkey {
    let instructions = vec![
        spl_associated_token_account::instruction::create_associated_token_account(
            &context.payer.pubkey(),
            owner,
            mint,
            &spl_token::ID,
        ),
    ];

    context.get_new_latest_blockhash().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    get_associated_token_address_with_program_id(owner, mint, &spl_token::ID)
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let mut instructions = vec![system_instruction::create_account(
        &context.payer.pubkey(),
        &mint.pubkey(),
        rent.minimum_balance(Mint::LEN),
        Mint::LEN as u64,
        &spl_token::ID,
    )];

    instructions.push(
        initialize_mint(
            &spl_token::ID,
            &mint.pubkey(),
            mint_authority,
            freeze_authority,
            decimals,
        )
        .unwrap(),
    );

    context.get_new_latest_blockhash().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    owner: &Pubkey,
    token_account: &Keypair,
    mint: &Pubkey,
) -> Result<(), BanksClientError> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let mut instructions = vec![];

    instructions.push(system_instruction::create_account(
        &context.payer.pubkey(),
        &token_account.pubkey(),
        rent.minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN as u64,
        &spl_token::ID,
    ));

    instructions
        .push(initialize_account3(&spl_token::ID, &token_account.pubkey(), mint, owner).unwrap());

    context.get_new_latest_blockhash().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer, token_account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn mint_to_instruction(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    mint_authority: &Keypair,
    token: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    context.get_new_latest_blockhash().await.unwrap();

    spl_mint_to(
        &spl_token::ID,
        mint,
        token,
        &mint_authority.pubkey(),
        &[],
        amount,
    )
}
pub async fn mint_to(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    mint_authority: &Keypair,
    token: &Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    context.get_new_latest_blockhash().await.unwrap();

    let ix = mint_to_instruction(context, mint, mint_authority, token, amount)
        .await
        .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint_authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}
