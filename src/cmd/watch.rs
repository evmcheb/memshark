use std::str::FromStr;

use clap::{Args, Parser, Subcommand, ValueEnum};

use ethers::{
    types::{Address, U256},
    utils::hex,
};

use eyre::Result;

fn parse_u256(s: &str) -> Result<U256> {
    Ok(if s.starts_with("0x") {
        U256::from_str(s)?
    } else {
        U256::from_dec_str(s)?
    })
}

fn parse_gwei(s: &str) -> Result<U256> {
    let f = s.parse::<f64>()?;
    Ok(U256::from((f * 1e9) as u64))
}

pub fn strip_0x_prefix(s: &str) -> Result<String, &'static str> {
    Ok(s.strip_prefix("0x").unwrap_or(s).to_string())
}

fn parse_selector(s: &str) -> Result<[u8; 4]> {
    let bytes = hex::decode(s.strip_prefix("0x").unwrap_or(s))?;
    <[u8; 4]>::try_from(bytes.as_slice())
        .map_err(|_| eyre::eyre!("selector must be exactly 4 bytes (8 hex chars)"))
}

#[derive(Debug, Parser)]
#[clap(name = "shark", version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum OutputMode {
    Rlp,
    Hash,
    Json,
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
    pub confirmed: bool,

    #[clap(long, short)]
    pub to: Option<Address>,
    #[clap(long, short)]
    pub from: Option<Address>,

    #[clap(long, short)]
    pub sig: Option<String>,
    #[clap(long, value_parser=parse_selector, conflicts_with_all=&["sig"])]
    pub selector: Option<[u8; 4]>,
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
    pub trace_addr: Option<Address>,
    #[clap(long, value_parser=strip_0x_prefix, conflicts_with_all=&["trace_data"])]
    pub trace_sig: Option<String>,
    #[clap(long, value_parser=strip_0x_prefix)]
    pub trace_data: Option<String>,

    #[clap(flatten)]
    pub rpc: RpcOpts,

    #[clap(long, value_enum, default_value_t=OutputMode::Rlp)]
    pub output: OutputMode,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Tx(TxArgs),
}
