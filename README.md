# Cyferio SDK

> [!IMPORTANT]  
> To try out the FHE module, please visit [`feat/confidential-token`](https://github.com/cyferio-labs/cyferio-sdk/tree/feat/confidential-token) branch

## Overview

Cyferio SDK is a modular rollup framework with Fully Homomorphic Encryption (FHE) integration that simplifies the creation and management of confidential rollups, providing developers with the necessary tools to build privacy-preserving applications.

By leveraging FHE, advanced modular rollup designs, and parallelism in computational proofs within a trustless computing layer, Cyferio SDK enables secure, near real-time computations on both public and private on-chain states while preserving composability and interoperability.

## Key Features

- **Modular Architecture**: Highly adaptable zk-rollup framework integrating state-of-the-art privacy-preserving solutions like FHE and Zero-Knowledge Proofs (ZKPs).
  
- **Module System Interface**:
  - Supports both stateless and stateful modules, enhancing composability.
  - Incorporates FHE-powered modules using the TFHE-rs library for computations on encrypted data.

- **Data Availability Interface**:
  - Integrates with various data availability solutions (e.g., Celestia, Avail).
  - Compatible with mainstream Layer 1 blockchains for settlement layers.

- **zkVM Interface**:
  - Supports optimistic, zero-knowledge, and verifiable FHE virtual machines.
  - Compatible with various zkVMs, including RISC Zero and SP1.
  - Produces succinct verifiable proofs for transaction executions.

- **Threshold Service Network**:
  - Secure key management for FHE keys.
  - Robust FHE key generation and threshold decryption using MPC protocols.

<p align="center">
 <img src="assets/Cyferio SDK Arch.png" alt="TMC architecture"/>
    <br>
    <em>The Architecture of Cyferio SDK</em>
</p>

## Use Cases

- **DeFi**:
  - **Dark Pools**: Enable private large trades to reduce market impact.
  - **Blind Auctions**: Conduct auctions with hidden bids to prevent manipulation.
  - **MEV-Resistant DEXs**: Build exchanges where transactions can't be front-run.
  - **Private Prediction Markets**: Enable on-chain prediction markets with confidential betting.

- **Social Applications**:
  - **Efficient Identity Verification**: Perform identity checks without constant off-chain data retrieval.
  - **Privacy-Preserving Interactions**: Ensure all user interactions remain private.

- **Gaming**:
  - **Real-Time Response**: Enable near real-time transaction responses in distributed systems.
  - **Secure Interactions**: Operate nodes in a "dark forest" state for enhanced security.
  - **Asset Integration**: Flexible combination of DeFi and GameFi assets.
  - **Flexible Gas Fees**: Implement dynamic gas fee structures to lower entry barriers.

<p align="center">
 <img src="assets/Cyferio SDK flow.png" alt="Cyferio SDK flow"/>
    <br>
    <em>The Workflow of Cyferio SDK</em>
</p>

## Getting Started

Cyferio SDK is integrated with the [`Cyferio Hub`](https://github.com/cyferio-labs/cyferio-hub-node) in default. That means for local development environment, you have to run the Cyferio Hub node as DA layer before creating your confidential rollup with Cyferio SDK. Please refer to the [Cyferio Hub README](https://github.com/cyferio-labs/cyferio-hub-node/blob/main/README.md) for more details.

Note that for local debugging, you can simply switch to `mock_da` by changing the feature flag in `crates/rollup/Cargo.toml`

```
[features]
default = ["mock_da", "risc0"]
```

## How to run the demo rollup with mock-da

1. Change the working directory:

```shell,test-ci
$ cd crates/rollup/
```

2. If you want to run a fresh rollup, clean the database:

```sh,test-ci
$ make clean-db
```

3. Start the rollup node:

This will compile and start the rollup node:

```shell,test-ci,bashtestmd:long-running,bashtestmd:wait-until=RPC
$ cargo run --bin node
```

4. Submit a token creation transaction to the `bank` module:

```sh,test-ci
$ make test-create-token
```

5. Note the transaction hash from the output of the above command

```text
Submitting tx: 0: 0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db
Transaction 0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db has been submitted: AcceptTxResponse { data: TxInfo { id: TxHash("0xa02ed59b5c698d49ad088584b86aff2134fd8e96746c1fce57b2518eb7c843e2"), status: Submitted }, meta: {} }
Triggering batch publishing
Your batch was submitted to the sequencer for publication. Response: SubmittedBatchInfo { da_height: 2, num_txs: 1 }
Going to wait for target slot number 2 to be processed, up to 300s
Rollup has processed target DA height=2!
```

6. To get the token address, fetch the events of the transaction hash from #5

```bash,test-ci
$ curl -sS http://127.0.0.1:12346/ledger/txs/0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db | jq
{
  "data": {
    "type": "tx",
    "number": 0,
    "hash": "0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db",
    "event_range": {
      "start": 0,
      "end": 1
    },
    "body": "",
    "receipt": {
      "result": "successful",
      "data": {
        "gas_used": [
          3296,
          3296
        ]
      }
    },
    "events": [],
    "batch_number": 0
  },
  "meta": {}
}
$ curl -sS http://127.0.0.1:12346/ledger/txs/0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db/events | jq
{
  "data": [
    {
      "type": "event",
      "number": 0,
      "key": "Bank/TokenCreated",
      "value": {
        "TokenCreated": {
          "token_name": "sov-test-token",
          "coins": {
            "amount": 1000000,
            "token_id": "token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42"
          },
          "minter": {
            "User": "sov15vspj48hpttzyvxu8kzq5klhvaczcpyxn6z6k0hwpwtzs4a6wkvqwr57gc"
          },
          "authorized_minters": [
            {
              "User": "sov1l6n2cku82yfqld30lanm2nfw43n2auc8clw7r5u5m6s7p8jrm4zqrr8r94"
            },
            {
              "User": "sov15vspj48hpttzyvxu8kzq5klhvaczcpyxn6z6k0hwpwtzs4a6wkvqwr57gc"
            }
          ]
        }
      },
      "module": {
        "type": "moduleRef",
        "name": "bank"
      }
    }
  ],
  "meta": {}
}
```

7. Get a total supply of the token:

```bash,test-ci,bashtestmd:compare-output
$ curl -Ss http://127.0.0.1:12346/modules/bank/tokens/token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42/total-supply | jq -c -M
{"data":{"amount":1000000,"token_id":"token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42"},"meta":{}}
```

Feel free to explore and contribute to the project. For any questions or issues, please open an issue or contact the maintainers.
