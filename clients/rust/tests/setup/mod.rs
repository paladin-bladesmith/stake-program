#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program::state::find_duna_document_pda;
use solana_program::pubkey;
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::Instruction,
    pubkey::Pubkey,
    vote::state::VoteState,
};
use spl_transfer_hook_interface::{
    get_extra_account_metas_address, offchain::add_extra_account_metas_for_execute,
};

use crate::setup::config::get_duna_hash;

pub mod config;
pub mod harvest;
pub mod rewards;
pub mod sol_staker_stake;
pub mod stake;
pub mod token;
pub mod validator_stake;
pub mod vote;

/// Scaling factor for rewards per token (1e18).
pub const REWARDS_PER_TOKEN_SCALING_FACTOR: u128 = 1_000_000_000_000_000_000;

pub const SWAD: u64 = 10u64.pow(9);

pub const DUNA_PROGRAM_ID: Pubkey = pubkey!("8TwDM3rkxQuFCiS2iPB1HB3Q3qnN7b6J4SCTDCpw9SS1");

pub async fn setup(program_overrides: &[(&'static str, Pubkey)]) -> ProgramTestContext {
    let mut program_test = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    );
    program_test.add_program(
        "paladin_rewards_program",
        paladin_rewards_program_client::ID,
        None,
    );
    program_test.add_program(
        "paladin_sol_stake_view_program",
        paladin_sol_stake_view_program_client::ID,
        None,
    );

    for (name, program_id) in program_overrides {
        program_test.add_program(name, *program_id, None);
    }

    let mut context = program_test.start_with_context().await;
    context
        .warp_to_slot(context.genesis_config().epoch_schedule.first_normal_slot + 1)
        .unwrap();

    context
}

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

pub fn calculate_stake_rewards_per_token(rewards: u64, stake_amount: u64) -> u128 {
    if stake_amount == 0 {
        0
    } else {
        // Calculation: rewards / stake_amount
        //
        // Scaled by 1e18 to store 18 decimal places of precision.
        (rewards as u128)
            .checked_mul(REWARDS_PER_TOKEN_SCALING_FACTOR)
            .and_then(|product| product.checked_div(stake_amount as u128))
            .unwrap()
    }
}

pub fn pack_to_vec<T>(val: T) -> Vec<u8>
where
    T: solana_sdk::program_pack::Pack,
{
    let mut buf = vec![0; T::LEN];
    T::pack(val, &mut buf).unwrap();

    buf
}

pub async fn setup_holder_rewards(
    context: &mut ProgramTestContext,
    token_account: &Pubkey,
) -> Pubkey {
    // Create the holder rewards account for the vault.
    let rent = context.banks_client.get_rent().await.unwrap();
    let vault_holder_rewards = HolderRewards::find_pda(token_account).0;
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                unharvested_rewards: 0,
                rent_sponsor: Pubkey::default(),
                rent_debt: 0,
                minimum_balance: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    vault_holder_rewards
}

pub async fn get_duna_pda_from_vote(context: &mut ProgramTestContext, vote: Pubkey) -> Pubkey {
    let account = get_account!(context, vote);
    let vote_state =
        VoteState::deserialize(&account.data).expect("Failed to deserialize vote state");
    let (duna_doc, _) = find_duna_document_pda(&vote_state.authorized_withdrawer, &get_duna_hash());

    duna_doc
}

pub fn sign_duna_document(context: &mut ProgramTestContext, acc: &Pubkey) -> Pubkey {
    sign_duna_document_with_data(context, acc, vec![1; 1])
}

pub async fn sign_duna_document_with_vote(
    context: &mut ProgramTestContext,
    vote: Pubkey,
) -> Pubkey {
    let account = get_account!(context, vote);
    let withdrawer = VoteState::deserialize(&account.data)
        .expect("Failed to deserialize vote state")
        .authorized_withdrawer;

    sign_duna_document_with_data(context, &withdrawer, vec![1; 1])
}

pub fn sign_duna_document_with_data(
    context: &mut ProgramTestContext,
    acc: &Pubkey,
    data: Vec<u8>,
) -> Pubkey {
    let doc_hash = get_duna_hash();
    let (duna_acc, _) = find_duna_document_pda(acc, &doc_hash);

    context.set_account(
        &duna_acc,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data,
            owner: DUNA_PROGRAM_ID,
            ..Default::default()
        }),
    );

    duna_acc
}
