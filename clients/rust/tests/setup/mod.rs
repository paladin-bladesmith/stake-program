#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use solana_program_test::ProgramTestContext;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_transfer_hook_interface::{
    get_extra_account_metas_address, offchain::add_extra_account_metas_for_execute,
};

pub mod config;
pub mod rewards;
pub mod token;
pub mod validator_stake;
pub mod vote;

/// Scaling factor for rewards per token (1e9).
pub const REWARDS_PER_TOKEN_SCALING_FACTOR: u128 = 1_000_000_000;

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

#[allow(clippy::too_many_arguments)]
pub async fn add_extra_account_metas_for_transfer(
    context: &mut ProgramTestContext,
    instruction: &mut Instruction,
    program_id: &Pubkey,
    source_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    amount: u64,
) {
    let extra_metas_address = get_extra_account_metas_address(mint_pubkey, program_id);
    let extra_metas_account = get_account!(context, extra_metas_address);

    add_extra_account_metas_for_execute(
        instruction,
        program_id,
        source_pubkey,
        mint_pubkey,
        destination_pubkey,
        authority_pubkey,
        amount,
        |key| {
            let data = if key.eq(&extra_metas_address) {
                Some(extra_metas_account.data.clone())
            } else {
                None
            };
            async move { Ok(data) }
        },
    )
    .await
    .unwrap();
}
