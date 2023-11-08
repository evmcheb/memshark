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
dotenv.load_dotenv()

FB_SIGNER = Account.from_key(os.environ.get("FLASHBOTS_SIGNER"))

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
    "https://rpc.f1b.io",
    "https://rpc.lokibuilder.xyz"
]


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
    message = messages.encode_defunct(text=Web3.keccak(text=body).hex())
    signature = FB_SIGNER.address+":"+FB_SIGNER.sign_message(message).signature.hex()
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

from concurrent.futures import ThreadPoolExecutor
def send_all(bundle, block):
    with ThreadPoolExecutor(max_workers=20) as executor:
        for relay in RELAYS:
            executor.submit(send_bundle, bundle, block, relay)
            print(f"{relay} sent")
