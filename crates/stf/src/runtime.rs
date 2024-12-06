#![allow(unused_doc_comments)]
//! This module implements the core logic of the rollup.
//! To add new functionality to your rollup:
//!   1. Add a new module dependency to your `Cargo.toml` file
//!   2. Add the module to the `Runtime` below
//!   3. Update `genesis.json` with any additional data required by your new module
#[cfg(feature = "native")]
use std::sync::Arc;

use sov_capabilities::StandardProvenRollupCapabilities as StandardCapabilities;
use sov_kernels::soft_confirmations::SoftConfirmationsKernel;
#[cfg(feature = "native")]
use sov_modules_api::capabilities::KernelWithSlotMapping;
use sov_modules_api::capabilities::{AuthorizationData, Guard, HasCapabilities, HasKernel};
#[cfg(feature = "native")]
use sov_modules_api::macros::{expose_rpc, CliWallet};
use sov_modules_api::prelude::*;
use sov_modules_api::{BlobDataWithId, OperatingMode};
use sov_modules_api::{DispatchCall, Event, Genesis, MessageCodec, Spec};
use sov_rollup_interface::da::DaSpec;

use crate::chain_hash;
#[cfg(feature = "native")]
use crate::genesis_config::GenesisPaths;

/// The runtime defines the logic of the rollup.
///
/// At a high level, the rollup node receives serialized "call messages" from the DA layer and executes them as atomic transactions.
/// Upon reception, the message is deserialized and forwarded to an appropriate module.
///
/// The module-specific logic is implemented by module creators, but all the glue code responsible for message
/// deserialization/forwarding is handled by a rollup `runtime`.
///
/// To define the runtime, we need to specify all the modules supported by our rollup (see the `Runtime` struct bellow)
///
/// The `Runtime` defines:
/// - how the rollup modules are wired up together.
/// - how the state of the rollup is initialized.
/// - how messages are dispatched to appropriate modules.
///
/// Runtime lifecycle:
///
/// 1. Initialization:
///     When a rollup is deployed for the first time, it needs to set its genesis state.
///     The `#[derive(Genesis)]` macro will generate `Runtime::genesis(config)` method which returns
///     `Storage` with the initialized state.
///
/// 2. Calls:      
///     The `Module` interface defines a `call` method which accepts a module-defined type and triggers the specific `module logic.`
///     In general, the point of a call is to change the module state, but if the call throws an error,
///     no state is updated (the transaction is reverted).
///
/// `#[derive(MessageCodec)]` adds deserialization capabilities to the `Runtime` (by implementing the `decode_call` method).
/// `Runtime::decode_call` accepts a serialized call message and returns a type that implements the `DispatchCall` trait.
///  The `DispatchCall` implementation (derived by a macro) forwards the message to the appropriate module and executes its `call` method.
#[derive(Default, Genesis, DispatchCall, Event, MessageCodec, RuntimeRestApi)]
#[dispatch_call(derive(UniversalWallet))]
#[cfg_attr(feature = "native", derive(CliWallet), expose_rpc)]
pub struct Runtime<S: Spec> {
    /// The `accounts` module is responsible for managing user accounts.
    pub accounts: sov_accounts::Accounts<S>,
    /// The Nonces module.
    pub nonces: sov_nonces::Nonces<S>,
    /// The bank module is responsible for minting, transferring, and burning tokens
    pub bank: sov_bank::Bank<S>,
    /// The sequencer registry module is responsible for authorizing users to sequencer rollup transactions
    pub sequencer_registry: sov_sequencer_registry::SequencerRegistry<S>,
    /// The Attester Incentives module.
    pub attester_incentives: sov_attester_incentives::AttesterIncentives<S>,
    /// The Prover Incentives module.
    pub prover_incentives: sov_prover_incentives::ProverIncentives<S>,
    /// The example module.
    pub example_module: example_module::ExampleModule<S>,
    /// The Chain state module.
    pub chain_state: sov_chain_state::ChainState<S>,
    /// The Blob storage module.
    pub blob_storage: sov_blob_storage::BlobStorage<S>,
    /// The Paymaster module.
    pub paymaster: sov_paymaster::Paymaster<S>,
    /// The confidential token module.
    pub confidential_token: confidential_token::Bank<S>,
}

impl<S: Spec> sov_modules_stf_blueprint::Runtime<S> for Runtime<S>
where
    S::Da: DaSpec,
{
    const CHAIN_HASH: [u8; 32] = chain_hash::CHAIN_HASH;
    type GenesisConfig = GenesisConfig<S>;

    #[cfg(feature = "native")]
    type GenesisPaths = GenesisPaths;

    #[cfg(feature = "native")]
    fn endpoints(
        api_state: sov_modules_api::rest::ApiState<S>,
    ) -> sov_modules_api::RuntimeEndpoints {
        use ::sov_modules_api::rest::HasRestApi;
        use ::sov_rollup_apis::dedup::{DeDupEndpoint, NonceDeDupEndpoint};

        let axum_router = Self::default().rest_api(api_state.clone());
        // Provide an endpoint to return dedup information associated with addresses.
        // Since our runtime is using the nonces module we can use the provided `NonceDeDupEndpoint` implementation.
        let dedup_endpoint = NonceDeDupEndpoint::new(api_state.clone());
        let axum_router = axum_router.merge(dedup_endpoint.axum_router());

        sov_modules_api::RuntimeEndpoints {
            axum_router,
            jsonrpsee_module: get_rpc_methods::<S>(api_state),
            background_handles: Vec::new(),
        }
    }

    #[cfg(feature = "native")]
    fn genesis_config(
        genesis_paths: &Self::GenesisPaths,
    ) -> Result<Self::GenesisConfig, anyhow::Error> {
        crate::genesis_config::create_genesis_config(genesis_paths)
    }

    fn operating_mode(genesis: &Self::GenesisConfig) -> OperatingMode {
        genesis.chain_state.operating_mode
    }
}

impl<S: Spec> HasCapabilities<S> for Runtime<S> {
    type Capabilities<'a> = StandardCapabilities<'a, S, sov_paymaster::Paymaster<S>>;
    type AuthorizationData = AuthorizationData<S>;

    fn capabilities(&self) -> Guard<Self::Capabilities<'_>> {
        Guard::new(StandardCapabilities {
            bank: &self.bank,
            sequencer_registry: &self.sequencer_registry,
            accounts: &self.accounts,
            nonces: &self.nonces,
            gas_payer: &self.paymaster,
            attester_incentives: &self.attester_incentives,
            prover_incentives: &self.prover_incentives,
        })
    }
}

impl<S: Spec> HasKernel<S> for Runtime<S> {
    type BlobType = BlobDataWithId;
    type Kernel<'a> = SoftConfirmationsKernel<'a, S>;

    fn inner(&self) -> Guard<Self::Kernel<'_>> {
        Guard::new(SoftConfirmationsKernel {
            chain_state: &self.chain_state,
            blob_storage: &self.blob_storage,
        })
    }

    #[cfg(feature = "native")]
    fn kernel_with_slot_mapping(&self) -> Arc<dyn KernelWithSlotMapping<S>> {
        Arc::new(self.chain_state.clone())
    }
}
