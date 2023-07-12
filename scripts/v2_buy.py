import click
from eth_account.signers.local import LocalAccount
from web3 import Web3, HTTPProvider
from flashbots import flashbot
from eth_account.account import Account
from hexbytes import HexBytes
import os
from eth_account import Account
import sys
import json
import dotenv
import fbutil
import requests
import os.path

my_path = os.path.abspath(os.path.dirname(__file__))
path = os.path.join(my_path, "./abis/v2router.abi")

dotenv.load_dotenv()

w3 = Web3(HTTPProvider(os.environ.get("ETH_RPC_URL")))
flashbot(w3, fbutil.FB_SIGNER)

with open(path) as f:
    V2_ROUTER_ABI = json.load(f)

EST_GAS = 150_000
WETH = Web3.to_checksum_address("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
V2_ROUTER = Web3.to_checksum_address("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D")

V2_CONTRACT = w3.eth.contract(V2_ROUTER, abi=V2_ROUTER_ABI)

ACC: LocalAccount = Account.from_key(os.environ.get("ETH_SIGNER_KEY"))

# print a summary of the account
@click.command()
@click.option('--token', help='Token contract address', required=True)
@click.option('--amount', help='Amount of ETH to buy with', required=True)
@click.option('--bribe', help='Amount of ETH to use in the gas bribe', required=True)
@click.option('--simulate', help='Simulate the bundle', is_flag=True)
def run(token, amount, bribe, simulate):
    token = Web3.to_checksum_address(token)
    bribe = Web3.to_wei(bribe, "ether")
    amount = Web3.to_wei(amount, "ether")
    price = int(bribe / EST_GAS)

    print(f"Waiting for stdin to snipe {token} for {amount/1e18:.3f} ETH with {bribe/1e18:.3f} ETH bribe")
    print(f"Account: {ACC.address}")
    print(f"Gas price: {price/1e9:.3f}")

    # read rlp-encoded from sys.stdin
    rlp = sys.stdin.readline().strip()
    # hex string to HexBytes
    rlp = HexBytes(rlp)

    #price = int(w3.eth.gas_price * 1.5)

    tx = V2_CONTRACT.functions.swapExactETHForTokens(0, [WETH, token], ACC.address, 10000000000).build_transaction({
        'from': ACC.address,
        'value': amount,
        'gas': int(500000),
        "maxFeePerGas": price,
        "maxPriorityFeePerGas": price,
        'nonce': w3.eth.get_transaction_count(ACC.address),
        'chainId': 1
    })
    print(tx)

    # sign the transaction
    tx_signed = ACC.sign_transaction(tx)

    bundle = [
        rlp.hex(),
        tx_signed.rawTransaction.hex()
    ]
    print(bundle)

    if simulate:
        print(fbutil.call_bundle(bundle, w3.eth.block_number, "https://relay.flashbots.net"))

    block = w3.eth.block_number
    fbutil.send_all(bundle, block + 1)
    print("aiming for block", block + 1)

if __name__ == '__main__':
    run()