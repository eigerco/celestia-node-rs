//! A browser compatible wrappers for the [`lumina-node`].
use std::result::Result as StdResult;

use js_sys::Array;
use libp2p::identity::Keypair;
use libp2p::multiaddr::Protocol;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use tracing::error;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, SharedWorker, WorkerOptions, WorkerType};

use lumina_node::blockstore::IndexedDbBlockstore;
use lumina_node::network::{canonical_network_bootnodes, network_genesis, network_id};
use lumina_node::node::NodeConfig;
use lumina_node::store::IndexedDbStore;

use crate::utils::{js_value_from_display, JsContext, Network};
use crate::worker::commands::{
    GetConnectedPeers, GetHeader, GetListeners, GetLocalPeerId, GetMultipleHeaders, GetNetworkInfo,
    GetPeerTrackerInfo, GetSamplingMetadata, GetSyncerInfo, IsRunning, LastSeenNetworkHead,
    MultipleHeaderQuery, NodeCommand, NodeResponse, RequestHeader, RequestMultipleHeaders,
    SetPeerTrust, SingleHeaderQuery, StartNode, WaitConnected,
};
use crate::worker::SharedWorkerChannel;
use crate::wrapper::libp2p::NetworkInfoSnapshot;
use crate::Result;

const LUMINA_SHARED_WORKER_NAME: &str = "lumina";

/// Config for the lumina wasm node.
#[wasm_bindgen(js_name = NodeConfig)]
#[derive(Serialize, Deserialize, Debug)]
pub struct WasmNodeConfig {
    /// A network to connect to.
    pub network: Network,
    /// Hash of the genesis block in the network.
    #[wasm_bindgen(getter_with_clone)]
    pub genesis_hash: Option<String>,
    /// A list of bootstrap peers to connect to.
    #[wasm_bindgen(getter_with_clone)]
    pub bootnodes: Vec<String>,
}

#[wasm_bindgen]
struct NodeDriver {
    _worker: SharedWorker,
    _onerror_callback: Closure<dyn Fn(MessageEvent)>,
    channel: SharedWorkerChannel<NodeCommand, NodeResponse>,
}

#[wasm_bindgen]
impl NodeDriver {
    /// Create a new connection to a Lumina node in a Shared Worker.
    /// Note that single Shared Worker can be accessed from multiple tabs, so Lumina may already
    /// be running be running, before `NodeDriver::start` call.
    #[wasm_bindgen(constructor)]
    pub async fn new() -> NodeDriver {
        let mut opts = WorkerOptions::new();
        opts.type_(WorkerType::Module);
        opts.name(LUMINA_SHARED_WORKER_NAME);
        let worker = SharedWorker::new_with_worker_options("/js/worker.js", &opts)
            .expect("could not worker");

        let onerror_callback: Closure<dyn Fn(MessageEvent)> = Closure::new(|ev: MessageEvent| {
            error!("received error from shared worker: {ev:?}");
        });
        worker.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));

        let channel = SharedWorkerChannel::new(worker.port());

        Self {
            _worker: worker,
            _onerror_callback: onerror_callback,
            channel,
        }
    }

    /// Check whether Lumina is currently running
    pub async fn is_running(&mut self) -> Result<bool> {
        let response = self.channel.send(IsRunning).await?;

        Ok(response.await?)
    }

    /// Start a node with the provided config, if it's not running
    pub async fn start(&self, config: WasmNodeConfig) -> Result<()> {
        let command = StartNode(config);
        let response = self.channel.send(command).await?;

        Ok(response.await??)
    }

    /// Get node's local peer ID.
    pub async fn local_peer_id(&self) -> Result<String> {
        let response = self.channel.send(GetLocalPeerId).await?;

        Ok(response.await?)
    }

    /// Get current [`PeerTracker`] info.
    pub async fn peer_tracker_info(&self) -> Result<JsValue> {
        let response = self.channel.send(GetPeerTrackerInfo).await?;

        Ok(to_value(&response.await?)?)
    }

    /// Wait until the node is connected to at least 1 peer.
    pub async fn wait_connected(&self) -> Result<()> {
        let command = WaitConnected { trusted: false };
        let response = self.channel.send(command).await?;

        Ok(response.await?)
    }

    /// Wait until the node is connected to at least 1 trusted peer.
    pub async fn wait_connected_trusted(&self) -> Result<()> {
        let command = WaitConnected { trusted: false };
        let response = self.channel.send(command).await?;

        Ok(response.await?)
    }

    /// Get current network info.
    pub async fn network_info(&self) -> Result<NetworkInfoSnapshot> {
        let response = self.channel.send(GetNetworkInfo).await?;

        Ok(response.await?)
    }

    /// Get all the multiaddresses on which the node listens.
    pub async fn listeners(&self) -> Result<Array> {
        let response = self.channel.send(GetListeners).await?;
        let response = response.await?.iter().map(js_value_from_display).collect();

        Ok(response)
    }

    /// Get all the peers that node is connected to.
    pub async fn connected_peers(&self) -> Result<Array> {
        let response = self.channel.send(GetConnectedPeers).await?;
        let response = response.await?.iter().map(js_value_from_display).collect();

        Ok(response)
    }

    /// Trust or untrust the peer with a given ID.
    pub async fn set_peer_trust(&self, peer_id: &str, is_trusted: bool) -> Result<()> {
        let command = SetPeerTrust {
            peer_id: peer_id.to_string(),
            is_trusted,
        };
        let response = self.channel.send(command).await?;

        Ok(response.await?)
    }

    /// Request the head header from the network.
    pub async fn request_head_header(&self) -> Result<JsValue> {
        let command = RequestHeader(SingleHeaderQuery::Head);
        let response = self.channel.send(command).await?;
        Ok(to_value(&response.await??)?)
    }

    /// Request a header for the block with a given hash from the network.
    pub async fn request_header_by_hash(&self, hash: &str) -> Result<JsValue> {
        let command = RequestHeader(SingleHeaderQuery::ByHash(hash.parse()?));
        let response = self.channel.send(command).await?;

        Ok(to_value(&response.await??)?)
    }

    /// Request a header for the block with a given height from the network.
    pub async fn request_header_by_height(&self, height: u64) -> Result<JsValue> {
        let command = RequestHeader(SingleHeaderQuery::ByHeight(height));
        let response = self.channel.send(command).await?;

        Ok(to_value(&response.await??)?)
    }

    /// Request headers in range (from, from + amount] from the network.
    ///
    /// The headers will be verified with the `from` header.
    pub async fn request_verified_headers(
        &self,
        from_header: JsValue,
        amount: u64,
    ) -> Result<Array> {
        let from = from_value(from_header)?;
        let command = RequestMultipleHeaders(MultipleHeaderQuery::GetVerified { from, amount });
        let response = self.channel.send(command).await?;

        let result = response
            .await
            .iter()
            .map(|h| to_value(&h).unwrap()) // XXX
            .collect();

        Ok(result)
    }

    /// Get current header syncing info.
    pub async fn syncer_info(&self) -> Result<JsValue> {
        let response = self.channel.send(GetSyncerInfo).await?;

        Ok(to_value(&response.await?)?)
    }

    /// Get the latest header announced in the network.
    pub async fn get_network_head_header(&self) -> Result<JsValue> {
        let command = LastSeenNetworkHead;
        let response = self.channel.send(command).await?;

        Ok(to_value(&response.await?)?)
    }

    /// Get the latest locally synced header.
    pub async fn get_local_head_header(&self) -> Result<JsValue> {
        let command = GetHeader(SingleHeaderQuery::Head);
        let response = self.channel.send(command).await?;

        Ok(to_value(&response.await?)?)
    }

    /// Get a synced header for the block with a given hash.
    pub async fn get_header_by_hash(&self, hash: &str) -> Result<JsValue> {
        let command = GetHeader(SingleHeaderQuery::ByHash(hash.parse()?));
        let response = self.channel.send(command).await?;

        Ok(to_value(&response.await?)?)
    }

    /// Get a synced header for the block with a given height.
    pub async fn get_header_by_height(&self, height: u64) -> Result<JsValue> {
        let command = GetHeader(SingleHeaderQuery::ByHeight(height));
        let response = self.channel.send(command).await?;

        Ok(to_value(&response.await?)?)
    }

    /// Get synced headers from the given heights range.
    ///
    /// If start of the range is undefined (None), the first returned header will be of height 1.
    /// If end of the range is undefined (None), the last returned header will be the last header in the
    /// store.
    ///
    /// # Errors
    ///
    /// If range contains a height of a header that is not found in the store.
    pub async fn get_headers(
        &self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> Result<Array> {
        let command = GetMultipleHeaders(MultipleHeaderQuery::Range {
            start_height,
            end_height,
        });
        let response = self.channel.send(command).await?;
        let result = response
            .await
            .iter()
            .map(|h| to_value(&h).unwrap())
            .collect();

        Ok(result)
    }

    /// Get data sampling metadata of an already sampled height.
    pub async fn get_sampling_metadata(&self, height: u64) -> Result<JsValue> {
        let command = GetSamplingMetadata { height };
        let response = self.channel.send(command).await?;

        Ok(to_value(&response.await?)?)
    }
}

#[wasm_bindgen(js_class = NodeConfig)]
impl WasmNodeConfig {
    /// Get the configuration with default bootnodes and genesis hash for provided network
    pub fn default(network: Network) -> WasmNodeConfig {
        WasmNodeConfig {
            network,
            genesis_hash: network_genesis(network.into()).map(|h| h.to_string()),
            bootnodes: canonical_network_bootnodes(network.into())
                .filter(|addr| addr.iter().any(|proto| proto == Protocol::WebTransport))
                .map(|addr| addr.to_string())
                .collect::<Vec<_>>(),
        }
    }

    pub(crate) async fn into_node_config(
        self,
    ) -> Result<NodeConfig<IndexedDbBlockstore, IndexedDbStore>> {
        let network_id = network_id(self.network.into());
        let store = IndexedDbStore::new(network_id)
            .await
            .js_context("Failed to open the store")?;
        let blockstore = IndexedDbBlockstore::new(&format!("{network_id}-blockstore"))
            .await
            .js_context("Failed to open the blockstore")?;

        let p2p_local_keypair = Keypair::generate_ed25519();

        let genesis_hash = self.genesis_hash.map(|h| h.parse()).transpose()?;
        let p2p_bootnodes = self
            .bootnodes
            .iter()
            .map(|addr| addr.parse())
            .collect::<StdResult<_, _>>()?;

        Ok(NodeConfig {
            network_id: network_id.to_string(),
            genesis_hash,
            p2p_bootnodes,
            p2p_local_keypair,
            p2p_listen_on: vec![],
            blockstore,
            store,
        })
    }
}
