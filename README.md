# memshark-rs

`memshark-rs` is a highly performant Ethereum Virtual Machine (EVM) mempool filtering tool written in Rust. It enables precise monitoring and analysis of transactions based on various filters.

## Features

- Supports filtering transactions by (one or several) attributes such as from address, to address, value, nonce, tip, gas price, signature, and data.
- Offers multiple output modes (RLP, Hash, JSON).
- Supports several types of filters (regex, range, exact).
- Supports filtering by sub-calls using `debug_traceCall`

## Install from bins (Linux and macOS)

curl one-liner:

```
curl -fsSL https://github.com/evmcheb/shark/raw/master/install.sh | bash
```

wget one-liner:

```
wget -O - https://github.com/evmcheb/shark/raw/master/install.sh | bash
```

## Install from source

```
git clone https://github.com/evmcheb/shark
cd shark
cargo install --path .
```

### Run

The CLI tool is called `shark`:

```bash
shark tx/block [filters]
```

### Options
- `--n` - Number of transactions to output
- `--rpc-url` - Websocket RPC URL (default: ETH_RPC_WSS)
- `--legacy` - Use the legacy subscription method (default: false)
- `--confirmed` - Include confirmed transactions that were missed in the mempool (default: false)

### Output Options
- `--output rlp` - Output the RLP-encoded transaction
- `--output hash` - Output only the transaction hash
- `--output json` - Output the full transaction in JSON format

### Equality Filters
- `--from <ADDRESS>` - Match transactions from a specific address
- `--to <ADDRESS>` - Match transactions to a specific address
- `--value <AMOUNT>` - Match transactions with exact value in wei
- `--nonce <NUMBER>` - Match transactions with specific nonce
- `--tip <AMOUNT>` - Match transactions with specific priority fee (tip) in wei
- `--gas-price <AMOUNT>` - Match transactions with specific gas price in wei

### Range Filters
- `--value-gt <AMOUNT>` - Match transactions with value greater than specified amount (wei)
- `--value-lt <AMOUNT>` - Match transactions with value less than specified amount (wei)
- `--nonce-gt <NUMBER>` - Match transactions with nonce greater than specified number
- `--nonce-lt <NUMBER>` - Match transactions with nonce less than specified number
- `--tip-gt <AMOUNT>` - Match transactions with priority fee greater than specified amount (wei)
- `--tip-lt <AMOUNT>` - Match transactions with priority fee less than specified amount (wei)
- `--gas-price-gt <AMOUNT>` - Match transactions with gas price greater than specified amount (wei)
- `--gas-price-lt <AMOUNT>` - Match transactions with gas price less than specified amount (wei)

### Calldata Filters
- `--sig <SIGNATURE>` - Match transactions with specific function signature (e.g., "transfer(address,uint256)")
- `--data <HEX>` - Match transactions with exact calldata (hex encoded)
- `--data-re <REGEX>` - Match transactions where calldata matches the given regex pattern

### Trace Filters
> **Note**: Trace filters will match against any subcall in the transaction trace, not just the top-level call. This means you can match transactions that interact with a contract indirectly through other contracts.

- `--trace-addr <ADDRESS>` - Match transactions that touch the specified address during execution
- `--trace-data <HEX>` - When used with --trace-addr, match only if the call to the address uses this exact calldata
- `--trace-sig <SIGNATURE>` - When used with --trace-addr, match only if the call to the address uses this function signature

### Examples
```
shark tx --n 1 --touches 0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640 --touches-sig "function swap(address,bool,int256,uint160,bytes calldata)"
```
- Find the next transaction that is sent to/makes a call to `0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640` with `swap` signature.


Contributing
------------

Contributions are welcome! Please refer to the contribution guidelines for details on how to participate in this project.

Support and Issues
------------------

https://twitter.com/evmcheb

Special thanks to the Ethers-rs library and the Rust community for their support.

---

Happy filtering!