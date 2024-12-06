#![deny(missing_docs)]
//! StarterRollup provides a minimal self-contained rollup implementation

use anyhow::Error;
use async_trait::async_trait;
use sov_cyferio_hub::service::DaProvider;
use sov_cyferio_hub::spec::CyferioSpec;
use sov_db::ledger_db::LedgerDb;
use sov_db::storage_manager::NativeStorageManager;

use sov_mock_zkvm::MockZkvmHost;
use sov_mock_zkvm::{MockCodeCommitment, MockZkvm};
use sov_modules_api::default_spec::DefaultSpec;
use sov_modules_api::rest::StateUpdateReceiver;
use sov_modules_api::RuntimeEndpoints;
use sov_modules_api::SyncStatus;
use sov_modules_api::{CryptoSpec, Spec};
use sov_modules_api::{DaSyncState, ZkVerifier};
use sov_modules_rollup_blueprint::pluggable_traits::PluggableSpec;
use sov_modules_rollup_blueprint::proof_serializer::SovApiProofSerializer;
use sov_modules_rollup_blueprint::{FullNodeBlueprint, RollupBlueprint};
#[cfg(feature = "risc0")]
use sov_risc0_adapter::host::Risc0Host;
#[cfg(feature = "risc0")]
use sov_risc0_adapter::Risc0;
use sov_rollup_interface::execution_mode::{ExecutionMode, Native};
use sov_rollup_interface::node::da::DaServiceWithRetries;
use sov_rollup_interface::zk::aggregated_proof::CodeCommitment;
use sov_sequencer::SequencerDb;
#[cfg(feature = "sp1")]
use sov_sp1_adapter::{host::SP1Host, SP1Verifier, SP1};
use sov_state::Storage;
use sov_state::{DefaultStorageSpec, ProverStorage};
use sov_stf_runner::processes::{ParallelProverService, ProverService, RollupProverConfig};
use sov_stf_runner::RollupConfig;
use std::sync::Arc;
use stf_starter::Runtime;
use tokio::sync::watch::{self};

/// Rollup with [`MockDaService`].
#[derive(Default)]
pub struct CyferioRollup<M> {
    phantom: std::marker::PhantomData<M>,
}

/// This is the place where all the rollup components come together, and
/// they can be easily swapped with alternative implementations as needed.
#[cfg(feature = "risc0")]
impl<M: ExecutionMode> RollupBlueprint<M> for CyferioRollup<M>
where
    DefaultSpec<CyferioSpec, Risc0, MockZkvm, M>: PluggableSpec,
{
    type Spec = DefaultSpec<CyferioSpec, Risc0, MockZkvm, M>;
    type Runtime = Runtime<Self::Spec>;
}
#[cfg(feature = "sp1")]
impl<M: ExecutionMode> RollupBlueprint<M> for MockRollup<M>
where
    DefaultSpec<MockDaSpec, SP1, MockZkvm, M>: PluggableSpec,
{
    type Spec = DefaultSpec<MockDaSpec, SP1, MockZkvm, M>;
    type Runtime = Runtime<Self::Spec>;
}

#[async_trait]
impl FullNodeBlueprint<Native> for CyferioRollup<Native> {
    type DaService = DaServiceWithRetries<DaProvider>;
    /// Manager for the native storage lifecycle.
    type StorageManager = NativeStorageManager<
        CyferioSpec,
        ProverStorage<DefaultStorageSpec<<<Self::Spec as Spec>::CryptoSpec as CryptoSpec>::Hasher>>,
    >;
    /// Prover service.
    type ProverService = ParallelProverService<
        <Self::Spec as Spec>::Address,
        <<Self::Spec as Spec>::Storage as Storage>::Root,
        <<Self::Spec as Spec>::Storage as Storage>::Witness,
        Self::DaService,
        <Self::Spec as Spec>::InnerZkvm,
        <Self::Spec as Spec>::OuterZkvm,
    >;

    type ProofSerializer = SovApiProofSerializer<Self::Spec>;

    fn create_outer_code_commitment(
        &self,
    ) -> <<Self::ProverService as ProverService>::Verifier as ZkVerifier>::CodeCommitment {
        MockCodeCommitment::default()
    }

    async fn create_endpoints(
        &self,
        state_update_receiver: StateUpdateReceiver<<Self::Spec as Spec>::Storage>,
        sync_status_receiver: watch::Receiver<SyncStatus>,
        ledger_db: &LedgerDb,
        sequencer_db: &SequencerDb,
        da_service: &Self::DaService,
        da_sync_state: Arc<DaSyncState>,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
        shutdown_receiver: tokio::sync::watch::Receiver<()>,
    ) -> Result<RuntimeEndpoints, Error> {
        sov_modules_rollup_blueprint::register_endpoints::<Self, Native>(
            state_update_receiver.clone(),
            sync_status_receiver,
            ledger_db,
            sequencer_db,
            da_service,
            da_sync_state,
            rollup_config,
            shutdown_receiver,
        )
        .await
    }

    async fn create_da_service(
        &self,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
    ) -> Self::DaService {
        DaServiceWithRetries::new_fast(
            DaProvider::from_config(rollup_config.da.clone())
                .await
                .unwrap(),
        )
    }

    async fn create_prover_service(
        &self,
        prover_config: RollupProverConfig,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
        _da_service: &Self::DaService,
    ) -> Self::ProverService {
        #[cfg(feature = "risc0")]
        let inner_vm = if let RollupProverConfig::Skip = prover_config {
            Risc0Host::new(b"")
        } else {
            let elf = std::fs::read(risc0_starter::MOCK_DA_PATH)
                .unwrap_or_else(|e| {
                    panic!(
                        "Could not read guest elf file from `{}`. {}",
                        risc0_starter::MOCK_DA_PATH,
                        e
                    )
                })
                .leak();
            Risc0Host::new(elf)
        };

        #[cfg(feature = "sp1")]
        let inner_vm = if let RollupProverConfig::Skip = prover_config {
            SP1Host::new(b"")
        } else {
            let elf = &sp1_starter::SP1_GUEST_MOCK_ELF;
            SP1Host::new(elf)
        };

        let outer_vm = MockZkvmHost::new_non_blocking();
        let da_verifier = Default::default();

        ParallelProverService::new_with_default_workers(
            inner_vm,
            outer_vm,
            da_verifier,
            prover_config,
            CodeCommitment::default(),
            rollup_config.proof_manager.prover_address,
        )
    }

    fn create_storage_manager(
        &self,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
    ) -> Result<Self::StorageManager, Error> {
        NativeStorageManager::new(&rollup_config.storage.path)
    }

    fn create_proof_serializer(
        &self,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
        sequencer_db: &SequencerDb,
    ) -> anyhow::Result<Self::ProofSerializer> {
        Ok(Self::ProofSerializer::new(
            sequencer_db,
            rollup_config.sequencer.is_preferred_sequencer(),
        ))
    }
}

impl sov_modules_rollup_blueprint::WalletBlueprint<Native> for CyferioRollup<Native> {}
