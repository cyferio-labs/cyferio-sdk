# Overview

This package is a convenient starting point for building a rollup using the Sovereign SDK:

## The repo structure:

- `crates/stf`: The `STF` is derived from the `Runtime` and is used in the `rollup` and `provers` crates.
- `crates/provers`: This crate is responsible for creating proofs for the `STF`.
- `crates/rollup`: This crate runs the `STF` and offers additional full-node functionalities.

(!) Note for using WIP repo.
This repo utilizes private [Sovereign SDK repo](https://github.com/Sovereign-Labs/sovereign-sdk-wip) and default cargo needs this environment variable to use an SSH key:

```
export CARGO_NET_GIT_FETCH_WITH_CLI=true
```

# Running the sov-rollup-starter

## How to run the sov-rollup-starter with mock-da

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

## How to run the sov-rollup-starter using Celestia Da

1. Change the working directory:
   ```bash
   $ cd crates/rollup/
   ```
2. If you want to run a fresh rollup, clean the database:
   ```bash
   $ make clean
   ```
3. Start the Celestia local docker service. (make sure you have docker daemon running).
   ```bash
   $ make start
   ```
4. Start the rollup node with the feature flag building with the celestia adapter. To build with the sp1 prover, you may replace `risc0` with `sp1`.
   This will compile and start the rollup node:
   ```bash
   $ cargo run --bin node --no-default-features --features celestia_da,risc0
   ```
5. Submit a token creation transaction to the `bank` module.
   To build with the sp1 prover, you may replace `risc0` with `sp1`.
   Using `CELESTIA=1` will enable the client to be built with Celestia support and submit the test token
   ```bash
   $ CELESTIA=1 ZKVM=risc0 make test-create-token
   ```
6. Note the transaction hash from the output of the above command
   ```text
   Submitting tx: 0: 0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db
   Transaction 0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db has been submitted: AcceptTxResponse { data: TxInfo { id: TxHash("0xa02ed59b5c698d49ad088584b86aff2134fd8e96746c1fce57b2518eb7c843e2"), status: Submitted }, meta: {} }
   Triggering batch publishing
   Your batch was submitted to the sequencer for publication. Response: SubmittedBatchInfo { da_height: 2, num_txs: 1 }
   Going to wait for target slot number 2 to be processed, up to 300s
   Rollup has processed target DA height=2!
   ```
7. To get the token address, fetch the events of the transaction hash from #5

   ```bash
   $ curl -sS http://127.0.0.1:12346/ledger/txs/0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db
   # Output omitted, should be similar to what has been seen in mock-da section
   ```

8. Get a total supply of the token:

```bash,test-ci,bashtestmd:compare-output
$ curl -Ss http://127.0.0.1:12346/modules/bank/tokens/token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42/total-supply | jq -c -M
{"data":{"amount":1000000,"token_id":"token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42"},"meta":{}}
```

## Enabling the prover

By default, demo-rollup disables proving (i.e. the default behavior is. If we want to enable proving, several options are available:

- `export SOV_PROVER_MODE=skip` Skips verification logic.
- `export SOV_PROVER_MODE=simulate` Run the rollup verification logic inside the current process.
- `export SOV_PROVER_MODE=execute` Run the rollup verifier in a zkVM executor.
- `export SOV_PROVER_MODE=prove` Run the rollup verifier and create a SNARK of execution.

#### Note: Check that users and sequencers have enough tokens to pay for the transactions.

For the transaction to be processed successfully, you have to ensure that the sender account has enough funds to pay for the transaction fees and the sequencer has staked enough tokens to pay for the pre-execution checks. This `README` file uses addresses from the `./test-data/genesis/demo/mock` folder, which are pre-populated with enough funds.

To be able to execute most simple transactions, the transaction sender should have about `1_000_000_000` tokens on their account and the sequencer should have staked `100_000_000` tokens in the registry.

More details can be found in the Sovereign book [available here](https://github.com/Sovereign-Labs/sovereign-book).
