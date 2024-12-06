use anyhow::{bail, Context as _, Result};
use schemars::JsonSchema;
use sov_modules_api::macros::UniversalWallet;
use sov_modules_api::{
    Context, EventEmitter, SafeString, SafeVec, Spec, StateAccessor, StateReader, TxState,
};
use sov_state::User;
use strum::{EnumDiscriminants, EnumIs, VariantArray};

use crate::event::Event;
use crate::utils::{Payable, TokenHolderRef};
use crate::{Bank, Coins, EncryptedAmount, Token, TokenId};

// FHE deps
use crate::mock_decryption;
use bincode;
use tfhe::{prelude::*, set_server_key, CompressedPublicKey, CompressedServerKey};

/// The maximum number of addresses that can be authorized to mint a token.
pub const MAX_AUTHORIZED_MINTERS: usize = 20;

/// This enumeration represents the available call messages for interacting with the sov-bank module.
#[derive(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Eq,
    Clone,
    JsonSchema,
    EnumDiscriminants,
    EnumIs,
    UniversalWallet,
)]
#[schemars(bound = "S::Address: ::schemars::JsonSchema", rename = "CallMessage")]
#[serde(rename_all = "snake_case")]
#[strum_discriminants(derive(VariantArray, EnumIs))]
pub enum CallMessage<S: Spec> {
    /// Creates a new token with the specified name and initial balance.
    CreateToken {
        /// The name of the new token.
        token_name: SafeString,
        /// The initial balance of the new token.
        initial_balance: EncryptedAmount,
        /// The address of the account that the new tokens are minted to.
        mint_to_address: S::Address,
        /// Authorized minter list.
        authorized_minters: SafeVec<S::Address, MAX_AUTHORIZED_MINTERS>,
    },

    /// Transfers a specified amount of tokens to the specified address.
    #[sov_wallet(show_as = "Transfer to address {} {}.")]
    Transfer {
        /// The address to which the tokens will be transferred.
        to: S::Address,
        /// The amount of tokens to transfer.
        coins: Coins,
    },

    /// Mints a specified amount of tokens.
    Mint {
        /// The amount of tokens to mint.
        coins: Coins,
        /// Address to mint tokens to
        mint_to_address: S::Address,
    },

    /// Freezes a token so that the supply is frozen
    Freeze {
        /// Address of the token to be frozen
        token_id: TokenId,
    },
}

impl<S: Spec> Bank<S> {
    /// Creates a token from a set of configuration parameters.
    /// Checks if a token already exists at that address. If so return an error.
    #[allow(clippy::too_many_arguments)]
    pub fn create_token(
        &self,
        token_name: String,
        initial_balance: EncryptedAmount,
        minter: impl Payable<S>,
        authorized_minters: Vec<impl Payable<S>>,
        originator: impl Payable<S>,
        state: &mut impl TxState<S>,
    ) -> Result<TokenId> {
        tracing::info!(%token_name, %minter, sender= %originator, "Create token request");

        let authorized_minters = authorized_minters
            .iter()
            .map(|minter| minter.as_token_holder())
            .collect::<Vec<_>>();

        // Fetch the fhe keys from state and set the server key in the environment
        let fhe_public_key =
            bincode::deserialize::<CompressedPublicKey>(&self.fhe_public_key.get(state)?.unwrap())?
                .decompress();
        let fhe_server_key =
            bincode::deserialize::<CompressedServerKey>(&self.fhe_server_key.get(state)?.unwrap())?
                .decompress();
        set_server_key(fhe_server_key);

        let (token_id, token) = Token::<S>::create(
            &token_name,
            &[(minter.as_token_holder(), initial_balance.clone())],
            &authorized_minters,
            originator,
            self.tokens.prefix(),
            &fhe_public_key,
            state,
        )?;

        if self.tokens.get(&token_id, state)?.is_some() {
            bail!(
                "Token {} at {} address already exists",
                token_name,
                token_id
            );
        }

        self.tokens.set(&token_id, &token, state)?;
        self.emit_event(
            state,
            Event::TokenCreated {
                token_name: token_name.clone(),
                coins: Coins {
                    amount: initial_balance,
                    token_id,
                },
                minter: minter.as_token_holder().into(),
                authorized_minters: authorized_minters.iter().map(|m| m.into()).collect(),
            },
        );
        tracing::info!(%token_name, %token_id, "Token created");
        Ok(token_id)
    }

    /// Transfers the set of `coins` to the address specified by `to`.
    pub fn transfer(
        &self,
        to: impl Payable<S>,
        coins: Coins,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        let to = to.as_token_holder();
        let sender = context.sender();
        self.transfer_from(sender, to, coins.clone(), state)?;
        self.emit_event(
            state,
            Event::TokenTransferred {
                from: sender.as_token_holder().into(),
                to: to.into(),
                coins,
            },
        );
        Ok(())
    }

    /// Mints the `coins`to the address `mint_to_identity` using the externally owned account ("EOA") supplied by
    /// `context.sender()` as the authorizer.
    /// Returns an error if the token ID doesn't exist or `context.sender()` is not authorized to mint tokens.
    ///
    /// On success, it updates the `self.tokens` set to store the new balance.
    pub fn mint_from_eoa(
        &self,
        coins: &Coins,
        mint_to_identity: impl Payable<S>,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        self.mint(
            coins,
            mint_to_identity,
            TokenHolderRef::from(&context.sender()),
            state,
        )
    }

    /// Mints the `coins` to the  `mint_to_identity` if `authorizer` is an allowed minter.
    /// Returns an error if the token ID doesn't exist or `context.sender()` is not authorized to mint tokens.
    ///
    /// On success, it updates the `self.tokens` set to store the new minted address.
    pub fn mint(
        &self,
        coins: &Coins,
        mint_to_identity: impl Payable<S>,
        authorizer: impl Payable<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        let mint_to_identity = mint_to_identity.as_token_holder();
        let context_logger = || {
            format!(
                "Failed mint coins({}) to {} by authorizer {}",
                coins, mint_to_identity, authorizer
            )
        };
        let mut token = self
            .tokens
            .get_or_err(&coins.token_id, state)
            .with_context(context_logger)??;

        // Fetch the fhe keys from state and set the server key in the environment
        let fhe_public_key =
            bincode::deserialize::<CompressedPublicKey>(&self.fhe_public_key.get(state)?.unwrap())?
                .decompress();
        let fhe_server_key = bincode::deserialize::<CompressedServerKey>(
            &self.fhe_server_key.get(state)?.unwrap().as_ref(),
        )?
        .decompress();
        set_server_key(fhe_server_key);

        let authorizer = authorizer.as_token_holder();
        token
            .mint(
                authorizer,
                mint_to_identity,
                &coins.amount,
                &fhe_public_key,
                state,
            )
            .with_context(context_logger)?;
        self.tokens.set(&coins.token_id, &token, state)?;
        self.emit_event(
            state,
            Event::TokenMinted {
                mint_to_identity: mint_to_identity.into(),
                coins: coins.clone(),
            },
        );

        Ok(())
    }

    /// Tries to freeze the token ID `token_id`.
    /// Returns an error if the token ID doesn't exist,
    /// otherwise calls the [`Token::freeze`] function, and update the token set upon success.
    pub(crate) fn freeze(
        &self,
        token_id: TokenId,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        let context_logger = || {
            format!(
                "Failed freeze token_id={} by sender {}",
                token_id,
                context.sender()
            )
        };

        let mut token = self
            .tokens
            .get_or_err(&token_id, state)
            .with_context(context_logger)??;

        let sender_ref = context.sender();
        let sender = sender_ref.as_token_holder();
        token.freeze(sender).with_context(context_logger)?;

        self.tokens.set(&token_id, &token, state)?;
        self.emit_event(
            state,
            Event::TokenFrozen {
                freezer: sender.into(),
                token_id,
            },
        );

        Ok(())
    }
}

impl<S: Spec> Bank<S> {
    /// Transfers the set of `coins` from the address `from` to the address `to`.
    ///
    /// Returns an error if the token ID doesn't exist.
    pub fn transfer_from(
        &self,
        from: impl Payable<S>,
        to: impl Payable<S>,
        coins: Coins,
        state: &mut impl StateAccessor,
    ) -> Result<()> {
        let from = from.as_token_holder();
        let to = to.as_token_holder();
        let context_logger = || {
            format!(
                "Failed transfer from={} to={} of coins({})",
                &from, &to, coins
            )
        };

        // Fetch the fhe keys from state and set the server key in the environment
        let fhe_public_key =
            bincode::deserialize::<CompressedPublicKey>(&self.fhe_public_key.get(state)?.unwrap())?
                .decompress();
        let fhe_server_key =
            bincode::deserialize::<CompressedServerKey>(&self.fhe_server_key.get(state)?.unwrap())?
                .decompress();
        set_server_key(fhe_server_key);

        let token = self
            .tokens
            .get_or_err(&coins.token_id, state)
            .map(|token| token.with_context(context_logger))
            .with_context(context_logger)??;
        token
            .transfer(from, to, &coins.amount, &fhe_public_key, state)
            .with_context(context_logger)?;

        Ok(())
    }

    /// Helper function to return the balance of the token stored at `token_id`
    /// for the user having the address `user_address` from the underlying storage. If the token ID doesn't exist, or
    /// if the user doesn't have tokens of that type, return `None`. Otherwise, wrap the resulting balance in `Some`.
    pub fn get_balance_of<Accessor: StateAccessor>(
        &self,
        user_address: impl Payable<S>,
        token_id: TokenId,
        state: &mut Accessor,
    ) -> Result<Option<u64>, <Accessor as StateReader<User>>::Error> {
        let user_address = user_address.as_token_holder();

        // Fetch the balance from state, return None if the token doesn't exist
        let raw_balance = match self.tokens.get(&token_id, state)? {
            Some(token) => match token.balances.get(&user_address, state)? {
                Some(balance) => balance,
                None => return Ok(Some(0)), // if token exist and user has no balance, return 0
            },
            None => return Ok(None), // if token doesn't exist, return None
        };

        // Fetch the fhe keys from state and set the server key in the environment
        let fhe_server_key =
            bincode::deserialize::<CompressedServerKey>(&self.fhe_server_key.get(state)?.unwrap())
                .unwrap()
                .decompress();
        set_server_key(fhe_server_key);

        // decrypt the balance from FheUint64 to u64
        let balance = mock_decryption::decrypt(&raw_balance);
        tracing::info!("Decrypted balance of the user: {}", balance);
        Ok(Some(balance))
    }

    /// Get encrypted balance in FheUint64 of the user at the address `user_address` for the token stored at the address `token_id`.
    pub fn get_raw_balance_of<Accessor: StateAccessor>(
        &self,
        user_address: impl Payable<S>,
        token_id: TokenId,
        state: &mut Accessor,
    ) -> Result<Option<EncryptedAmount>, <Accessor as StateReader<User>>::Error> {
        let user_address = user_address.as_token_holder();
        self.tokens
            .get(&token_id, state)?
            .and_then(|token| token.balances.get(&user_address, state).transpose())
            .transpose()
    }

    /// Get the name of a token by ID
    pub fn get_token_name<Accessor: StateReader<User>>(
        &self,
        token_id: &TokenId,
        state: &mut Accessor,
    ) -> Result<Option<String>, Accessor::Error> {
        let token = self.tokens.get(token_id, state)?;
        Ok(token.map(|token| token.name))
    }

    /// Returns the total supply of the token with the given `token_id`.
    pub fn get_total_supply_of<Accessor: StateAccessor>(
        &self,
        token_id: &TokenId,
        state: &mut Accessor,
    ) -> Result<Option<u64>, <Accessor as StateReader<User>>::Error> {
        let raw_total_supply = self
            .tokens
            .get(token_id, state)?
            .map(|token| token.total_supply)
            .unwrap();

        // Fetch the fhe keys from state and set the server key in the environment
        let fhe_server_key =
            bincode::deserialize::<CompressedServerKey>(&self.fhe_server_key.get(state)?.unwrap())
                .unwrap()
                .decompress();
        set_server_key(fhe_server_key);

        // decrypt the total supply from FheUint64 to u64
        let total_supply = mock_decryption::decrypt(&raw_total_supply);
        tracing::info!("Decrypted total supply: {}", total_supply);
        Ok(Some(total_supply))
    }

    /// Returns the total supply of the token with the given `token_id`.
    pub fn get_raw_total_supply_of<Accessor: StateAccessor>(
        &self,
        token_id: &TokenId,
        state: &mut Accessor,
    ) -> Result<Option<EncryptedAmount>, <Accessor as StateReader<User>>::Error> {
        Ok(self
            .tokens
            .get(token_id, state)?
            .map(|token| token.total_supply))
    }

    /// Returns the FHE public key of the bank
    pub fn get_fhe_public_key<Accessor: StateReader<User>>(
        &self,
        state: &mut Accessor,
    ) -> Result<Option<Vec<u8>>, <Accessor as StateReader<User>>::Error> {
        self.fhe_public_key.get(state)
    }
}

/// Creates a new prefix from an already existing prefix `parent_prefix` and a `token_id`
/// by extending the parent prefix.
pub(crate) fn prefix_from_address_with_parent(
    parent_prefix: &sov_state::Prefix,
    token_id: &TokenId,
) -> sov_state::Prefix {
    let mut prefix = parent_prefix.as_ref().to_vec();
    prefix.extend_from_slice(format!("{}", token_id).as_bytes());
    sov_state::Prefix::new(prefix)
}
