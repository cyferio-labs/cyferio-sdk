//! This module implements the various "hooks" that are called by the STF during execution.
//! These hooks can be used to add custom logic at various points in the slot lifecycle:
//! - Before and after each transaction is executed.
//! - At the beginning and end of each batch ("blob")
//! - At the beginning and end of each slot (DA layer block)
use super::runtime::Runtime;
use sov_modules_api::hooks::KernelSlotHooks;
use sov_modules_api::hooks::{ApplyBatchHooks, FinalizeHook, SlotHooks, TxHooks};
use sov_modules_api::{BatchSequencerReceipt, Spec, StateCheckpoint, StateReaderAndWriter};
use sov_modules_api::{StateProvider, TxScratchpad};
use sov_rollup_interface::da::DaSpec;
use sov_state::namespaces::Accessory;
use sov_state::Storage;

impl<S: Spec> TxHooks for Runtime<S> {
    type Spec = S;
}

impl<S: Spec> ApplyBatchHooks for Runtime<S> {
    type Spec = S;
    type BatchResult = BatchSequencerReceipt<S>;

    fn begin_batch_hook<I: StateProvider<Self::Spec>>(
        &self,
        _sender: &<S::Da as DaSpec>::Address,
        _state: &mut TxScratchpad<S, I>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn end_batch_hook<I: StateProvider<Self::Spec>>(
        &self,
        _result: &Self::BatchResult,
        _state: &mut TxScratchpad<S, I>,
    ) {
    }
}

impl<S: Spec> SlotHooks for Runtime<S> {
    type Spec = S;
    fn begin_slot_hook(
        &self,
        _pre_state_root: &<<S as Spec>::Storage as Storage>::Root,
        _versioned_working_set: &mut StateCheckpoint<S::Storage>,
    ) {
    }

    fn end_slot_hook(&self, _state: &mut StateCheckpoint<S::Storage>) {}
}

impl<S: Spec> FinalizeHook for Runtime<S> {
    type Spec = S;

    fn finalize_hook(
        &self,
        _root_hash: &<<S as Spec>::Storage as Storage>::Root,
        _accessory_working_set: &mut impl StateReaderAndWriter<Accessory>,
    ) {
    }
}

impl<S: Spec> KernelSlotHooks for Runtime<S> {
    type Spec = S;
}
