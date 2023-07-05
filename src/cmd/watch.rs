use std::str::FromStr;

use clap::{Args, Parser, Subcommand};

use ethers::types::{U256, Address, U64};

use eyre::Result;

/// parse a hex str or decimal str as U256
fn parse_u64(s: &str) -> Result<U64> {
    Ok(if s.starts_with("0x") { U64::from_str(s)? } else { U64::from_dec_str(s)? })
}
fn parse_u256(s: &str) -> Result<U256> {
    Ok(if s.starts_with("0x") { U256::from_str(s)? } else { U256::from_dec_str(s)? })
}
fn parse_gwei(s: &str) -> Result<U256> {
    // string to float
    let f = s.parse::<f64>()?;
    // float to gwei
    let g = f * 1e9;
    // convert to u256
    Ok(U256::from(g as u64))
}

pub fn strip_0x_prefix(s: &str) -> Result<String, &'static str> {
    Ok(s.strip_prefix("0x").unwrap_or(s).to_string())
}

/// Here's my app!
#[derive(Debug, Parser)]
#[clap(name = "shark", version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args)]
pub struct RpcOpts {
    #[clap(long = "rpc-url", short = 'r', env = "ETH_RPC_WSS")]
    pub rpc_url: String,

    #[clap(long, short)]
    pub legacy: bool,
}

#[derive(Debug, Args)]
pub struct TxArgs {
    #[clap(long, short)]
    pub n: Option<u64>,

    #[clap(long)]
    pub with_missed: bool,

    #[clap(long, short)]
    pub to: Option<Address>,
    #[clap(long, short)]
    pub from: Option<Address>,

    #[clap(long, short)]
    pub sig: Option<String>,
    #[clap(long, short, value_parser=strip_0x_prefix, conflicts_with_all=&["data_re"])]
    pub data: Option<String>,
    #[clap(long)]
    pub data_re: Option<String>,

    #[clap(long, value_parser=parse_u256, conflicts_with_all=&["value_gt", "value_lt"])]
    pub value: Option<U256>,
    #[clap(long, value_parser=parse_u256)]
    pub value_gt: Option<U256>,
    #[clap(long, value_parser=parse_u256)]
    pub value_lt: Option<U256>,

    #[clap(long, value_parser=parse_u256, conflicts_with_all=&["nonce_gt", "nonce_lt"])]
    pub nonce: Option<U256>,
    #[clap(long, value_parser=parse_u256)]
    pub nonce_gt: Option<U256>,
    #[clap(long, value_parser=parse_u256)]
    pub nonce_lt: Option<U256>,

    #[clap(long, value_parser=parse_gwei, conflicts_with_all=&["tip_gt", "tip_lt"])]
    pub tip: Option<U256>,
    #[clap(long, value_parser=parse_gwei)]
    pub tip_gt: Option<U256>,
    #[clap(long, value_parser=parse_gwei)]
    pub tip_lt: Option<U256>,

    #[clap(long, value_parser=parse_gwei, conflicts_with_all=&["gas_price_gt", "gas_price_lt"])]
    pub gas_price: Option<U256>,
    #[clap(long, value_parser=parse_gwei)]
    pub gas_price_gt: Option<U256>,
    #[clap(long, value_parser=parse_gwei)]
    pub gas_price_lt: Option<U256>,

    #[clap(long)]
    pub touches: Option<Address>,
    #[clap(long, value_parser=strip_0x_prefix, conflicts_with_all=&["touches_data"])]
    pub touches_sig: Option<String>,
    #[clap(long, value_parser=strip_0x_prefix)]
    pub touches_data: Option<String>,

    #[clap(flatten)]
    pub rpc: RpcOpts,
}

#[derive(Debug, Args)]
pub struct BlockArgs {
    #[clap(long, short, value_parser=parse_u64)]
    pub number: Option<U64>,

    #[clap(flatten)]
    pub rpc: RpcOpts,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Help message for read.
    #[clap(name = "block")]
    WithBlock(BlockArgs),
    Tx(TxArgs),
}