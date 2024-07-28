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
    // TODO: Add subcommands here
}
