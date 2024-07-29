use std::{path::PathBuf, str::FromStr};

use paladin_stake_program_client::{
    accounts::Config, instructions::InitializeConfigBuilder, pdas::find_vault_pda,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    message::Message,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};

pub struct InitializeConfigArgs {
    /// Stake token mint address.
    pub mint: String,

    /// Stake config account address.
    pub account: Option<PathBuf>,

    /// Stake slashing authority address.
    pub slash_authority: Option<String>,

    /// Stake config authority address.
    pub config_authority: Option<String>,

    /// Stake deactivation cooldown time in hours.
    pub cooldown_hours: u64,

    /// Maximum deactivation amount as a percentage of stake.
    pub max_deactivation_basis_points: u16,
}

pub async fn process_initialize_config(
    rpc_client: &RpcClient,
    signer: &dyn Signer,
    args: InitializeConfigArgs,
) -> Result<Signature, Box<dyn std::error::Error>> {
    // Account input.

    let account = if let Some(account) = args.account {
        read_keypair_file(account)?
    } else {
        Keypair::new()
    };

    let slash_authority = if let Some(authority) = args.slash_authority {
        Pubkey::from_str(&authority).unwrap()
    } else {
        signer.pubkey()
    };

    let config_authority = if let Some(authority) = args.config_authority {
        Pubkey::from_str(&authority).unwrap()
    } else {
        signer.pubkey()
    };

    let mint = Pubkey::from_str(&args.mint).unwrap();

    let (vault_token_account, _) = find_vault_pda(&account.pubkey());

    // Instruction creation.

    let create_ix = system_instruction::create_account(
        &signer.pubkey(),
        &account.pubkey(),
        rpc_client
            .get_minimum_balance_for_rent_exemption(Config::LEN)
            .await?,
        Config::LEN as u64,
        &paladin_stake_program_client::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(account.pubkey())
        .config_authority(config_authority)
        .slash_authority(slash_authority)
        .mint(mint)
        .vault(vault_token_account)
        .cooldown_time_seconds(args.cooldown_hours.checked_mul(3600).unwrap())
        .max_deactivation_basis_points(args.max_deactivation_basis_points)
        .instruction();

    // Send transaction.

    let mut transaction = Transaction::new_unsigned(Message::new(
        &[create_ix, initialize_ix],
        Some(&signer.pubkey()),
    ));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {err}"))?;

    transaction
        .try_sign(&vec![signer, &account], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {err}"))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {err}"))?;

    Ok(signature)
}
