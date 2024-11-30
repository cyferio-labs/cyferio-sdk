use super::test_helpers::{read_private_keys, start_rollup};
use anyhow::Context;
use futures::StreamExt;
use sov_cli::NodeClient;
use sov_mock_da::{BlockProducingConfig, MockAddress, MockDaConfig, MockDaSpec};
use sov_mock_zkvm::MockZkvm;
use sov_modules_api::execution_mode::Native;
use sov_modules_api::macros::config_value;
use sov_modules_api::transaction::{PriorityFeeBips, Transaction, UnsignedTransaction};
use sov_modules_api::Spec;
use sov_modules_rollup_blueprint::logging::default_rust_log_value;
use sov_rollup_interface::common::SafeVec;
use sov_stf_runner::processes::RollupProverConfig;
use std::env;
use std::str::FromStr;
use stf_starter::chain_hash::CHAIN_HASH;
use stf_starter::genesis_config::GenesisPaths;
use stf_starter::Runtime;
use stf_starter::RuntimeCall;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

const TOKEN_NAME: &str = "sov-token";
const MAX_TX_FEE: u64 = 100_000_000;

type TestSpec = sov_modules_api::default_spec::DefaultSpec<MockDaSpec, MockZkvm, MockZkvm, Native>;

#[tokio::test(flavor = "multi_thread")]
async fn bank_tx_tests() -> Result<(), anyhow::Error> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::from_str(
                &env::var("RUST_LOG").unwrap_or_else(|_| default_rust_log_value().to_string()),
            )
            .unwrap(),
        )
        .init();
    let (rpc_port_tx, rpc_port_rx) = tokio::sync::oneshot::channel();
    let (rest_port_tx, rest_port_rx) = tokio::sync::oneshot::channel();

    let rollup_task = tokio::spawn(async {
        start_rollup(
            rpc_port_tx,
            rest_port_tx,
            GenesisPaths::from_dir("../../test-data/genesis/mock/"),
            RollupProverConfig::Skip,
            MockDaConfig {
                connection_string: "sqlite::memory:".to_string(),
                sender_address: MockAddress::new([0; 32]),
                finalization_blocks: 3,
                block_producing: BlockProducingConfig::OnBatchSubmit,
                block_time_ms: 100_000,
            },
        )
        .await;
    });
    let _ = rpc_port_rx.await;
    let rest_port = rest_port_rx.await?.port();
    let client = NodeClient::new_at_localhost(rest_port).await?;

    // If the rollup throws an error, return it and stop trying to send the transaction
    tokio::select! {
        err = rollup_task => err?,
        res = send_test_create_token_tx(&client) => res?,
    }
    Ok(())
}

async fn send_test_create_token_tx(client: &NodeClient) -> Result<(), anyhow::Error> {
    let key_and_address = read_private_keys::<TestSpec>("tx_signer_private_key.json");
    let key = key_and_address.private_key;
    let user_address: <TestSpec as Spec>::Address = key_and_address.address;

    let token_id = sov_bank::get_token_id::<TestSpec>(TOKEN_NAME, &user_address);
    let initial_balance = 1000;

    let msg = RuntimeCall::<TestSpec>::Bank(sov_bank::CallMessage::<TestSpec>::CreateToken {
        token_name: TOKEN_NAME.try_into().unwrap(),
        initial_balance,
        mint_to_address: user_address,
        authorized_minters: SafeVec::default(),
    });
    let chain_id = config_value!("CHAIN_ID");
    let nonce = 0;
    let max_priority_fee = PriorityFeeBips::ZERO;
    let gas_limit = None;
    let tx = Transaction::<Runtime<TestSpec>, TestSpec>::new_signed_tx(
        &key,
        &CHAIN_HASH,
        UnsignedTransaction::new(
            msg,
            chain_id,
            max_priority_fee,
            MAX_TX_FEE,
            nonce,
            gas_limit,
        ),
    );

    let mut slot_subscription = client
        .client
        .subscribe_slots()
        .await
        .context("Failed to subscribe to slots!")?;

    client
        .client
        .publish_batch_with_serialized_txs(&[tx])
        .await?;

    // Wait until the rollup has processed the next slot
    let _slot_number = slot_subscription
        .next()
        .await
        .transpose()?
        .map(|slot| slot.number)
        .unwrap_or_default();

    let balance = client
        .get_balance::<TestSpec>(&user_address, &token_id, None)
        .await?;
    assert_eq!(initial_balance, balance);

    Ok(())
}
