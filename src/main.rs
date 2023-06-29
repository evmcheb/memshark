mod cmd;
mod filters;

use clap::{Parser};
use cmd::watch::{App, Command::{WithBlock, Tx}};
use ethers::{providers::{Provider, Ws, Middleware, StreamExt}, types::{Transaction, Block, H256,TxHash}, utils::hex};
use ethers::abi::HumanReadableParser;

use crate::filters::Filters;

fn handle_tx(tx: Transaction) {
    println!("{:?}", tx);
}

fn handle_block(block: Block<H256>) {
    println!("{:?}", block);
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = App::parse();
    println!("Hello, world! {:?}", args);
    // Connect to rpc url
    match args.command {
        WithBlock(args) => {
            //let mut filters = Filters:
            //let provider = Provider::<Ws>::connect(args.rpc.rpc_url).await.unwrap();
            //let block = provider.get_block(ethers::types::BlockNumber::Latest).await?;
            //match block {
                //None => println!("No block found"),
                //Some(block) => {
                    //filters.apply()
                //},
            //}
        }
        Tx(args) => {
            let mut filters = Filters::new();
            if let Some(from) = args.from {
                filters.add_filter(Box::new(filters::FromFilter::new(from)));
            }
            if let Some(to) = args.to {
                filters.add_filter(Box::new(filters::ToFilter::new(to)));
            }
            if let Some(value) = args.value {
                filters.add_filter(Box::new(filters::ValueFilter::new(value)));
            }
            if let Some(nonce) = args.nonce {
                filters.add_filter(Box::new(filters::NonceFilter::new(nonce)));
            }
            if let Some(tip) = args.tip {
                filters.add_filter(Box::new(filters::TipFilter::new(tip)));
            }
            if let Some(gas_price) = args.gas_price {
                filters.add_filter(Box::new(filters::GasPriceFilter::new(gas_price)));
                filters.add_filter(Box::new(filters::MaxFeeFilter::new(gas_price)));
            }
            if let Some(sig) = args.sig {
                let sig = HumanReadableParser::parse_function(&sig)?.short_signature();
                filters.add_filter(Box::new(filters::calldata::SigFilter::new(sig)));
            }
            if let Some(data) = args.data {
                let data = hex::decode(data)?;
                filters.add_filter(Box::new(filters::calldata::DataFilter::new(data)));
            }
            if let Some(re) = args.data_re {
                filters.add_filter(Box::new(filters::calldata::RegexFilter::new(re)));
            }

            if args.value_gt.is_some() || args.value_lt.is_some() {
                filters.add_filter(Box::new(filters::ValueRangeFilter::new(args.value_gt, args.value_lt)));
            }
            if args.nonce_gt.is_some() || args.nonce_lt.is_some() {
                filters.add_filter(Box::new(filters::NonceRangeFilter::new(args.nonce_gt, args.nonce_lt)));
            }
            if args.tip_gt.is_some() || args.tip_lt.is_some() {
                filters.add_filter(Box::new(filters::TipRangeFilter::new(args.tip_gt, args.tip_lt)));
            }
            if args.gas_price_gt.is_some() || args.gas_price_lt.is_some() {
                filters.add_filter(Box::new(filters::GasPriceRangeFilter::new(args.gas_price_gt, args.gas_price_lt)));
                filters.add_filter(Box::new(filters::MaxFeeRangeFilter::new(args.gas_price_gt, args.gas_price_lt)));
            }

            let provider = Provider::<Ws>::connect(args.rpc.rpc_url).await.unwrap();
            if args.rpc.legacy {
                let mut stream = provider.subscribe_pending_txs().await?;
                loop {
                    let txn_hash = stream.next().await;
                    let txn = provider.get_transaction(txn_hash.unwrap()).await?;
                    match txn {
                        None => println!("No transaction found"),
                        Some(txn) => {
                            if filters.apply(&txn) {
                                println!("{}", serde_json::to_string(&txn)?);
                            }
                        },
                    }
                }
            } else {
                let mut stream = provider.subscribe_full_pending_txs().await?;
                loop {
                    let txn = stream.next().await;
                    match txn {
                        None => println!("No transaction found"),
                        Some(txn) => {
                            if filters.apply(&txn) {
                                println!("{}", serde_json::to_string(&txn)?);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}