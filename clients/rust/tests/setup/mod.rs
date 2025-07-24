#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use paladin_rewards_program_client::accounts::HolderRewards;
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{account::Account, pubkey::Pubkey};

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
    vault_autority: &Pubkey,
) -> Pubkey {
    // Create the holder rewards account for the vault.
    let rent = context.banks_client.get_rent().await.unwrap();
    let vault_holder_rewards = HolderRewards::find_pda(vault_autority).0;
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&HolderRewards {
                last_accumulated_rewards_per_token: 0,
                deposited: 0,
                padding: 0,
            })
            .unwrap(),
            owner: paladin_rewards_program_client::ID,
            ..Default::default()
        }
        .into(),
    );

    vault_holder_rewards
}
