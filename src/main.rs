mod cmd;
mod filters;
mod trace;

use clap::Parser;
use cmd::watch::{
    App,
    Command::{Tx, WithBlock},
    OutputMode,
};
use ethers::{
    abi::HumanReadableParser,
    providers::{Middleware, Provider, StreamExt, Ws},
    types::Transaction,
    utils::hex,
};

use crate::filters::{add_optional_filter, add_range_filter, Filters};
use anyhow::Result;

fn print_output(output_mode: OutputMode, txn: &Transaction) -> Result<()> {
    match output_mode {
        OutputMode::Rlp => {
            println!("{}", hex::encode(txn.rlp()));
        }
        OutputMode::Hash => {
            println!("{:?}", txn.hash);
        }
        OutputMode::Json => {
            println!("{}", serde_json::to_string(&txn)?);
        }
    }
    Ok(())
}

async fn process_transaction(
    txn: Transaction,
    args: &cmd::watch::TxArgs,
    filters: &mut Filters,
    provider: &Provider<Ws>,
    count: &mut u64,
) -> Result<()> {
    if !filters.apply(&txn) {
        return Ok(());
    }
    let output_and_exit =
        |count: &mut u64, args: &cmd::watch::TxArgs, txn: &Transaction| -> Result<()> {
            print_output(args.output, txn)?;
            if let Some(n) = args.n {
                *count += 1;
                if *count >= n {
                    // Exit silently
                    std::process::exit(0);
                }
            }
            Ok(())
        };

    match args.trace_addr {
        None => output_and_exit(count, args, &txn)?,
        Some(touches) => {
            if let Some(flattened) = trace::get_flattened_trace(txn.clone(), provider.clone()).await
            {
                for (addr, input) in &flattened {
                    if touches == *addr {
                        let input_hex = hex::encode(input);
                        let matches_data = match args.trace_data.as_deref() {
                            None => true,
                            Some(data) => data == &input_hex,
                        };

                        let matches_sig = match &args.trace_sig {
                            None => true,
                            Some(sig) => {
                                let parsed_sig = HumanReadableParser::parse_function(sig)
                                    .expect("Invalid function signature")
                                    .short_signature();
                                input.starts_with(&parsed_sig)
                            }
                        };

                        if matches_data && matches_sig {
                            output_and_exit(count, args, &txn)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read .env file
    dotenv::dotenv().ok();
    let args = App::parse();
    // Connect to rpc url
    match args.command {
        WithBlock(_args) => {
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
            add_optional_filter(&mut filters, args.from, |v| {
                Box::new(filters::equality::FromFilter::new(v))
            });
            add_optional_filter(&mut filters, args.to, |v| {
                Box::new(filters::equality::ToFilter::new(v))
            });
            add_optional_filter(&mut filters, args.value, |v| {
                Box::new(filters::equality::ValueFilter::new(v))
            });
            add_optional_filter(&mut filters, args.nonce, |v| {
                Box::new(filters::equality::NonceFilter::new(v))
            });
            add_optional_filter(&mut filters, args.tip, |v| {
                Box::new(filters::equality::TipFilter::new(v))
            });
            add_optional_filter(&mut filters, args.gas_price, |v| {
                Box::new(filters::equality::GasPriceFilter::new(v))
            });

            add_range_filter(&mut filters, args.value_gt, args.value_lt, |gt, lt| {
                Box::new(filters::range::ValueRangeFilter::new(gt, lt))
            });
            add_range_filter(&mut filters, args.nonce_gt, args.nonce_lt, |gt, lt| {
                Box::new(filters::range::NonceRangeFilter::new(gt, lt))
            });
            add_range_filter(&mut filters, args.tip_gt, args.tip_lt, |gt, lt| {
                Box::new(filters::range::TipRangeFilter::new(gt, lt))
            });
            add_range_filter(
                &mut filters,
                args.gas_price_gt,
                args.gas_price_lt,
                |gt, lt| Box::new(filters::range::GasPriceRangeFilter::new(gt, lt)),
            );

            if let Some(sig) = &args.sig {
                let sig = HumanReadableParser::parse_function(sig)?.short_signature();
                filters.add_filter(Box::new(filters::calldata::SigFilter::new(sig)));
            }
            if let Some(data) = &args.data {
                let data = hex::decode(data)?;
                filters.add_filter(Box::new(filters::calldata::DataFilter::new(data)));
            }
            if let Some(re) = &args.data_re {
                filters.add_filter(Box::new(filters::calldata::RegexFilter::new(re)));
            }

            let provider = Provider::<Ws>::connect(args.rpc.rpc_url.clone())
                .await
                .unwrap();
            let mut count: u64 = 0;
            if args.rpc.legacy {
                let mut stream = provider.subscribe_pending_txs().await?;
                loop {
                    let txn_hash = stream.next().await;
                    let txn = provider.get_transaction(txn_hash.unwrap()).await?;
                    if let Some(txn) = txn {
                        process_transaction(txn, &args, &mut filters, &provider, &mut count)
                            .await?;
                    }
                }
            } else {
                let mut stream = provider.subscribe_full_pending_txs().await?;
                loop {
                    let txn = stream.next().await;
                    if let Some(txn) = txn {
                        process_transaction(txn, &args, &mut filters, &provider, &mut count)
                            .await?;
                    }
                }
            }
        }
    }
    Ok(())
}
