#![allow(unused)]

pub mod token;
pub use token::*;

use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{pubkey::Pubkey, signer::Signer, system_instruction, transaction::Transaction};

#[macro_export]
macro_rules! assert_instruction_error {
    ( $error:expr, $matcher:pat ) => {
        match $error {
            solana_program_test::BanksClientError::TransactionError(
                solana_sdk::transaction::TransactionError::InstructionError(_, $matcher),
            ) => {
                assert!(true)
            }
            err => assert!(false, "Expected instruction error but got '{:#?}'", err),
        };
    };
}

#[macro_export]
macro_rules! assert_custom_error {
    ( $error:expr, $matcher:pat ) => {
        match $error {
            solana_program_test::BanksClientError::TransactionError(
                solana_sdk::transaction::TransactionError::InstructionError(
                    _,
                    solana_sdk::instruction::InstructionError::Custom(x),
                ),
            ) => match num_traits::FromPrimitive::from_i32(x as i32) {
                Some($matcher) => assert!(true),
                Some(other) => {
                    assert!(
                        false,
                        "Expected another custom instruction error than '{:#?}'",
                        other
                    )
                }
                None => assert!(false, "Expected custom instruction error"),
            },
            err => assert!(
                false,
                "Expected custom instruction error but got '{:#?}'",
                err
            ),
        };
    };
}

#[macro_export]
macro_rules! get_account {
    ( $context:expr, $pubkey:expr ) => {{
        let account = $context
            .banks_client
            .get_account($pubkey)
            .await
            .expect(&format!("account not found: {}", $pubkey));

        assert!(account.is_some());

        account.unwrap()
    }};
}

pub async fn airdrop(
    context: &mut ProgramTestContext,
    receiver: &Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
    Ok(())
}
