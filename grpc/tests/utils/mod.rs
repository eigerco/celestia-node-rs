use celestia_types::state::{AccAddress, Address};
use tendermint::crypto::default::ecdsa_secp256k1::SigningKey;
use tendermint::public_key::Secp256k1 as VerifyingKey;

pub use imp::*;

/// [`TestAccount`] stores celestia account credentials and information, for cases where we don't
/// mind jusk keeping the plaintext secret key in memory
#[derive(Debug, Clone)]
pub struct TestAccount {
    /// Bech32 `AccountId` of this account
    pub address: Address,
    /// public key
    pub verifying_key: VerifyingKey,
    /// private key
    pub signing_key: SigningKey,
}

impl TestAccount {
    pub fn random() -> Self {
        let signing_key = SigningKey::random(&mut rand_core::OsRng);
        let verifying_key = *signing_key.verifying_key();

        Self {
            address: AccAddress::new(verifying_key.into()).into(),
            verifying_key,
            signing_key,
        }
    }

    pub fn from_pk(pk: &[u8]) -> Self {
        let signing_key = SigningKey::from_slice(pk).unwrap();
        let verifying_key = *signing_key.verifying_key();

        Self {
            address: AccAddress::new(verifying_key.into()).into(),
            verifying_key,
            signing_key,
        }
    }
}

pub fn load_account() -> TestAccount {
    let address = include_str!("../../../ci/credentials/bridge-0.addr");
    let hex_key = include_str!("../../../ci/credentials/bridge-0.plaintext-key");

    let signing_key =
        SigningKey::from_slice(&hex::decode(hex_key.trim()).expect("valid hex representation"))
            .expect("valid key material");

    TestAccount {
        address: address.trim().parse().expect("valid address"),
        verifying_key: *signing_key.verifying_key(),
        signing_key,
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::{future::Future, sync::OnceLock};

    use celestia_grpc::{GrpcClient, TxClient};
    use tokio::sync::{Mutex, MutexGuard};
    use tonic::transport::Channel;

    use super::*;

    pub const CELESTIA_GRPC_URL: &str = "http://localhost:19090";

    pub fn new_grpc_client() -> GrpcClient<Channel> {
        GrpcClient::with_url(CELESTIA_GRPC_URL).expect("creating client failed")
    }

    // we have to sequence the tests which submits transactions.
    // multiple independent tx clients don't work well in parallel
    // as they break each other's account.sequence
    pub async fn new_tx_client() -> (MutexGuard<'static, ()>, TxClient<Channel, SigningKey>) {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let lock = LOCK.get_or_init(|| Mutex::new(())).lock().await;

        let creds = load_account();
        let client = TxClient::with_url(
            CELESTIA_GRPC_URL,
            &creds.address,
            creds.verifying_key,
            creds.signing_key,
        )
        .await
        .unwrap();

        (lock, client)
    }

    pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(future)
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use std::future::Future;

    use celestia_grpc::{GrpcClient, TxClient};
    use tokio::sync::oneshot;
    use tonic_web_wasm_client::Client;
    use wasm_bindgen_futures::spawn_local;

    use super::*;

    const CELESTIA_GRPCWEB_PROXY_URL: &str = "http://localhost:18080";

    pub fn new_grpc_client() -> GrpcClient<Client> {
        GrpcClient::with_grpcweb_url(CELESTIA_GRPCWEB_PROXY_URL)
    }

    pub async fn new_tx_client() -> ((), TxClient<Client, SigningKey>) {
        let creds = load_account();
        let client = TxClient::with_grpcweb_url(
            CELESTIA_GRPCWEB_PROXY_URL,
            &creds.address,
            creds.verifying_key,
            creds.signing_key,
        )
        .await
        .unwrap();

        ((), client)
    }

    pub fn spawn<F>(future: F) -> oneshot::Receiver<()>
    where
        F: Future<Output = ()> + 'static,
    {
        let (tx, rx) = oneshot::channel();
        spawn_local(async move {
            future.await;
            let _ = tx.send(());
        });

        rx
    }
}
