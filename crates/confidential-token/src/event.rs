use sov_modules_api::Spec;

use crate::utils::TokenHolder;
use crate::{Coins, TokenId};

/// Bank Event
#[derive(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Clone,
)]
#[serde(
    bound = "S::Address: serde::Serialize + serde::de::DeserializeOwned",
    rename_all = "snake_case"
)]
pub enum Event<S: Spec> {
    /// Event for Token Creation
    TokenCreated {
        /// The name of the new token.
        token_name: String,
        /// The new tokens that were minted.
        coins: Coins,
        /// The token holder that the new tokens are minted to.
        minter: TokenHolder<S>,
        /// Authorized minter list.
        authorized_minters: Vec<TokenHolder<S>>,
    },
    /// Event for Token Transfer
    TokenTransferred {
        /// The identity that is transferring the tokens.
        from: TokenHolder<S>,
        /// The token holder that the tokens were transferred to.
        to: TokenHolder<S>,
        /// The tokens transferred.
        coins: Coins,
    },
    /// The supply of a token was frozen
    TokenFrozen {
        /// The token holder that froze the tokens
        freezer: TokenHolder<S>,
        /// The ID of the token that was transferred
        token_id: TokenId,
    },
    /// Event for Token Minting
    TokenMinted {
        /// The identity to mint the tokens to
        mint_to_identity: TokenHolder<S>,
        /// The coins minted
        coins: Coins,
    },
}
