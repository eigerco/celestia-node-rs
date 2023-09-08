use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{p2p::P2pService, store::Store, Service};

type Result<T, E = SyncerError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum SyncerError {}

#[allow(unused)]
#[derive(Debug)]
pub struct Syncer<P2pSrv: P2pService> {
    p2p: Arc<P2pSrv>,
    store: Arc<RwLock<Store>>,
}

pub struct SyncerArgs<P2pSrv: P2pService> {
    pub p2p: Arc<P2pSrv>,
    pub store: Arc<RwLock<Store>>,
}

#[doc(hidden)]
#[derive(Debug)]
pub enum SyncerCmd {}

#[async_trait]
impl<P2pSrv: P2pService> Service for Syncer<P2pSrv> {
    type Command = SyncerCmd;
    type Args = SyncerArgs<P2pSrv>;
    type Error = SyncerError;

    async fn start(args: SyncerArgs<P2pSrv>) -> Result<Self, SyncerError> {
        Ok(Self {
            p2p: args.p2p,
            store: args.store,
        })
    }

    async fn stop(&self) -> Result<()> {
        todo!()
    }

    async fn send_command(&self, _cmd: SyncerCmd) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
pub trait SyncerService<P2pSrv: P2pService>:
    Service<Args = SyncerArgs<P2pSrv>, Command = SyncerCmd, Error = SyncerError>
{
}

#[async_trait]
impl<P2pSrv: P2pService> SyncerService<P2pSrv> for Syncer<P2pSrv> {}
