use std::io;
use std::sync::{Arc, Mutex};
use async_channel::{unbounded, Receiver, Sender};
use starknet_types_core::felt::Felt;
use thiserror::Error;
use torii_grpc::types::schema::Entity as DojoEntity;
use log::info;
use torii_client::client::Client;
use torii_grpc::types::PatternMatching;
use torii_grpc::types::{EntityKeysClause, KeysClause};
use futures_util::StreamExt;

#[cfg(not(target_arch = "wasm32"))]
use tokio::{
    runtime::Runtime,
    task::spawn_local
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error while creating thread")]
    RuntimeError(#[from] io::Error)
}

pub struct DojoClientConfig {
    /// The URL of the torii indexer
    pub torii_url: String,
    /// The base URL to the RPC interface

    pub rpc_url: String,
    /// The base URL to the relay server
    pub relay_url: String,

    // TODO: Get an explanation of how this works in detail, this seems weird
    /// The ID of the main world
    pub world_id: Felt,
}

pub struct EntityIterator<'a> {
    channel: &'a Receiver<DojoEntity>,
    count: usize
}

impl Iterator for EntityIterator<'_> {
    type Item = DojoEntity;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count <= 0 {
            None
        } else {
            self.count -= 1;
            self.channel.try_recv()
                .ok()
        }
    }
}

pub struct DojoClient {
    // We have to keep a handle to the runtime for at least as long as the dojo client is active.
    #[cfg(not(target_arch = "wasm32"))]
    runtime: Runtime,

    config: DojoClientConfig,
    rx: Receiver<DojoEntity>,
}

impl DojoClient {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(config: DojoClientConfig) -> Result<Arc<Self>, Error> {


        let (result, tx) = Self::new_internal(config)?;

        let borrowed = Arc::new(result);
        let cloned = borrowed.clone();
        borrowed.runtime.spawn(async move {
            cloned.start(tx).await
        });

        Ok(borrowed)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(config: DojoClientConfig) -> Result<Arc<Self>, Error> {
        use wasm_bindgen_futures::spawn_local;

        let (result, tx) = Self::new_internal(config)?;

        let borrowed = Arc::new(result);
        let cloned = borrowed.clone();
        spawn_local(async move {
            cloned.start(tx).await
        });

        Ok(borrowed)
    }

    fn new_internal(config: DojoClientConfig) -> Result<(Self, Sender<DojoEntity>), Error> {
        // MEMORY: In terms of memory usage, this is really risky.
        //         If a lot of messages are getting through, we could exhaust the available memory,
        //         especially in the event of a memory bound limitation.
        let (tx, rx) =
            unbounded::<DojoEntity>();

        Ok((Self {
            rx,
            config,
            // SAFETY: Move this somewhere else, so that we have to chance of fucking things up.
            #[cfg(not(target_arch = "wasm32"))]
            runtime: Runtime::new()?
        }, tx))
    }

    pub fn take(&self, max: usize) -> EntityIterator {
        EntityIterator {
            channel: &self.rx,
            count: max,
        }
    }

    async fn start(&self, tx: Sender<DojoEntity>) {
        info!("Starting torii client");

        let client = Client::new(
            self.config.torii_url.clone(),
            self.config.rpc_url.clone(),
            self.config.relay_url.clone(),
            self.config.world_id.clone()
        )
            .await
            // We have no other choice, as we are in a task context, and cannot easily bubble up the problems.
            .expect("Impossible to create torii client");


        let mut receiver = client.on_entity_updated(vec![
            EntityKeysClause::Keys(KeysClause {
                keys: vec![],
                pattern_matching: PatternMatching::VariableLen,
                models: vec![]
            })
        ]).await.expect("Impossible to create a new receiver");

        while let Some(Ok((_, entity))) = receiver.next().await {
            tx.send(entity).await
                .expect("Impossible to send entity. Is the channel closed?")
        }
    }
}
