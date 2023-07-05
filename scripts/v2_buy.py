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

with open(path) as f:
    V2_ROUTER_ABI = json.load(f)

V2_ROUTER = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"
TOKEN_TO_BUY = "0x6982508145454Ce325dDbE47a25d4ec3d2311933"

ACC: LocalAccount = Account.from_key(os.environ.get("ETH_SIGNER_KEY"))
print(ACC.address)

w3 = Web3(HTTPProvider(os.environ.get("ETH_RPC_URL")))
flashbot(w3, fbutil.FB_SIGNER)

V2_CONTRACT = w3.eth.contract(V2_ROUTER, abi=V2_ROUTER_ABI)

# read rlp-encoded from sys.stdin
rlp = sys.stdin.readline().strip()
# hex string to HexBytes
rlp = HexBytes(rlp)

price = int(w3.eth.gas_price * 1.5)
print(price/1e9)

# swap tx

WETH = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
tx = V2_CONTRACT.functions.swapExactETHForTokens(0, [WETH, w3.to_checksum_address(TOKEN_TO_BUY)], ACC.address, 10000000000).build_transaction({
    'from': ACC.address,
    'value': w3.to_wei(0.1, "ether"),
    'gas': 1000000,
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

print(fbutil.call_bundle(bundle, w3.eth.block_number, "https://relay.flashbots.net"))

block = w3.eth.block_number
fbutil.send_all(bundle, block + 1)
print("aiming for block", block + 1)