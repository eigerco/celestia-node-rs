use celestia_types::{hash::Hash, state::ErrorCode};
use k256::ecdsa::signature::Error as SignatureError;
use tonic::Status;

/// Alias for a `Result` with the error type [`celestia_tonic::Error`].
///
/// [`celestia_tonic::Error`]: crate::Error
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Representation of all the errors that can occur when interacting with [`celestia_tonic`].
///
/// [`celestia_tonic`]: crate
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Tonic error
    #[error(transparent)]
    TonicError(#[from] Status),

    /// Transport error
    #[error("Transport: {0}")]
    TransportError(String),

    /// Tendermint Error
    #[error(transparent)]
    TendermintError(#[from] tendermint::Error),

    /// Celestia types error
    #[error(transparent)]
    CelestiaTypesError(#[from] celestia_types::Error),

    /// Tendermint Proto Error
    #[error(transparent)]
    TendermintProtoError(#[from] tendermint_proto::Error),

    /// Failed to parse gRPC response
    #[error("Failed to parse response")]
    FailedToParseResponse,

    /// Unexpected reponse type
    #[error("Unexpected response type")]
    UnexpectedResponseType(String),

    /// Empty blob submission list
    #[error("Attempted to submit blob transaction with empty blob list")]
    TxEmptyBlobList,

    /// Broadcasting transaction failed
    #[error("Broadcasting transaction {0} failed; code: {1}, error: {2}")]
    TxBroadcastFailed(Hash, ErrorCode, String),

    /// Executing transaction failed
    #[error("Transaction {0} execution failed; code: {1}, error: {2}")]
    TxExecutionFailed(Hash, ErrorCode, String),

    /// Transaction was evicted from the mempool
    #[error("Transaction {0} was evicted from the mempool")]
    TxEvicted(Hash),

    /// Transaction wasn't found, it was likely rejected
    #[error("Transaction {0} wasn't found, it was likely rejected")]
    TxNotFound(Hash),

    /// Provided public key differs from one associated with account
    #[error("Provided public key differs from one associated with account")]
    PublicKeyMismatch,

    /// Signing error
    #[error(transparent)]
    SigningError(#[from] SignatureError),
}

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
impl From<Error> for wasm_bindgen::JsValue {
    fn from(error: Error) -> wasm_bindgen::JsValue {
        error.to_string().into()
    }
}
