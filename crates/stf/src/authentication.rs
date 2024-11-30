//! The stf-rollup supports `sov-module` authenticator. To support other authentication schemes,
//! you can check out how we support `EVM` authenticator here:
//! https://github.com/Sovereign-Labs/sovereign-sdk-wip/blob/146d5c2c5fa07ab7bb59ba6b2e64690ac9b63830/examples/demo-rollup/stf/src/authentication.rs#L29-L32
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sov_modules_api::capabilities::FatalError;
use sov_state::User;
use std::marker::PhantomData;

use crate::chain_hash::CHAIN_HASH;
use crate::runtime::{Runtime, RuntimeCall};

use sov_modules_api::capabilities::{
    AuthenticationError, AuthenticationOutput, AuthorizationData, UnregisteredAuthenticationError,
};
use sov_modules_api::runtime::capabilities::TransactionAuthenticator;
use sov_modules_api::{DaSpec, DispatchCall, ProvableStateReader, RawTx, Spec};

impl<S: Spec> TransactionAuthenticator<S> for Runtime<S> {
    type Decodable = <Self as DispatchCall>::Decodable;

    type AuthorizationData = AuthorizationData<S>;

    type Input = Auth;

    #[cfg_attr(all(target_os = "zkvm", feature = "bench"), cycle_tracker)]
    fn authenticate<Accessor: ProvableStateReader<User, Spec = S>>(
        &self,
        input: &Self::Input,
        accessor: &mut Accessor,
    ) -> Result<
        AuthenticationOutput<S, Self::Decodable, Self::AuthorizationData>,
        AuthenticationError,
    > {
        match input {
            Auth::Mod(tx) => sov_modules_api::capabilities::authenticate::<Accessor, S, Self>(
                tx,
                &CHAIN_HASH,
                accessor,
            ),
        }
    }

    fn authenticate_unregistered<Accessor: ProvableStateReader<User, Spec = S>>(
        &self,
        raw_tx: &Self::Input,
        accessor: &mut Accessor,
    ) -> Result<
        AuthenticationOutput<S, Self::Decodable, Self::AuthorizationData>,
        UnregisteredAuthenticationError,
    > {
        let Auth::Mod(contents) = raw_tx;

        let (tx_and_raw_hash, auth_data, runtime_call) =
            sov_modules_api::capabilities::authenticate::<Accessor, S, Runtime<S>>(
                contents,
                &CHAIN_HASH,
                accessor,
            )
            .map_err(|e| match e {
                AuthenticationError::FatalError(err, hash) => {
                    UnregisteredAuthenticationError::FatalError(err, hash)
                }
                AuthenticationError::OutOfGas(err) => {
                    UnregisteredAuthenticationError::OutOfGas(err)
                }
            })?;

        match &runtime_call {
            RuntimeCall::SequencerRegistry(sov_sequencer_registry::CallMessage::Register {
                ..
            }) => Ok((tx_and_raw_hash, auth_data, runtime_call)),
            _ => Err(UnregisteredAuthenticationError::FatalError(
                FatalError::Other(
                    "The runtime call included in the transaction was invalid.".to_string(),
                ),
                tx_and_raw_hash.raw_tx_hash,
            ))?,
        }
    }
    fn add_standard_auth(tx: RawTx) -> Self::Input {
        Auth::Mod(tx.data)
    }
}

#[derive(Debug, PartialEq, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum Auth {
    Mod(Vec<u8>),
}

pub struct ModAuth<S: Spec, Da: DaSpec> {
    _phantom: PhantomData<(S, Da)>,
}
