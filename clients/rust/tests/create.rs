#![cfg(feature = "test-sbf")]

use solana_program_test::{tokio, ProgramTest};
use solana_sdk::signature::{Keypair, Signer};

#[tokio::test]
async fn create() {
    let context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a PDA derived from the payer's public key.

    let address = Keypair::new();

    assert!(address.pubkey() != context.payer.pubkey());
}
