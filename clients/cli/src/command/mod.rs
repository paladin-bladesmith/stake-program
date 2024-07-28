pub mod initialize_config;

use clap::{Parser, Subcommand};
use solana_clap_v3_utils::input_parsers::{
    parse_url_or_moniker, signer::SignerSourceParserBuilder,
};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(about, author, version)]
#[rustfmt::skip]
pub struct Arguments {
    /// Configuration file to use [default: system configuration file].
    #[clap(
        short,
        long = "config",
        global = true,
        value_name = "PATH",
    )]
    pub config_file: Option<String>,

    /// Filepath or URL to a keypair [default: client keypair].
    #[clap(
        long,
        global = true,
        value_name = "PATH",
        value_parser = SignerSourceParserBuilder::default().allow_all().build()
    )]
    pub keypair: Option<PathBuf>,

    /// Show additional information.
    #[clap(
        short,
        long,
        global = true
    )]
    pub verbose: bool,

    /// JSON RPC URL for the cluster [default: value from configuration file].
    #[clap(
        short,
        long = "url",
        global = true,
        value_name = "URL",
        value_parser = parse_url_or_moniker
    )]
    pub json_rpc_url: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
#[rustfmt::skip]
pub enum Commands {
    /// Creates a Stake config account which controls staking parameters for a specified mint.
    InitializeConfig {
        /// Stake token mint address.
        #[clap(
            value_name = "MINT",
        )]
        mint: String,
                
        /// Filepath or URL to a keypair representing the account to be created
        /// [default: random keypair].
        #[clap(
            long,
            value_name = "PATH",
        )]
        account: Option<PathBuf>,

        /// Slash authority [default: client keypair address].
        #[clap(
            long,
            value_name = "SLASH AUTHORITY",
        )]
        slash_authority: Option<String>,

        /// Config authority [default: client keypair address].
        #[clap(
            long,
            value_name = "CONFIG AUTHORITY",
        )]
        config_authority: Option<String>,

        /// Cooldown time for deactivation in hours.
        #[clap(
            long = "cooldown",
            value_name = "COOLDOWN",
            default_value = "720",
        )]
        cooldown_time_seconds: u64,

        /// Maximum deactivation amount as a percentage of stake.
        #[clap(
            long = "percentage",
            value_name = "DEACTIVATION",
            default_value = "5",
        )]
        max_deactivation_basis_points: u16,
    },
}
