mod cmd;
mod filters;

use std::{sync::Arc, rc::Rc, default, collections::HashMap};

use clap::{Parser};
use cmd::watch::{App, Command::{WithBlock, Tx}};
use ethers::{providers::{Provider, Ws, Middleware, StreamExt}, types::{Transaction, Block, H256,TxHash, BlockId, BlockNumber, GethDebugTracingCallOptions, GethDebugTracingOptions, GethDebugBuiltInTracerType, GethDebugTracerType, CallConfig, GethDebugTracerConfig, GethDebugBuiltInTracerConfig, Address, Bytes, NameOrAddress}, utils::hex};
use ethers::abi::HumanReadableParser;
use ethers::types::GethTrace::{Known, Unknown};
use ethers::types::GethTraceFrame::CallTracer;

use crate::filters::Filters;

async fn get_flattened_trace(tx: Transaction, provider: Provider<Ws>) -> Option<HashMap<Address, Bytes>> {
    let mut opts = GethDebugTracingCallOptions::default();
    opts.tracing_options.tracer_config =
        Some(GethDebugTracerConfig::BuiltInTracer(GethDebugBuiltInTracerConfig::CallTracer(
            CallConfig { only_top_call: Some(false), with_log: Some(false) },
        )));
    opts.tracing_options.timeout = Some("1s".to_string());
    opts.tracing_options.tracer = Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::CallTracer));
    let block_id = BlockId::Number(BlockNumber::Latest);
    let traces = provider.debug_trace_call(&tx, Some(block_id), opts).await;
    if let Ok(traces) = traces {
        // Recursively flatten the CallFrame
        // mapping of To -> Bytes
        let mut flattened: HashMap<Address, Bytes> = HashMap::new();
        fn flatten(frame: &ethers::types::CallFrame, flattened: &mut HashMap<Address, Bytes>) {
            match &frame.to {
                Some(a) => {
                    if let NameOrAddress::Address(addr) = a {
                        flattened.insert(*addr, frame.input.clone());
                    }
                },
                None => {
                    // Contract creations are ignored
                },
            }
            if let Some(child_calls) = &frame.calls {
                for child in child_calls {
                    flatten(child, flattened);
                }
            }
        }
        if let Known(known_trace) = traces {
            if let CallTracer(t) = known_trace {
                flatten(&t, &mut flattened);
                return Some(flattened)
            }
        }
    }
    None
}

fn handle_block(block: Block<H256>) {
    println!("{:?}", block);
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Read .env file
    dotenv::dotenv().ok();
    let args = App::parse();
    // Connect to rpc url
    match args.command {
        WithBlock(args) => {
            //let mut filters = Filters:
            //let provider = Provider::<Ws>::connect(args.rpc.rpc_url).await.unwrap();
            //let block = provider.get_block(ethers::types::BlockNumber::Latest).await?;
            //match block {
                //None => println!("No block found"),
                //Some(block) => {
                    //filter.apply()
                //},
            //}
        }
        Tx(args) => {
            let mut filters = Filters::new();
            if let Some(from) = args.from {
                filters.add_filter(Box::new(filters::equality::FromFilter::new(from)));
            }
            if let Some(to) = args.to {
                filters.add_filter(Box::new(filters::equality::ToFilter::new(to)));
            }
            if let Some(value) = args.value {
                filters.add_filter(Box::new(filters::equality::ValueFilter::new(value)));
            }
            if let Some(nonce) = args.nonce {
                filters.add_filter(Box::new(filters::equality::NonceFilter::new(nonce)));
            }
            if let Some(tip) = args.tip {
                filters.add_filter(Box::new(filters::equality::TipFilter::new(tip)));
            }
            if let Some(gas_price) = args.gas_price {
                filters.add_filter(Box::new(filters::equality::GasPriceFilter::new(gas_price)));
                filters.add_filter(Box::new(filters::equality::MaxFeeFilter::new(gas_price)));
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
                filters.add_filter(Box::new(filters::range::ValueRangeFilter::new(args.value_gt, args.value_lt)));
            }
            if args.nonce_gt.is_some() || args.nonce_lt.is_some() {
                filters.add_filter(Box::new(filters::range::NonceRangeFilter::new(args.nonce_gt, args.nonce_lt)));
            }
            if args.tip_gt.is_some() || args.tip_lt.is_some() {
                filters.add_filter(Box::new(filters::range::TipRangeFilter::new(args.tip_gt, args.tip_lt)));
            }
            if args.gas_price_gt.is_some() || args.gas_price_lt.is_some() {
                filters.add_filter(Box::new(filters::range::GasPriceRangeFilter::new(args.gas_price_gt, args.gas_price_lt)));
                filters.add_filter(Box::new(filters::range::MaxFeeRangeFilter::new(args.gas_price_gt, args.gas_price_lt)));
            }

            let provider = Provider::<Ws>::connect(args.rpc.rpc_url).await.unwrap();
            let mut count: u64 = 0;
            if args.rpc.legacy {
                let mut stream = provider.subscribe_pending_txs().await?;
                'next: loop {
                    let txn_hash = stream.next().await;
                    let txn = provider.get_transaction(txn_hash.unwrap()).await?;
                    match txn {
                        // None => println!("No transaction found"),
                        None => continue,
                        Some(txn) => {
                            if !filters.apply(&txn) {
                                continue;
                            }
                            let touches = match args.touches {
                                Some(touches) => touches,
                                None => {
                                    // println!("{}", serde_json::to_string(&txn)?);
                                    println!("{}", hex::encode(txn.rlp()));
                                    if let Some(n) = args.n {
                                        count += 1;
                                        if count == n {
                                            break;
                                        }
                                    }
                                    continue;
                                },
                            };
                        
                            let flattened = match get_flattened_trace(txn.clone(), provider.clone()).await {
                                Some(flattened) => flattened,
                                None => {
                                    // println!("No trace found for {:?}", txn.hash);
                                    continue;
                                },
                            };
                        
                            for (addr, input) in &flattened {
                                if touches == *addr {
                                    let input_hex = hex::encode(input);
                                    let matched = args.touches_data.as_ref().map_or(true, |data| data.as_str() == input_hex)
                                        && args.touches_sig.as_ref().map_or(true, |sig| {
                                            let sig = HumanReadableParser::parse_function(&sig).unwrap().short_signature();
                                            input.starts_with(&sig)
                                        });
                        
                                    if matched {
                                        // println!("{}", serde_json::to_string(&txn)?);
                                        // print RLP encoded transaction
                                        println!("{}", hex::encode(txn.rlp()));
                                        if let Some(n) = args.n {
                                            count += 1;
                                            if count == n {
                                                break;
                                            }
                                        }
                                        continue 'next;
                                    }
                                }
                            }
                        },
                    }
                }
            } else {
                let mut stream = provider.subscribe_full_pending_txs().await?;
                'next: loop {
                    let txn = stream.next().await;
                    match txn {
                        None => continue,
                        Some(txn) => {
                            if !filters.apply(&txn) {
                                continue;
                            }
                            let touches = match args.touches {
                                Some(touches) => touches,
                                None => {
                                    // println!("{}", serde_json::to_string(&txn)?);
                                    println!("{}", hex::encode(txn.rlp()));
                                    if let Some(n) = args.n {
                                        count += 1;
                                        if count == n {
                                            break;
                                        }
                                    }
                                    continue;
                                },
                            };
                        
                            let flattened = match get_flattened_trace(txn.clone(), provider.clone()).await {
                                Some(flattened) => flattened,
                                None => {
                                    // println!("No trace found for {:?}", txn.hash);
                                    continue;
                                },
                            };
                        
                            for (addr, input) in &flattened {
                                if touches == *addr {
                                    let input_hex = hex::encode(input);
                                    let matched = args.touches_data.as_ref().map_or(true, |data| data.as_str() == input_hex)
                                        && args.touches_sig.as_ref().map_or(true, |sig| {
                                            let sig = HumanReadableParser::parse_function(&sig).unwrap().short_signature();
                                            input.starts_with(&sig)
                                        });
                        
                                    if matched {
                                        //println!("{}", serde_json::to_string(&txn)?);
                                        println!("{}", hex::encode(txn.rlp()));
                                        if let Some(n) = args.n {
                                            count += 1;
                                            if count == n {
                                                break;
                                            }
                                        }
                                        continue 'next;
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
    Ok(())
}