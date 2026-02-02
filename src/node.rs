use std::sync::Arc;

#[cfg(feature = "acceptor")]
use crate::protocol::acceptor::node::OwnedAspect;
#[cfg(feature = "provider")]
use crate::protocol::provider::node::OwnedAspect;
use serde::{Serialize, de::DeserializeOwned};
use stardust_xr_fusion::node::NodeError;
use stardust_xr_wire::{
    flex::{deserialize, serialize},
    messenger::MessengerError,
};

use crate::panel_item::PanelItemHandle;
/// Common methods all nodes share, to make them easier to use.
pub trait NodeType: Sized + Send + Sync + 'static {
    /// Get a reference to the node.
    fn node(&self) -> &NodeCore;
    // What's the node's ID? (used for comparison)
    fn id(&self) -> u64 {
        self.node().id
    }
    /// Try to get the client
    fn item(&self) -> &Arc<PanelItemHandle> {
        &self.node().item
    }
    /// Set whether the node is active or not. This has different effects depending on the node.
    fn set_enabled(&self, enabled: bool) -> Result<(), NodeError> {
        if self.node().owned {
            OwnedAspect::set_enabled(self.node(), enabled)
        } else {
            Err(NodeError::DoesNotExist)
        }
    }
}

pub struct NodeCore {
    pub item: Arc<PanelItemHandle>,
    pub id: u64,
    pub(crate) owned: bool,
}
impl NodeCore {
    pub(crate) fn new(item: Arc<PanelItemHandle>, id: u64, owned: bool) -> Self {
        Self { item, id, owned }
    }

    /// Send a signal directly - no weak reference upgrade!
    pub(crate) fn send_signal<S: Serialize>(
        &self,
        aspect: u64,
        signal: u64,
        data: &S,
    ) -> Result<(), NodeError> {
        let (serialized, fds) = serialize(data).map_err(|e| NodeError::Serialization { e })?;
        self.item
            .message_sender_handle
            .signal(self.id, aspect, signal, &serialized, fds)
            .map_err(|e| match e {
                MessengerError::ReceiverDropped => NodeError::ClientDropped,
                other => NodeError::MessengerError { e: other },
            })
    }

    #[allow(dead_code)]
    /// Execute a method directly - no weak reference upgrade!
    pub(crate) async fn call_method<S: Serialize, D: DeserializeOwned>(
        &self,
        aspect: u64,
        method: u64,
        data: &S,
    ) -> Result<D, NodeError> {
        let (serialized, fds) = serialize(data).map_err(|e| NodeError::Serialization { e })?;

        let response = self
            .item
            .message_sender_handle
            .method(self.id, aspect, method, &serialized, fds)
            .await
            .map_err(|e| match e {
                MessengerError::ReceiverDropped => NodeError::ClientDropped,
                other => NodeError::MessengerError { e: other },
            })?
            .map_err(|e| NodeError::ReturnedError { e })?;

        let (response, fds) = response.into_components();
        deserialize(&response, fds).map_err(|e| NodeError::Deserialization { e })
    }
}
impl NodeType for NodeCore {
    fn node(&self) -> &NodeCore {
        self
    }
}
impl OwnedAspect for NodeCore {}
impl std::fmt::Debug for NodeCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeCore")
            .field("id", &self.id)
            .field("owned", &self.owned)
            .finish()
    }
}
impl Drop for NodeCore {
    fn drop(&mut self) {
        if self.owned {
            let _ = self.destroy();
        }
    }
}
