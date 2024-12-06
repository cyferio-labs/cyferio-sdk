use anyhow::Context;
use borsh::{BorshDeserialize, BorshSerialize};
use core::str::FromStr;
use serde::{Deserialize, Serialize};
use sov_rollup_interface::reexports::schemars::{self};
use sov_rollup_interface::sov_universal_wallet::UniversalWallet;
use std::hash::Hash;
use subxt::utils::{AccountId32, MultiAddress};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Hash,
    // derive_more::Display,
    BorshSerialize,
    BorshDeserialize,
    UniversalWallet,
)]
pub struct CyferioAddress(#[sov_wallet(as_ty = "CyferioAddressSchema")] [u8; 32]);

const CYFERIO: &str = "cyferio";
// Add schema definition for UniversalWallet
#[derive(sov_rollup_interface::sov_universal_wallet::UniversalWallet)]
#[allow(dead_code)]
#[doc(hidden)]
struct CyferioAddressSchema(#[sov_wallet(display(bech32(prefix = "CYFERIO")))] Vec<u8>);

impl sov_rollup_interface::BasicAddress for CyferioAddress {}

impl From<AccountId32> for CyferioAddress {
    fn from(account_id: AccountId32) -> Self {
        Self(*account_id.as_ref())
    }
}

impl From<CyferioAddress> for AccountId32 {
    fn from(address: CyferioAddress) -> Self {
        AccountId32::from(address.0)
    }
}

impl FromStr for CyferioAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let account_id =
            AccountId32::from_str(s).map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;
        Ok(Self::from(account_id))
    }
}

impl schemars::JsonSchema for CyferioAddress {
    fn schema_name() -> String {
        "CyferioAddress".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        serde_json::from_value(serde_json::json!({
            "type": "string",
            "pattern": "^cyferio[a-z0-9]+$",
            "description": "A Cyferio address",
        }))
        .expect("Invalid schema; this is a bug, please report it")
    }
}

impl AsRef<[u8]> for CyferioAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for CyferioAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 使用 bech32 格式显示地址
        write!(f, "{}", AccountId32::from(self.0))
    }
}

/// Decodes slice of bytes into CyferioAddress
/// Treats it as string if it starts with HRP and the rest is valid ASCII
/// Otherwise just decodes the bytes directly
impl TryFrom<&[u8]> for CyferioAddress {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.starts_with(b"cyferio") && bytes.is_ascii() {
            // safety: we checked that it is ASCII
            let s = unsafe { std::str::from_utf8_unchecked(bytes) };
            s.parse().context("failed parsing cyferio address")
        } else {
            bytes
                .try_into()
                .map(Self)
                .context("Invalid address length: expected 32 bytes")
        }
    }
}

impl From<MultiAddress<AccountId32, ()>> for CyferioAddress {
    fn from(multi_address: MultiAddress<AccountId32, ()>) -> Self {
        match multi_address {
            MultiAddress::Id(account_id) => Self::from(account_id),
            _ => panic!("Unsupported MultiAddress variant"),
        }
    }
}

impl From<[u8; 32]> for CyferioAddress {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl Default for CyferioAddress {
    fn default() -> Self {
        Self([0u8; 32])
    }
}
