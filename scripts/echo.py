# read from stdin, read json 
# write to stdout
from eth_account.signers.local import LocalAccount
from web3 import Web3, HTTPProvider
from flashbots import flashbot
from eth_account.account import Account
import os

import sys
import json
import dotenv

dotenv.load_dotenv()

ETH_ACCOUNT_SIGNATURE: LocalAccount = Account.from_key(os.environ.get("ETH_SIGNER_KEY"))

w3 = Web3(HTTPProvider(os.environ.get("ETH_RPC_URL")))
flashbot(w3, ETH_ACCOUNT_SIGNATURE)

# read line from sys.stdin
hash = sys.stdin.readline()

price = int(w3.eth.gas_price * 1.5)

tx = {
    'from': ETH_ACCOUNT_SIGNATURE.address,
    'to': ETH_ACCOUNT_SIGNATURE.address,
    'value': 0,
    'gas': 21000,
    "maxFeePerGas": price,
    "maxPriorityFeePerGas": price,
    'nonce': w3.eth.get_transaction_count(ETH_ACCOUNT_SIGNATURE.address),
    'data': b'',
    'chainId': 1
}

# sign the transaction
tx_signed = ETH_ACCOUNT_SIGNATURE.sign_transaction(tx)
print(tx_signed.rawTransaction.hex())

bundle = [
    {"signedTransaction": tx_signed.rawTransaction},
]