//! Defines REST queries exposed by the bank module, along with the relevant types.

use axum::routing::get;
use jsonrpsee::core::RpcResult;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::prelude::utoipa::openapi::OpenApi;
use sov_modules_api::prelude::{axum, serde_yaml, UnwrapInfallible};
use sov_modules_api::rest::utils::{errors, ApiResult, Path, Query};
use sov_modules_api::rest::{ApiState, HasCustomRestApi};
use sov_modules_api::{ApiStateAccessor, Spec};

use crate::{get_token_id, Bank, EncryptedAmount, TokenId};

/// Response type for balance queries
#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub enum BalanceResponse {
    /// Balance in plaintext format
    Plaintext {
        /// The token balance amount
        amount: Option<u64>,
        /// The token identifier
        token_id: Option<TokenId>,
    },
    /// Balance in encrypted format
    Ciphertext {
        /// The encrypted token balance
        encrypted_amount: Option<EncryptedAmount>,
        /// The token identifier
        token_id: Option<TokenId>,
    },
}

/// Response type for total supply queries
#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub enum TotalSupplyResponse {
    /// Total supply in plaintext format
    Plaintext {
        /// The total supply amount
        amount: Option<u64>,
        /// The token identifier
        token_id: Option<TokenId>,
    },
    /// Total supply in encrypted format
    Ciphertext {
        /// The encrypted total supply
        encrypted_amount: Option<EncryptedAmount>,
        /// The token identifier
        token_id: Option<TokenId>,
    },
}

/// Response containing the FHE public key
#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct FhePublicKeyResponse {
    /// The FHE public key bytes
    pub public_key: Option<Vec<u8>>,
}

#[rpc_gen(client, server, namespace = "confidentialToken")]
impl<S: Spec> Bank<S> {
    #[rpc_method(name = "balanceOf")]
    /// Method that returns the balance of the user at the address `user_address` for the token
    /// stored at the address `token_id`.
    pub fn balance_of(
        &self,
        version: Option<u64>,
        user_address: S::Address,
        token_id: TokenId,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<BalanceResponse> {
        let amount = if let Some(v) = version {
            let state = &mut state
                .state_at_height(v)
                .map_err(|_| jsonrpsee::types::error::ErrorCode::MethodNotFound)?;
            self.get_balance_of(&user_address, token_id, state)
        } else {
            self.get_balance_of(&user_address, token_id, state)
        }
        .unwrap_infallible();
        Ok(BalanceResponse::Plaintext {
            amount: amount,
            token_id: Some(token_id),
        })
    }

    #[rpc_method(name = "rawBalanceOf")]
    /// Method that returns the raw balance of the user at the address `user_address` for the token
    /// stored at the address `token_id`.
    pub fn raw_balance_of(
        &self,
        version: Option<u64>,
        user_address: S::Address,
        token_id: TokenId,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<BalanceResponse> {
        let amount = if let Some(v) = version {
            let state = &mut state
                .state_at_height(v)
                .map_err(|_| jsonrpsee::types::error::ErrorCode::MethodNotFound)?;
            self.get_raw_balance_of(&user_address, token_id, state)
        } else {
            self.get_raw_balance_of(&user_address, token_id, state)
        }
        .unwrap_infallible();
        Ok(BalanceResponse::Ciphertext {
            encrypted_amount: amount,
            token_id: Some(token_id),
        })
    }

    #[rpc_method(name = "supplyOf")]
    /// Method that returns the supply of a token stored at the address `token_id`.
    pub fn supply_of(
        &self,
        version: Option<u64>,
        token_id: TokenId,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<TotalSupplyResponse> {
        let amount = if let Some(v) = version {
            let state = &mut state
                .state_at_height(v)
                .map_err(|_| jsonrpsee::types::error::ErrorCode::MethodNotFound)?;
            self.get_total_supply_of(&token_id, state)
        } else {
            self.get_total_supply_of(&token_id, state)
        }
        .unwrap_infallible();
        Ok(TotalSupplyResponse::Plaintext {
            amount: amount,
            token_id: Some(token_id),
        })
    }

    #[rpc_method(name = "rawSupplyOf")]
    /// Method that returns the raw supply of a token stored at the address `token_id`.
    pub fn raw_supply_of(
        &self,
        version: Option<u64>,
        token_id: TokenId,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<TotalSupplyResponse> {
        let amount = if let Some(v) = version {
            let state = &mut state
                .state_at_height(v)
                .map_err(|_| jsonrpsee::types::error::ErrorCode::MethodNotFound)?;
            self.get_raw_total_supply_of(&token_id, state)
        } else {
            self.get_raw_total_supply_of(&token_id, state)
        }
        .unwrap_infallible();
        Ok(TotalSupplyResponse::Ciphertext {
            encrypted_amount: amount,
            token_id: Some(token_id),
        })
    }

    #[rpc_method(name = "FhePublicKey")]
    /// Method that returns the FHE public key of the bank.
    pub fn fhe_public_key(
        &self,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<FhePublicKeyResponse> {
        let public_key = self.get_fhe_public_key(state).unwrap();
        Ok(FhePublicKeyResponse { public_key })
    }
}

/// Axum routes.
impl<S: Spec> Bank<S> {
    async fn route_balance(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path((token_id, user_address)): Path<(TokenId, S::Address)>,
    ) -> ApiResult<BalanceResponse> {
        let amount = state
            .get_balance_of(&user_address, token_id, &mut accessor)
            .unwrap_infallible()
            .ok_or_else(|| errors::not_found_404("Balance", user_address))?;

        Ok(BalanceResponse::Plaintext {
            amount: Some(amount),
            token_id: Some(token_id),
        }
        .into())
    }

    async fn route_raw_balance(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path((token_id, user_address)): Path<(TokenId, S::Address)>,
    ) -> ApiResult<BalanceResponse> {
        let amount = state
            .get_raw_balance_of(&user_address, token_id, &mut accessor)
            .unwrap_infallible()
            .ok_or_else(|| errors::not_found_404("Balance", user_address))?;

        Ok(BalanceResponse::Ciphertext {
            encrypted_amount: Some(amount),
            token_id: Some(token_id),
        }
        .into())
    }

    async fn route_total_supply(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path(token_id): Path<TokenId>,
    ) -> ApiResult<TotalSupplyResponse> {
        let amount = state
            .get_total_supply_of(&token_id, &mut accessor)
            .unwrap_infallible()
            .ok_or_else(|| errors::not_found_404("Token", token_id))?;

        Ok(TotalSupplyResponse::Plaintext {
            amount: Some(amount),
            token_id: Some(token_id),
        }
        .into())
    }

    async fn route_raw_total_supply(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path(token_id): Path<TokenId>,
    ) -> ApiResult<TotalSupplyResponse> {
        let amount = state
            .get_raw_total_supply_of(&token_id, &mut accessor)
            .unwrap_infallible()
            .ok_or_else(|| errors::not_found_404("Token", token_id))?;

        Ok(TotalSupplyResponse::Ciphertext {
            encrypted_amount: Some(amount),
            token_id: Some(token_id),
        }
        .into())
    }

    async fn route_fhe_public_key(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
    ) -> ApiResult<FhePublicKeyResponse> {
        let public_key = state.get_fhe_public_key(&mut accessor).unwrap();
        Ok(FhePublicKeyResponse { public_key }.into())
    }

    async fn route_find_token_id(
        params: Query<types::FindTokenIdQueryParams<S::Address>>,
    ) -> ApiResult<types::TokenIdResponse> {
        let token_id = get_token_id::<S>(&params.token_name, &params.sender);
        Ok(types::TokenIdResponse { token_id }.into())
    }

    async fn route_authorized_minters(
        state: ApiState<S, Self>,
        mut accessor: ApiStateAccessor<S>,
        Path(token_id): Path<TokenId>,
    ) -> ApiResult<types::AuthorizedMintersResponse<S>> {
        let authorized_minters = state
            .tokens
            .get(&token_id, &mut accessor)
            .unwrap_infallible()
            .ok_or_else(|| errors::not_found_404("Token", token_id))?
            .authorized_minters;
        Ok(types::AuthorizedMintersResponse { authorized_minters }.into())
    }
}

impl<S: Spec> HasCustomRestApi for Bank<S> {
    type Spec = S;

    fn custom_rest_api(&self, state: ApiState<S>) -> axum::Router<()> {
        axum::Router::new()
            .route(
                "/tokens/:tokenId/balances/:address",
                get(Self::route_balance),
            )
            .route(
                "/tokens/:tokenId/raw-balances/:address",
                get(Self::route_raw_balance),
            )
            .route(
                "/tokens/:tokenId/total-supply",
                get(Self::route_total_supply),
            )
            .route(
                "/tokens/:tokenId/raw-total-supply",
                get(Self::route_raw_total_supply),
            )
            .route("/fhe-public-key", get(Self::route_fhe_public_key))
            .route(
                "/tokens/:tokenId/authorized-minters",
                get(Self::route_authorized_minters),
            )
            .route("/tokens", get(Self::route_find_token_id))
            .with_state(state.with(self.clone()))
    }

    fn custom_openapi_spec(&self) -> Option<OpenApi> {
        let mut open_api: OpenApi =
            serde_yaml::from_str(include_str!("../openapi-v3.yaml")).expect("Invalid OpenAPI spec");
        // Because https://github.com/juhaku/utoipa/issues/972
        for path_item in open_api.paths.paths.values_mut() {
            path_item.extensions = None;
        }
        Some(open_api)
    }
}

#[allow(missing_docs)]
pub mod types {
    use super::*;
    use crate::utils::TokenHolder;

    #[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct FindTokenIdQueryParams<Addr> {
        pub token_name: String,
        pub sender: Addr,
    }

    #[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    pub struct TokenIdResponse {
        pub token_id: TokenId,
    }

    #[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
    #[serde(bound = "S::Address: serde::Serialize + serde::de::DeserializeOwned")]
    pub struct AuthorizedMintersResponse<S: sov_modules_api::Spec> {
        pub authorized_minters: Vec<TokenHolder<S>>,
    }
}
