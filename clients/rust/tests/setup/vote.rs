#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use solana_program_test::ProgramTestContext;
use solana_sdk::{account::Account, clock::Epoch, pubkey::Pubkey, vote::state::VoteState};

pub fn create_vote_account(
    context: &mut ProgramTestContext,
    node: &Pubkey,
    authority: &Pubkey,
) -> Pubkey {
    create_vote_account_with_program_id(context, node, authority, &solana_sdk::vote::program::ID)
}

pub fn create_vote_account_with_program_id(
    context: &mut ProgramTestContext,
    node: &Pubkey,
    authority: &Pubkey,
    program_id: &Pubkey,
) -> Pubkey {
    let vote = Pubkey::new_unique();
    let mut data = vec![0u8; VoteState::size_of()];
    // 0-4 is the version offset
    data[4..36].copy_from_slice(node.as_ref()); // node_pubkey
    data[36..68].copy_from_slice(authority.as_ref()); // authorized_withdrawer

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
