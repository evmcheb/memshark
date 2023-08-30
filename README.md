# shark-rs

`shark-rs` is a highly performant Ethereum Virtual Machine (EVM) mempool filtering tool written in Rust. It enables precise monitoring and analysis of transactions based on various filters.

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
``````

## Install from source

```
git clone https://github.com/evmcheb/shark
cd shark
cargo install --path .
```

### Run

Configure your filters and execute the application:

bash

```bash
shark tx/block [filters]
```

### Filters

You can apply various filters as command-line options, such as:

*   `--from` - Filter by sender address.
*   `--to` - Filter by recipient address.
*   `--value` - Filter by Ether value.
*   `--nonce` - Filter by nonce value.
*   `--tip` - Filter by tip value.
*   `--legacy` - For nodes that do not support full pending transaction bodies. 
*   ... and more.

Consult the internal documentation and code to explore all available filters and their specific usage.

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

Acknowledgments
---------------

Special thanks to the Ethers-rs library and the Rust community for their support.

---

Happy filtering with `shark-rs`!