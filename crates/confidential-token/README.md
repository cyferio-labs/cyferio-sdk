# confidential-token
## Rollup starter for `confidential-token`

## Setup

### Configuring the Environment
Currently, we are using an optimistic-like rollup configuration since applying a zk prover to FHE is still under development.

Set the following environment variables in your terminal:
```sh
export SKIP_GUEST_BUILD=1
export SOV_PROVER_MODE=skip
```

### Setting Up FHE Keys
The rollup requires a set of keys: `{public key, server key, private key}`. Ideally, the public and server keys should be securely stored on-chain, while the private key should be safely stored within the node.

For demo purposes, this example stores the keys in a JSON file, which is insecure and not recommended for production.

Generate the keys by running:
```sh
# run this command at the project root directory

cargo run --release --bin fhe-keygen
```

### Generating Call Messages for `confidential-token`
These requests will be used to invoke confidential operations like token creation, transfer, and minting.

Generate the required scripts by running:
```sh
# run this command at the project root directory

cargo run --release --bin request-scripts-gen
```

## Running the Node
1. Navigate to the rollup directory:
    ```sh
    cd crates/rollup/
    ```
2. If you want to start a fresh rollup, clean the database and wallet:
    ```sh
    make clean-db
    make clean-wallet
    ```
3. Start the rollup node:

    This command will compile and start the rollup node:
    ```sh
    # Ensure environment variables are set
    # Use --release for faster FHE operations

    cargo run --release --bin node
    ```

## Interacting with `sov-bank-fhe`

### 1. Open a New Terminal

In a new terminal window, navigate to the `crates/rollup/` directory and set the environment variables.

### 2. Build `sov-cli` and Import Keys

Import the keys for the token deployer with config in `test-data/keys/token_deployer_private_key.json`

```sh
make import-keys
```

### 3. Query the FHE Public Key via RPC

```sh
make get-fhe-public-key
```

### 4. Create Confidential Tokens

This command creates and mints 1,000 tokens (encrypted with FHE) to the address `sov1l6n2cku82yfqld30lanm2nfw43n2auc8clw7r5u5m6s7p8jrm4zqrr8r94`. The encrypted token amount is stored on-chain in a compressed format.

```sh
# Wait 5–10 seconds for the transaction to complete

make test-fhe-create-token
```

Check the server logs for ongoing FHE operations.

### 5. Query the Total Supply of Tokens

Fetch the total token supply, both in its encrypted form and plaintext.

```sh
# Query in FHE ciphertext
make test-fhe-bank-raw-supply-of

# Query in plaintext (typically restricted to authorized addresses)
make test-fhe-bank-supply-of
```

### 6. Mint Additional Confidential Tokens

This command mints 500 tokens (encrypted with FHE) to the address `sov1l6n2cku82yfqld30lanm2nfw43n2auc8clw7r5u5m6s7p8jrm4zqrr8r94`. You can verify the updated total supply via RPC.

```sh
# Wait 5–10 seconds for the transaction to complete

make test-fhe-mint-token
```

### 7. Transfer Confidential Tokens

Transfer 100 tokens (encrypted with FHE) from `sov1l6n2cku82yfqld30lanm2nfw43n2auc8clw7r5u5m6s7p8jrm4zqrr8r94` to `sov15vspj48hpttzyvxu8kzq5klhvaczcpyxn6z6k0hwpwtzs4a6wkvqwr57gc`.

```sh
# Wait 5–10 seconds for the transaction to complete

make test-fhe-token-transfer
```

### 8. Query the Raw User Balance via RPC

To check the balance in ciphertext:

```sh
curl -sS -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"fheBank_rawBalanceOf","params":{"user_address":"PASTE_ADDRESS_HERE", "token_id":"token_1jj28lgplh28xfefte5990tnt6ka7gdy6z8zudh38z02yp3jrvhysrczk99"},"id":1}' http://127.0.0.1:12345
```

Example command for checking the balance of `sov1l6n2cku82yfqld30lanm2nfw43n2auc8clw7r5u5m6s7p8jrm4zqrr8r94`:

```sh
curl -sS -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"fheBank_rawBalanceOf","params":{"user_address":"sov1l6n2cku82yfqld30lanm2nfw43n2auc8clw7r5u5m6s7p8jrm4zqrr8r94", "token_id":"token_1jj28lgplh28xfefte5990tnt6ka7gdy6z8zudh38z02yp3jrvhysrczk99"},"id":1}' http://127.0.0.1:12345
```

To check the balance <u>**in plaintext**</u>:

```sh
curl -sS -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"fheBank_balanceOf","params":{"user_address":"PASTE_ADDRESS_HERE", "token_id":"token_1jj28lgplh28xfefte5990tnt6ka7gdy6z8zudh38z02yp3jrvhysrczk99"},"id":1}' http://127.0.0.1:12345
```