use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};
use spl_token_2022::{
    extension::{transfer_hook, ExtensionType},
    state::{Account, Mint},
};

/// Default extension for a mint account.
pub const MINT_EXTENSIONS: &[ExtensionType] = &[ExtensionType::TransferHook];

/// Default extension for a token account.
pub const TOKEN_ACCOUNT_EXTENSIONS: &[ExtensionType] =
    &[ExtensionType::TransferHook, ExtensionType::ImmutableOwner];

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    extensions: &[ExtensionType],
) -> Result<(), BanksClientError> {
    let account_size = ExtensionType::try_calculate_account_len::<Mint>(extensions).unwrap();
    let rent = context.banks_client.get_rent().await.unwrap();

    let mut instructions = vec![system_instruction::create_account(
        &context.payer.pubkey(),
        &mint.pubkey(),
        rent.minimum_balance(account_size),
        account_size as u64,
        &spl_token_2022::ID,
    )];

    if extensions.contains(&ExtensionType::TransferHook) {
        instructions.push(
            transfer_hook::instruction::initialize(
                &spl_token_2022::ID,
                &mint.pubkey(),
                Some(*mint_authority),
                // TODO: change program id to Rewards program
                Some(paladin_stake::ID),
            )
            .unwrap(),
        );
    }

    instructions.push(
        spl_token_2022::instruction::initialize_mint(
            &spl_token_2022::ID,
            &mint.pubkey(),
            mint_authority,
            freeze_authority,
            decimals,
        )
        .unwrap(),
    );

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer, mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_token(
    context: &mut ProgramTestContext,
    owner: &Pubkey,
    token_account: &Keypair,
    mint: &Pubkey,
    extensions: &[ExtensionType],
) -> Result<(), BanksClientError> {
    let length = ExtensionType::try_calculate_account_len::<Account>(extensions).unwrap();
    let rent = context.banks_client.get_rent().await.unwrap();

    let mut instructions = vec![];

    instructions.push(system_instruction::create_account(
        &context.payer.pubkey(),
        &token_account.pubkey(),
        rent.minimum_balance(length),
        length as u64,
        &spl_token_2022::ID,
    ));

    if extensions.contains(&ExtensionType::ImmutableOwner) {
        instructions.push(
            spl_token_2022::instruction::initialize_immutable_owner(
                &spl_token_2022::ID,
                &token_account.pubkey(),
            )
            .unwrap(),
        );
    }

    instructions.push(
        spl_token_2022::instruction::initialize_account3(
            &spl_token_2022::ID,
            &token_account.pubkey(),
            mint,
            owner,
        )
        .unwrap(),
    );

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &[&context.payer, token_account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

#[allow(dead_code)]
pub async fn mint_to(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    mint_authority: &Keypair,
    token: &Pubkey,
    amount: u64,
    decimals: u8,
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[spl_token_2022::instruction::mint_to_checked(
            &spl_token_2022::ID,
            &mint.pubkey(),
            token,
            &mint_authority.pubkey(),
            &[],
            amount,
            decimals,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer, mint_authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}
