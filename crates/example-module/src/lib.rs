use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sov_modules_api::macros::UniversalWallet;
use sov_modules_api::prelude::*;
use sov_modules_api::{
    Context, Error, EventEmitter, GenesisState, Module, ModuleId, ModuleInfo, ModuleRestApi, Spec,
    StateValue, TxState,
};

#[cfg(feature = "native")]
pub use query::*;

/// A new module:
/// - Must derive `ModuleInfo`
/// - Must contain `[id]` field
/// - Can contain any number of ` #[state]` or `[module]` fields
/// - Can derive ModuleRestApi to automatically generate Rest API endpoints
#[derive(Clone, ModuleInfo, ModuleRestApi)]
pub struct ExampleModule<S: Spec> {
    /// Id of the module.
    #[id]
    pub id: ModuleId,

    /// Some value kept in the state.
    #[state]
    pub value: StateValue<u32>,

    /// Reference to the Bank module.
    #[module]
    pub(crate) _bank: sov_bank::Bank<S>,
}

impl<S: Spec> Module for ExampleModule<S> {
    type Spec = S;

    type Config = ExampleModuleConfig;

    type CallMessage = CallMessage;

    type Event = Event;

    fn genesis(
        &self,
        _header: &<S::Da as sov_modules_api::DaSpec>::BlockHeader,
        _validity: &<S::Da as sov_modules_api::DaSpec>::ValidityCondition,
        _config: &Self::Config,
        _state: &mut impl GenesisState<S>,
    ) -> anyhow::Result<(), Error> {
        // The initialization logic
        Ok(())
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        _context: &Context<Self::Spec>,
        state: &mut impl TxState<S>,
    ) -> anyhow::Result<(), Error> {
        match msg {
            CallMessage::SetValue(new_value) => {
                self.value
                    .set(&new_value, state)
                    .map_err(Into::<anyhow::Error>::into)?;
                self.emit_event(state, Event::Set { value: new_value });

                Ok(())
            }
        }
    }
}

/// This enumeration represents the available call messages for interacting with
/// the `ExampleModule` module.
/// The `derive` for [`schemars::JsonSchema`] is a requirement of
/// [`sov_modules_api::ModuleCallJsonSchema`].
#[derive(
    BorshDeserialize,
    BorshSerialize,
    Debug,
    schemars::JsonSchema,
    PartialEq,
    Eq,
    UniversalWallet,
    Clone,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum CallMessage {
    SetValue(u32),
}

/// Example Event
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// Example event set value
    Set { value: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExampleModuleConfig {}

#[cfg(feature = "native")]
mod query {
    use crate::ExampleModule;
    use crate::{Deserialize, Serialize};
    use sov_modules_api::prelude::UnwrapInfallible;
    use sov_modules_api::ApiStateAccessor;

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Response {
        pub value: Option<u32>,
    }

    impl<S: sov_modules_api::Spec> ExampleModule<S> {
        /// Queries the state of the module.
        pub fn query_value(&self, state: &mut ApiStateAccessor<S>) -> Response {
            Response {
                value: self.value.get(state).unwrap_infallible(),
            }
        }
    }
}
