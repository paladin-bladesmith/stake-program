#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    clock::Epoch,
    pubkey::Pubkey,
    vote::state::{VoteState, VoteStateVersions},
};

pub async fn create_vote_account(
    context: &mut ProgramTestContext,
    node: &Pubkey,
    authority: &Pubkey,
) -> Pubkey {
    create_vote_account_with_program_id(context, node, authority, &solana_sdk::vote::program::ID)
        .await
}

pub async fn create_vote_account_with_program_id(
    context: &mut ProgramTestContext,
    node: &Pubkey,
    authority: &Pubkey,
    program_id: &Pubkey,
) -> Pubkey {
    let vote = Pubkey::new_unique();
    let root_slot = context.banks_client.get_root_slot().await.unwrap();

    let mut vote_state = VoteState::new_rand_for_tests(*node, root_slot);
    vote_state.authorized_withdrawer = *authority;

    let mut data = vec![0; VoteState::size_of()];
    VoteState::serialize(&VoteStateVersions::new_current(vote_state), &mut data).unwrap();

    let vote_account = Account {
        lamports: 5_000_000_000, // 5 SOL
        data,
        owner: *program_id,
        executable: false,
        rent_epoch: Epoch::default(),
    };

    context.set_account(&vote, &vote_account.into());
    vote
}

pub fn add_vote_account(
    context: &mut ProgramTestContext,
    node: &Pubkey,
    authority: &Pubkey,
) -> Pubkey {
    let vote = Pubkey::new_unique();
    let mut vote_state = VoteState::new_rand_for_tests(*node, 0);
    vote_state.authorized_withdrawer = *authority;

    let mut data = vec![0; VoteState::size_of()];
    VoteState::serialize(&VoteStateVersions::new_current(vote_state), &mut data).unwrap();
    let vote_account = Account {
        lamports: 5_000_000_000, // 5 SOL
        data,
        owner: solana_sdk::vote::program::ID,
        executable: false,
        rent_epoch: Epoch::default(),
    };
    context.set_account(&vote, &vote_account.into());

    vote
}
