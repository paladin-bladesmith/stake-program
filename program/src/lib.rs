#[cfg(feature = "bpf-entrypoint")]
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("GQurxHCYQCNfYR37nHNb6ZiLWg3jpbh2fWv2RpzwGqRK");
