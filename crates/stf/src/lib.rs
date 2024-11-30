//! The rollup State Transition Function.

pub mod authentication;
pub mod chain_hash;
#[cfg(feature = "native")]
pub mod genesis_config;
pub mod hooks;
pub mod runtime;

pub use runtime::*;
use sov_modules_stf_blueprint::StfBlueprint;
use sov_rollup_interface::stf::StateTransitionVerifier;

pub extern crate sov_modules_api;

/// Alias for StateTransitionVerifier.
pub type StfVerifier<DA, ZkSpec, RT, InnerVm, OuterVm> =
    StateTransitionVerifier<StfBlueprint<ZkSpec, RT>, DA, InnerVm, OuterVm>;

pub use sov_mock_da::MockDaSpec;
