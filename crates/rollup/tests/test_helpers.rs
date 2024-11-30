use sha2::Sha256;
use sov_cli::wallet_state::PrivateKeyAndAddress;
use sov_sequencer::batch_builders::standard::StdBatchBuilderConfig;
use sov_sequencer::BatchBuilderConfig;
use std::net::SocketAddr;
use std::path::Path;

use sov_mock_da::MockDaConfig;
use sov_modules_api::{Address, Spec};
use sov_modules_rollup_blueprint::FullNodeBlueprint;
use sov_rollup_starter::mock_rollup::MockRollup;
use sov_sequencer::batch_builders::preferred::PreferredBatchBuilderConfig;
use sov_sequencer::SequencerConfig;
use sov_stf_runner::processes::RollupProverConfig;
use sov_stf_runner::{HttpServerConfig, MonitoringConfig, ProofManagerConfig};
use sov_stf_runner::{RollupConfig, RunnerConfig, StorageConfig};
use std::str::FromStr;
use stf_starter::genesis_config::GenesisPaths;
use tokio::sync::oneshot;

const PROVER_ADDRESS: &str = "sov1pv9skzctpv9skzctpv9skzctpv9skzctpv9skzctpv9skzctpv9stup8tx";

pub async fn start_rollup(
    rpc_reporting_channel: oneshot::Sender<SocketAddr>,
    rest_reporting_channel: oneshot::Sender<SocketAddr>,
    rt_genesis_paths: GenesisPaths,
    rollup_prover_config: RollupProverConfig,
    da_config: MockDaConfig,
) {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();
    let sequencer_address = da_config.sender_address;

    let rollup_config = RollupConfig {
        storage: StorageConfig {
            path: temp_path.to_path_buf(),
        },
        runner: RunnerConfig {
            genesis_height: 0,
            da_polling_interval_ms: 200,
            rpc_config: HttpServerConfig::localhost_on_free_port(),
            axum_config: HttpServerConfig::localhost_on_free_port(),
            concurrent_sync_tasks: Some(1),
        },
        da: da_config,
        proof_manager: ProofManagerConfig {
            aggregated_proof_block_jump: 1,
            prover_address: Address::<Sha256>::from_str(PROVER_ADDRESS)
                .expect("Prover address is not valid"),
            max_number_of_transitions_in_db: 100,
            max_number_of_transitions_in_memory: 20,
        },
        sequencer: SequencerConfig {
            max_allowed_blocks_behind: 5,
            automatic_batch_production: false,
            da_address: sequencer_address,
            batch_builder: BatchBuilderConfig::preferred(PreferredBatchBuilderConfig {
                should_update_state: true,
            }),
            dropped_tx_ttl_secs: 0,
        },
        monitoring: MonitoringConfig::standard(),
    };

    let mock_demo_rollup = MockRollup::default();

    let rollup = mock_demo_rollup
        .create_new_rollup(&rt_genesis_paths, rollup_config, Some(rollup_prover_config))
        .await
        .unwrap();

    rollup
        .run_and_report_addr(Some(rpc_reporting_channel), Some(rest_reporting_channel))
        .await
        .unwrap();

    // Close the tempdir explicitly to ensure that rustc doesn't see that it's unused and drop it unexpectedly
    temp_dir.close().unwrap();
}

pub fn read_private_keys<S: Spec>(suffix: &str) -> PrivateKeyAndAddress<S> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let private_keys_dir = Path::new(&manifest_dir).join("../../test-data/keys");

    let data = std::fs::read_to_string(private_keys_dir.join(suffix))
        .expect("Unable to read file to string");

    let key_and_address: PrivateKeyAndAddress<S> =
        serde_json::from_str(&data).unwrap_or_else(|_| {
            panic!("Unable to convert data {} to PrivateKeyAndAddress", &data);
        });

    assert!(
        key_and_address.is_matching_to_default(),
        "Inconsistent key data"
    );

    key_and_address
}
