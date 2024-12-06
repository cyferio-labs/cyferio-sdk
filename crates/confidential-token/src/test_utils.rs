use sov_modules_api::digest::Digest;
use sov_modules_api::{CryptoSpec, Spec};

use crate::{Bank, BankGasConfig, TokenId};

impl TokenId {
    /// Generates a deterministic token id by hashing the input string
    pub fn generate<S: Spec>(seed: &str) -> Self {
        let hash: [u8; 32] = <S::CryptoSpec as CryptoSpec>::Hasher::digest(seed.as_bytes()).into();
        hash.into()
    }
}

impl<S: Spec> Bank<S> {
    /// Returns the underlying gas config
    pub fn gas_config(&self) -> &BankGasConfig<S::Gas> {
        &self.gas
    }

    /// Overrides the underlying gas config
    pub fn override_gas_config(&mut self, gas: BankGasConfig<S::Gas>) {
        self.gas = gas;
    }
}
