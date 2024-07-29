mod command;
use command::*;

use clap::{IntoApp, Parser};
use initialize_config::{process_initialize_config, InitializeConfigArgs};
use solana_clap_v3_utils::{
    input_parsers::signer::SignerSource, input_validators::normalize_to_url_if_moniker,
    keypair::signer_from_path,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_remote_wallet::remote_wallet::RemoteWalletManager;
use solana_sdk::{commitment_config::CommitmentConfig, signer::Signer};
use std::{process::exit, rc::Rc};

struct Config {
    commitment_config: CommitmentConfig,
    default_signer: Box<dyn Signer>,
    json_rpc_url: String,
    verbose: bool,
    websocket_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let mut wallet_manager: Option<Rc<RemoteWalletManager>> = None;

    let config = {
        let cli_config =
            solana_cli_config::Config::load(if let Some(config_file) = &args.config_file {
                config_file
            } else if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                config_file
            } else {
                eprintln!("missing configuration file");
                exit(1);
            })
            .unwrap_or_default();

        if let Some((_, matches)) = Arguments::command().get_matches().subcommand() {
            let default_signer = if let Ok(Some((signer, _))) =
                SignerSource::try_get_signer(matches, "keypair", &mut wallet_manager)
            {
                Box::new(signer)
            } else {
                signer_from_path(
                    matches,
                    &cli_config.keypair_path,
                    "keypair",
                    &mut wallet_manager,
                )?
            };

            let json_rpc_url =
                normalize_to_url_if_moniker(args.json_rpc_url.unwrap_or(cli_config.json_rpc_url));
            let websocket_url = solana_cli_config::Config::compute_websocket_url(&json_rpc_url);

            Config {
                commitment_config: CommitmentConfig::confirmed(),
                default_signer,
                json_rpc_url,
                verbose: args.verbose,
                websocket_url,
            }
        } else {
            panic!("No subcommand provided");
        }
    };
    solana_logger::setup_with_default("solana=info");

    if config.verbose {
        println!("JSON RPC URL: {}", config.json_rpc_url);
        println!("Websocket URL: {}", config.websocket_url);
    }
    let rpc_client =
        RpcClient::new_with_commitment(config.json_rpc_url.clone(), config.commitment_config);

    match args.command {
        Commands::InitializeConfig {
            account,
            slash_authority,
            config_authority,
            mint,
            cooldown_hours,
            max_deactivation_basis_points,
        } => {
            let signature = process_initialize_config(
                &rpc_client,
                config.default_signer.as_ref(),
                InitializeConfigArgs {
                    account,
                    slash_authority,
                    config_authority,
                    mint,
                    cooldown_hours,
                    max_deactivation_basis_points,
                },
            )
            .await
            .unwrap_or_else(|err| {
                eprintln!("error: {err}");
                exit(1);
            });
            println!("Signature: {signature}");
        }
    }

    Ok(())
}
