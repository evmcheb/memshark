from eth_account.signers.local import LocalAccount
from web3 import Web3, HTTPProvider
from flashbots import flashbot
from eth_account.account import Account
from hexbytes import HexBytes
import os
from eth_account import Account, messages

import sys
import json
import dotenv
import fbutil

import requests

dotenv.load_dotenv()

ACC: LocalAccount = Account.from_key(os.environ.get("ETH_SIGNER_KEY"))
print(ACC.address)

w3 = Web3(HTTPProvider(os.environ.get("ETH_RPC_URL")))
flashbot(w3, fbutil.FB_SIGNER)

# read rlp-encoded from sys.stdin
rlp = sys.stdin.readline().strip()
# hex string to HexBytes
rlp = HexBytes(rlp)

price = int(w3.eth.gas_price * 1.5)
print(price/1e9)

tx = {
    'from': ACC.address,
    'to': ACC.address,
    'value': 0,
    'gas': 21000,
    "maxFeePerGas": price,
    "maxPriorityFeePerGas": price,
    'nonce': w3.eth.get_transaction_count(ACC.address),
    'data': b'',
    'chainId': 1
}

# sign the transaction
tx_signed = ACC.sign_transaction(tx)

bundle = [
    rlp.hex(),
    tx_signed.rawTransaction.hex()
]
print(bundle)

block = w3.eth.block_number
fbutil.send_all(bundle, block + 1)
print("aiming for block", block + 1)