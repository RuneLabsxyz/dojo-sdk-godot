use std::cell::OnceCell;
use std::sync::Arc;
use godot::prelude::*;
use starknet_types_core::felt::Felt;
use crate::client::{DojoClient, DojoClientConfig};

/// The DojoWorld is the parent instance, that contains the APIs to interact with the contract.
#[derive(GodotClass)]
#[class(base=Node,init)]
struct DojoWorld {
    base: Base<Node>,
    /// The URL of the torii indexer
    #[export]
    torii_url: GString,
    /// The base URL to the RPC interface
    #[export]
    rpc_url: GString,
    /// The base URL to the relay server
    #[export]
    relay_url: GString,

    // TODO: Get an explanation of how this works in detail, this seems weird
    /// The ID of the main world
    #[export]
    world_id: GString,

    // Internal implementation
    client: OnceCell<Arc<DojoClient>>
}
 impl DojoWorld {
    fn get_config(&self) -> DojoClientConfig {
        DojoClientConfig {
            torii_url: self.torii_url.clone().into(),
            world_id: Felt::from_hex(&*String::from(self.world_id.clone()))
                .unwrap(),
            relay_url: self.relay_url.clone().into(),
            rpc_url: self.rpc_url.clone().into()
        }
    }
 }


#[godot_api]
impl INode for DojoWorld {
    fn process(&mut self, delta: f64) {
        if let Some(client) = self.client.get() {
            // For each frame, process no more than 5 events
            let mut iterator = client.take(5);

            while let Some(entity) = iterator.next() {
                todo!("Once the dojo bindgen is done, do the actual work of converting,\
                    and sending an event to whoever will listen to it.")
            }
        }
    }

    fn ready(&mut self) {
        // Create the client, and set it in the object
        let _ = self.client.set(
            DojoClient::new(
                self.get_config()
            ).expect("Impossible to create the new client")
        );
    }
}

#[godot_api]
impl DojoWorld {
    /// This signal is called each time an entity is updated.
    #[signal]
    pub fn on_update();
}