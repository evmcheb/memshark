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

import requests

RELAYS = [
    "https://builder0x69.io",
    "https://rpc.beaverbuild.org",
    "https://rsync-builder.xyz",
    "https://relay.flashbots.net",
    "https://api.blocknative.com/v1/auction" ,
    "https://builder.gmbit.co/rpc",
    "https://eth-builder.com",
    "https://rpc.titanbuilder.xyz",
    "https://buildai.net",
    "https://rpc.payload.de",
    #"https://mev.api.blxrbdn.com",
    "https://rpc.lightspeedbuilder.info",
    "https://rpc.nfactorial.xyz",
    "https://boba-builder.com/searcher",
    "https://rpc.f1b.io"
]

dotenv.load_dotenv()

"""
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "eth_callBundle",
  "params": [
    {
      txs,               // Array[String], A list of signed transactions to execute in an atomic bundle
      blockNumber,       // String, a hex encoded block number for which this bundle is valid on
      stateBlockNumber,  // String, either a hex encoded number or a block tag for which state to base this simulation on. Can use "latest"
      timestamp,         // (Optional) Number, the timestamp to use for this bundle simulation, in seconds since the unix epoch
    }
  ]
}
"""
def call_bundle(txns, block, relay):
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_callBundle",
        "params": [
            {
                "txs": txns,
                "blockNumber": str(hex(block)),
                "stateBlockNumber": str(hex(block)),
            }
        ]
    }
    body = json.dumps(req, separators=(',', ':'))
    print(body)
    message = messages.encode_defunct(text=Web3.keccak(text=body).hex())
    print(message)
    signature = FB_SIGNER.address+":"+FB_SIGNER.sign_message(message).signature.hex()
    print(signature)
    header = {
        "X-Flashbots-Signature": signature,
        "Content-Type": "application/json"
    }
    r = requests.post(relay, data=body, headers=header)
    return r.content

def send_bundle(txns, block, relay):
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_sendBundle",
        "params": [
            {
                "txs": txns,
                "blockNumber": str(hex(block)),
            }
        ]
    }
    body = json.dumps(req, separators=(',', ':'))
    message = messages.encode_defunct(text=Web3.keccak(text=body).hex())
    signature = FB_SIGNER.address+":"+FB_SIGNER.sign_message(message).signature.hex()
    header = {
        "X-Flashbots-Signature": signature,
        "Content-Type": "application/json"
    }
    r = requests.post(relay, data=body, headers=header)
    print(relay, r.content)
    return r.content
    

FB_SIGNER = Account.from_key(os.environ.get("FLASHBOTS_SIGNER"))
ACC: LocalAccount = Account.from_key(os.environ.get("ETH_SIGNER_KEY"))
print(ACC.address)

w3 = Web3(HTTPProvider(os.environ.get("ETH_RPC_URL")))
flashbot(w3, FB_SIGNER)

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
    tx_signed.rawTransaction.hex(),
    #rlp.hex()
]
print(bundle)

block = w3.eth.block_number
#try:
    #call_bundle(bundle, block)
    #print("simulation success")
#except Exception as e:
    #print("simulation failed")
    #print(e)
# use concurrent.futures to send to multiple relays
from concurrent.futures import ThreadPoolExecutor
with ThreadPoolExecutor(max_workers=20) as executor:
    for relay in RELAYS:
        executor.submit(send_bundle, bundle, block + 1, relay)
        print(f"{relay} sent")
print("aiming for block", block + 1)