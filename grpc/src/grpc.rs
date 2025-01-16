//! Types and client for the celestia grpc

use std::fmt;

use bytes::Bytes;
use celestia_grpc_macros::grpc_method;
use celestia_proto::celestia::blob::v1::query_client::QueryClient as BlobQueryClient;
use celestia_proto::celestia::core::v1::tx::tx_client::TxClient as TxStatusClient;
use celestia_proto::cosmos::auth::v1beta1::query_client::QueryClient as AuthQueryClient;
use celestia_proto::cosmos::bank::v1beta1::query_client::QueryClient as BankQueryClient;
use celestia_proto::cosmos::base::abci::v1beta1::GasInfo;
use celestia_proto::cosmos::base::node::v1beta1::service_client::ServiceClient as ConfigServiceClient;
use celestia_proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient as TendermintServiceClient;
use celestia_proto::cosmos::tx::v1beta1::service_client::ServiceClient as TxServiceClient;
use celestia_types::blob::BlobParams;
use celestia_types::block::Block;
use celestia_types::hash::Hash;
use celestia_types::state::auth::AuthParams;
use celestia_types::state::{Address, Coin, TxResponse};
use http_body::Body;
use tonic::body::BoxBody;
use tonic::client::GrpcService;

use crate::Result;

// cosmos.auth
mod auth;
// cosmos.bank
mod bank;
// cosmos.base.node
mod node;
// cosmos.base.tendermint
mod tendermint;
// celestia.core.tx
mod celestia_tx;
// celestia.blob
mod blob;
// cosmos.tx
mod cosmos_tx;

pub use crate::grpc::auth::Account;
pub use crate::grpc::celestia_tx::{TxStatus, TxStatusResponse};
pub use crate::grpc::cosmos_tx::{BroadcastMode, GetTxResponse};

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
pub use crate::grpc::auth::{JsAuthParams, JsBaseAccount, JsPublicKey};
#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
pub use crate::grpc::bank::JsCoin;

/// Error convertible to std, used by grpc transports
pub type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Struct wrapping all the tonic types and doing type conversion behind the scenes.
pub struct GrpcClient<T> {
    transport: T,
}

impl<T> GrpcClient<T> {
    /// Get the underlying transport.
    pub fn into_inner(self) -> T {
        self.transport
    }
}

impl<T> GrpcClient<T>
where
    T: GrpcService<BoxBody> + Clone,
    T::Error: Into<StdError>,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    /// Create a new client wrapping given transport
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    // cosmos.auth

    /// Get auth params
    #[grpc_method(AuthQueryClient::params)]
    async fn get_auth_params(&self) -> Result<AuthParams>;

    /// Get account
    #[grpc_method(AuthQueryClient::account)]
    async fn get_account(&self, account: &Address) -> Result<Account>;

    /// Get accounts
    #[grpc_method(AuthQueryClient::accounts)]
    async fn get_accounts(&self) -> Result<Vec<Account>>;

    // cosmos.bank

    /// Get balance of coins with given denom
    #[grpc_method(BankQueryClient::balance)]
    async fn get_balance(&self, address: &Address, denom: impl Into<String>) -> Result<Coin>;

    /// Get balance of all coins
    #[grpc_method(BankQueryClient::all_balances)]
    async fn get_all_balances(&self, address: &Address) -> Result<Vec<Coin>>;

    /// Get balance of all spendable coins
    #[grpc_method(BankQueryClient::spendable_balances)]
    async fn get_spendable_balances(&self, address: &Address) -> Result<Vec<Coin>>;

    /// Get total supply
    #[grpc_method(BankQueryClient::total_supply)]
    async fn get_total_supply(&self) -> Result<Vec<Coin>>;

    // cosmos.base.node

    /// Get Minimum Gas price
    #[grpc_method(ConfigServiceClient::config)]
    async fn get_min_gas_price(&self) -> Result<f64>;

    // cosmos.base.tendermint

    /// Get latest block
    #[grpc_method(TendermintServiceClient::get_latest_block)]
    async fn get_latest_block(&self) -> Result<Block>;

    /// Get block by height
    #[grpc_method(TendermintServiceClient::get_block_by_height)]
    async fn get_block_by_height(&self, height: i64) -> Result<Block>;

    // cosmos.tx

    /// Broadcast prepared and serialised transaction
    #[grpc_method(TxServiceClient::broadcast_tx)]
    async fn broadcast_tx(&self, tx_bytes: Vec<u8>, mode: BroadcastMode) -> Result<TxResponse>;

    /// Get Tx
    #[grpc_method(TxServiceClient::get_tx)]
    async fn get_tx(&self, hash: Hash) -> Result<GetTxResponse>;

    /// Broadcast prepared and serialised transaction
    #[grpc_method(TxServiceClient::simulate)]
    async fn simulate(&self, tx_bytes: Vec<u8>) -> Result<GasInfo>;

    // celestia.blob

    /// Get blob params
    #[grpc_method(BlobQueryClient::params)]
    async fn get_blob_params(&self) -> Result<BlobParams>;

    // celestia.core.tx

    /// Get status of the transaction
    #[grpc_method(TxStatusClient::tx_status)]
    async fn tx_status(&self, hash: Hash) -> Result<TxStatusResponse>;
}

#[cfg(not(target_arch = "wasm32"))]
impl GrpcClient<tonic::transport::Channel> {
    /// Create a new client connected to the given `url` with default
    /// settings of [`tonic::transport::Channel`].
    pub fn with_url(url: impl Into<String>) -> Result<Self, tonic::transport::Error> {
        let channel = tonic::transport::Endpoint::from_shared(url.into())?.connect_lazy();
        Ok(Self { transport: channel })
    }
}

#[cfg(target_arch = "wasm32")]
impl GrpcClient<tonic_web_wasm_client::Client> {
    /// Create a new client connected to the given `url` with default
    /// settings of [`tonic_web_wasm_client::Client`].
    pub fn with_grpcweb_url(url: impl Into<String>) -> Self {
        Self {
            transport: tonic_web_wasm_client::Client::new(url.into()),
        }
    }
}

impl<T> fmt::Debug for GrpcClient<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrpcClient { .. }")
    }
}

pub(crate) trait FromGrpcResponse<T> {
    fn try_from_response(self) -> Result<T>;
}

pub(crate) trait IntoGrpcParam<T> {
    fn into_parameter(self) -> T;
}

macro_rules! make_empty_params {
    ($request_type:ident) => {
        impl crate::grpc::IntoGrpcParam<$request_type> for () {
            fn into_parameter(self) -> $request_type {
                $request_type {}
            }
        }
    };
}

pub(crate) use make_empty_params;
