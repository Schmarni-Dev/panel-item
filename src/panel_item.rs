use std::{
    ops::Deref,
    sync::{Arc, OnceLock, atomic::AtomicU64},
};

use stardust_xr_fusion::AbortOnDrop;
use stardust_xr_wire::messenger::{self, MessageSenderHandle};
use tokio::{net::UnixStream, sync::Notify};

#[cfg(feature = "acceptor")]
use crate::protocol::acceptor::panel_item::PanelItem;
#[cfg(feature = "provider")]
use crate::protocol::provider::panel_item::PanelItem;
use crate::scenegraph::NodeRegistry;

pub struct PanelItemHandle {
    pub(crate) message_sender_handle: MessageSenderHandle,
    pub(crate) registry: NodeRegistry,
    _id_counter: AtomicU64,
    recv_task: OnceLock<AbortOnDrop>,
    event_handle: PanelItemEventHandle,
    item: OnceLock<PanelItem>,
}
#[derive(Clone, Debug)]
pub struct PanelItemEventHandle(Arc<Notify>);
impl PanelItemEventHandle {
    pub async fn wait(&self) {
        self.0.notified().await
    }
}

impl Deref for PanelItemHandle {
    type Target = PanelItem;

    fn deref(&self) -> &Self::Target {
        self.item.get().unwrap()
    }
}

impl PanelItemHandle {
    pub fn item(&self) -> &PanelItem {
        self.item.get().unwrap()
    }
    pub(crate) fn from_stream(stream: UnixStream) -> Arc<PanelItemHandle> {
        let (mut tx, mut rx) = messenger::create(stream);
        let handle = Arc::new_cyclic(|handle| PanelItemHandle {
            message_sender_handle: tx.handle(),
            registry: NodeRegistry::new(handle.clone()),
            #[cfg(feature = "acceptor")]
            _id_counter: AtomicU64::new(u64::MAX / 2),
            #[cfg(feature = "provider")]
            _id_counter: AtomicU64::new(64),
            recv_task: OnceLock::new(),
            event_handle: PanelItemEventHandle(Arc::new(Notify::new())),
            item: OnceLock::new(),
        });
        _ = handle.item.set(PanelItem::from_id(&handle, 32, false));
        let join_handle = tokio::spawn({
            let handle = handle.clone();
            async move {
                loop {
                    // TODO: are those futures cancel safe?
                    let r = tokio::select! {
                        r = tx.flush() => r,
                        r = rx.dispatch(&handle.registry) => r,
                    };
                    if r.is_ok() {
                        handle.event_handle.0.notify_waiters();
                    } else {
                        break;
                    }
                }
            }
        });

        _ = handle.recv_task.set(join_handle.into());
        handle
    }
    pub fn get_event_handle(&self) -> &PanelItemEventHandle {
        &self.event_handle
    }
}
