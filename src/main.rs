mod cmd;
mod filters;
mod trace;

use std::collections::{HashSet, VecDeque};
use std::time::Duration;

use clap::Parser;
use cmd::watch::{App, Command, OutputMode, TxArgs};
use ethers::{
    abi::HumanReadableParser,
    providers::{Middleware, Provider, ProviderError, Ws},
    types::{Transaction, H256},
    utils::hex,
};
use eyre::Result;
use futures::stream::{BoxStream, StreamExt};
use regex::Regex;

use crate::filters::{range, Filters};

const FETCH_CONCURRENCY: usize = 256;
const FETCH_RETRIES: usize = 4;
const SEEN_CAPACITY: usize = 16_384;

fn print_output(mode: OutputMode, tx: &Transaction) -> Result<()> {
    match mode {
        OutputMode::Rlp => println!("{}", hex::encode(tx.rlp())),
        OutputMode::Hash => println!("{:?}", tx.hash),
        OutputMode::Json => println!("{}", serde_json::to_string(tx)?),
    }
    Ok(())
}

fn build_filters(args: &TxArgs) -> Result<Filters> {
    let mut f = Filters::default();

    if let Some(v) = args.from {
        f.push(move |tx| tx.from == v);
    }
    if let Some(v) = args.to {
        f.push(move |tx| tx.to.map_or(v.is_zero(), |a| a == v));
    }
    if let Some(v) = args.value {
        f.push(move |tx| tx.value == v);
    }
    if let Some(v) = args.nonce {
        f.push(move |tx| tx.nonce == v);
    }
    if let Some(v) = args.tip {
        f.push(move |tx| tx.max_priority_fee_per_gas == Some(v));
    }
    if let Some(v) = args.gas_price {
        f.push(move |tx| tx.gas_price == Some(v));
    }

    range(&mut f, args.value_gt, args.value_lt, |tx| Some(tx.value));
    range(&mut f, args.nonce_gt, args.nonce_lt, |tx| Some(tx.nonce));
    range(&mut f, args.tip_gt, args.tip_lt, |tx| {
        tx.max_priority_fee_per_gas
    });
    range(&mut f, args.gas_price_gt, args.gas_price_lt, |tx| tx.gas_price);

    if let Some(sig) = &args.sig {
        let sel = HumanReadableParser::parse_function(sig)?.short_signature();
        f.push(move |tx| tx.input.starts_with(&sel));
    }
    if let Some(sel) = args.selector {
        f.push(move |tx| tx.input.starts_with(&sel));
    }
    if let Some(data) = &args.data {
        let bytes = hex::decode(data)?;
        f.push(move |tx| tx.input.as_ref() == bytes.as_slice());
    }
    if let Some(pattern) = &args.data_re {
        let re = Regex::new(pattern)?;
        f.push(move |tx| re.is_match(&hex::encode(&tx.input)));
    }
    Ok(f)
}

async fn retry_get_tx(
    provider: &Provider<Ws>,
    hash: H256,
) -> Result<Option<Transaction>, ProviderError> {
    let mut r = provider.get_transaction(hash).await;
    for _ in 0..FETCH_RETRIES {
        if r.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        r = provider.get_transaction(hash).await;
    }
    r
}

struct Seen {
    set: HashSet<H256>,
    order: VecDeque<H256>,
    cap: usize,
}

impl Seen {
    fn new(cap: usize) -> Self {
        Self {
            set: HashSet::with_capacity(cap),
            order: VecDeque::with_capacity(cap),
            cap,
        }
    }

    fn insert(&mut self, h: H256) {
        if self.set.insert(h) {
            self.order.push_back(h);
            if self.set.len() > self.cap {
                if let Some(old) = self.order.pop_front() {
                    self.set.remove(&old);
                }
            }
        }
    }

    fn contains(&self, h: &H256) -> bool {
        self.set.contains(h)
    }
}

struct Runtime {
    args: TxArgs,
    filters: Filters,
    trace_sig: Option<[u8; 4]>,
    provider: Provider<Ws>,
    count: u64,
}

impl Runtime {
    async fn process(&mut self, tx: &Transaction) -> Result<bool> {
        if !self.filters.apply(tx) {
            return Ok(false);
        }

        if let Some(touches) = self.args.trace_addr {
            let Some(flattened) =
                trace::get_flattened_trace(tx.clone(), self.provider.clone()).await
            else {
                return Ok(false);
            };
            let Some(input) = flattened.get(&touches) else {
                return Ok(false);
            };
            if let Some(expected) = self.args.trace_data.as_deref() {
                if expected != hex::encode(input) {
                    return Ok(false);
                }
            }
            if let Some(sel) = &self.trace_sig {
                if !input.starts_with(sel) {
                    return Ok(false);
                }
            }
        }

        print_output(self.args.output, tx)?;
        if let Some(n) = self.args.n {
            self.count += 1;
            if self.count >= n {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

async fn pending_stream<'a>(
    provider: &'a Provider<Ws>,
    legacy: bool,
) -> Result<BoxStream<'a, Transaction>> {
    if legacy {
        let p = provider.clone();
        let stream = provider
            .subscribe_pending_txs()
            .await?
            .map(move |hash| {
                let p = p.clone();
                async move { retry_get_tx(&p, hash).await }
            })
            .buffer_unordered(FETCH_CONCURRENCY)
            .filter_map(|r| async move {
                match r {
                    Ok(tx) => tx,
                    Err(e) => {
                        eprintln!("get_transaction failed, skipping: {e}");
                        None
                    }
                }
            })
            .boxed();
        Ok(stream)
    } else {
        Ok(provider.subscribe_full_pending_txs().await?.boxed())
    }
}

async fn run_tx(args: TxArgs) -> Result<()> {
    let filters = build_filters(&args)?;
    let trace_sig = args
        .trace_sig
        .as_deref()
        .map(|s| HumanReadableParser::parse_function(s).map(|f| f.short_signature()))
        .transpose()?;
    let provider = Provider::<Ws>::connect(&args.rpc.rpc_url).await?;

    let pending = pending_stream(&provider, args.rpc.legacy).await?;
    let mut blocks = provider.subscribe_blocks().await?;
    let mut seen = Seen::new(SEEN_CAPACITY);
    let mut rt = Runtime {
        args,
        filters,
        trace_sig,
        provider: provider.clone(),
        count: 0,
    };

    tokio::pin!(pending);
    loop {
        tokio::select! {
            Some(tx) = pending.next() => {
                seen.insert(tx.hash);
                if rt.process(&tx).await? { return Ok(()); }
            }
            Some(block) = blocks.next() => {
                if !rt.args.confirmed { continue; }
                let Some(num) = block.number else { continue };
                let Some(block) = rt.provider.get_block_with_txs(num).await? else { continue };
                for tx in block.transactions {
                    if seen.contains(&tx.hash) { continue; }
                    if rt.process(&tx).await? { return Ok(()); }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    match App::parse().command {
        Command::Tx(args) => run_tx(args).await,
    }
}
