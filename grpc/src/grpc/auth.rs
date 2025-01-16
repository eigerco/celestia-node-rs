use celestia_proto::cosmos::auth::v1beta1::{
    QueryAccountRequest, QueryAccountResponse, QueryAccountsRequest, QueryAccountsResponse,
    QueryParamsRequest as QueryAuthParamsRequest, QueryParamsResponse as QueryAuthParamsResponse,
};
use celestia_types::state::auth::{
    AuthParams, BaseAccount, ModuleAccount, RawBaseAccount, RawModuleAccount,
};
use celestia_types::state::Address;
use prost::{Message, Name};
use tendermint_proto::google::protobuf::Any;

use crate::grpc::{make_empty_params, FromGrpcResponse, IntoGrpcParam};
use crate::{Error, Result};

// TODO: move this stuff to types similarly to address
//       + add vesting accounts
/// Enum representing different types of account
#[derive(Debug, PartialEq)]
pub enum Account {
    /// Base account type
    Base(BaseAccount),
    /// Account for modules that holds coins on a pool
    Module(ModuleAccount),
}

impl std::ops::Deref for Account {
    type Target = BaseAccount;

    fn deref(&self) -> &Self::Target {
        match self {
            Account::Base(base) => base,
            Account::Module(module) => &module.base_account,
        }
    }
}

impl std::ops::DerefMut for Account {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Account::Base(base) => base,
            Account::Module(module) => &mut module.base_account,
        }
    }
}

impl FromGrpcResponse<AuthParams> for QueryAuthParamsResponse {
    fn try_from_response(self) -> Result<AuthParams> {
        let params = self.params.ok_or(Error::FailedToParseResponse)?;
        Ok(AuthParams {
            max_memo_characters: params.max_memo_characters,
            tx_sig_limit: params.tx_sig_limit,
            tx_size_cost_per_byte: params.tx_size_cost_per_byte,
            sig_verify_cost_ed25519: params.sig_verify_cost_ed25519,
            sig_verify_cost_secp256k1: params.sig_verify_cost_secp256k1,
        })
    }
}

impl FromGrpcResponse<Account> for QueryAccountResponse {
    fn try_from_response(self) -> Result<Account> {
        account_from_any(self.account.ok_or(Error::FailedToParseResponse)?)
    }
}

impl FromGrpcResponse<Vec<Account>> for QueryAccountsResponse {
    fn try_from_response(self) -> Result<Vec<Account>> {
        self.accounts.into_iter().map(account_from_any).collect()
    }
}

make_empty_params!(QueryAuthParamsRequest);

impl IntoGrpcParam<QueryAccountRequest> for &Address {
    fn into_parameter(self) -> QueryAccountRequest {
        QueryAccountRequest {
            address: self.to_string(),
        }
    }
}

impl IntoGrpcParam<QueryAccountsRequest> for () {
    fn into_parameter(self) -> QueryAccountsRequest {
        QueryAccountsRequest { pagination: None }
    }
}

fn account_from_any(any: Any) -> Result<Account> {
    let account = if any.type_url == RawBaseAccount::type_url() {
        let base_account =
            RawBaseAccount::decode(&*any.value).map_err(|_| Error::FailedToParseResponse)?;
        Account::Base(base_account.try_into()?)
    } else if any.type_url == RawModuleAccount::type_url() {
        let module_account =
            RawModuleAccount::decode(&*any.value).map_err(|_| Error::FailedToParseResponse)?;
        Account::Module(module_account.try_into()?)
    } else {
        return Err(Error::UnexpectedResponseType(any.type_url));
    };

    Ok(account)
}

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
pub use wbg::*;

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
mod wbg {
    use celestia_types::state::auth::AuthParams;
    use js_sys::{BigInt, Uint8Array};
    use tendermint::PublicKey;
    use wasm_bindgen::{prelude::*, JsCast};

    use crate::utils::make_object;

    use super::Account;

    #[wasm_bindgen(typescript_custom_section)]
    const _: &str = r#"
    /**
     * Public key
     */
    export interface PublicKey {
      type: "ed25519" | "secp256k1",
      value: Uint8Array
    }

    /**
     * Common data of all account types
     */
    export interface BaseAccount {
      address: string,
      pubkey?: PublicKey,
      accountNumber: bigint,
      sequence: bigint
    }

    /**
     * Auth module parameters
     */
    export interface AuthParams {
      maxMemoCharacters: bigint,
      txSigLimit: bigint,
      txSizeCostPerByte: bigint,
      sigVerifyCostEd25519: bigint,
      sigVerifyCostSecp256k1: bigint
    }
    "#;

    #[wasm_bindgen]
    extern "C" {
        /// Public key exposed to javascript.
        #[wasm_bindgen(typescript_type = "PublicKey")]
        pub type JsPublicKey;

        /// BaseAccount exposed to javascript.
        #[wasm_bindgen(typescript_type = "BaseAccount")]
        pub type JsBaseAccount;

        /// AuthParams exposed to javascript.
        #[wasm_bindgen(typescript_type = "AuthParams")]
        pub type JsAuthParams;
    }

    impl From<PublicKey> for JsPublicKey {
        fn from(value: PublicKey) -> JsPublicKey {
            let algo = match value {
                PublicKey::Ed25519(..) => "ed25519",
                PublicKey::Secp256k1(..) => "secp256k1",
                _ => unreachable!("unsupported pubkey algo found"),
            };
            let obj = make_object!(
                "type" => algo.into(),
                "value" => Uint8Array::from(value.to_bytes().as_ref())
            );

            obj.unchecked_into()
        }
    }

    impl From<Account> for JsBaseAccount {
        fn from(value: Account) -> JsBaseAccount {
            let obj = make_object!(
                "address" => value.address.to_string().into(),
                "pubkey" => value.pub_key.map(JsPublicKey::from).into(),
                "accountNumber" => BigInt::from(value.account_number),
                "sequence" => BigInt::from(value.sequence)
            );

            obj.unchecked_into()
        }
    }

    impl From<AuthParams> for JsAuthParams {
        fn from(value: AuthParams) -> JsAuthParams {
            let obj = make_object!(
                "maxMemoCharacters" => BigInt::from(value.max_memo_characters),
                "txSigLimit" => BigInt::from(value.tx_sig_limit),
                "txSizeCostPerByte" => BigInt::from(value.tx_size_cost_per_byte),
                "sigVerifyCostEd25519" => BigInt::from(value.sig_verify_cost_ed25519),
                "sigVerifyCostSecp256k1" => BigInt::from(value.sig_verify_cost_secp256k1)
            );

            obj.unchecked_into()
        }
    }
}
