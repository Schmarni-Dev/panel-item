use std::{fmt::Debug, sync::Arc};

use binderbinder::TransactionHandler;
use gluon_wire::{GluonDataReader, drop_tracking::DropNotifier};
use rustc_hash::FxHashMap;
use stardust_xr_fusion::{ClientHandle, fields::FieldRef};
use tokio::sync::{RwLock, watch};

use crate::protocol::{PanelItemAcceptor, PanelItemProviderHandler as _};

pub struct PanelItemProviderHandler {
    client: Arc<ClientHandle>,
    watch: watch::Receiver<FxHashMap<u64, (FieldRef, PanelItemAcceptor)>>,
    tx: watch::Sender<FxHashMap<u64, (FieldRef, PanelItemAcceptor)>>,
    drop_notifs: RwLock<Vec<DropNotifier>>,
}

impl Debug for PanelItemProviderHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PanelItemProviderHandler")
            .field("watch", &self.watch)
            .field("tx", &self.tx)
            .finish()
    }
}
impl PanelItemProviderHandler {
    pub fn new(client: Arc<ClientHandle>) -> Self {
        let (tx, watch) = watch::channel(FxHashMap::default());
        Self {
            client,
            watch,
            tx,
            drop_notifs: RwLock::default(),
        }
    }
    pub fn acceptors(&self) -> watch::Ref<'_, FxHashMap<u64, (FieldRef, PanelItemAcceptor)>> {
        self.watch.borrow()
    }
}
impl crate::protocol::PanelItemProviderHandler for PanelItemProviderHandler {
    fn register_acceptor(&self, acceptor: PanelItemAcceptor) {
        let tx = self.tx.clone();
        let client = self.client.clone();
        tokio::spawn(async move {
            let field_id = acceptor.get_field().await;
            let Ok(field) = FieldRef::import(&client, field_id.id).await else {
                return;
            };
            tx.send_modify(|v| {
                v.insert(field_id.id, (field, acceptor));
            });
        });
    }

    fn drop_acceptor(&self, acceptor: PanelItemAcceptor) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let field_id = acceptor.get_field().await;
            tx.send_modify(|v| {
                v.remove(&field_id.id);
            });
        });
    }

    async fn drop_notification_requested(&self, notifier: gluon_wire::drop_tracking::DropNotifier) {
        self.drop_notifs.write().await.push(notifier);
    }
}
impl TransactionHandler for PanelItemProviderHandler {
    async fn handle(
        &self,
        transaction: binderbinder::device::Transaction,
    ) -> binderbinder::payload::PayloadBuilder<'_> {
        let mut data = GluonDataReader::from_payload(transaction.payload);
        self.dispatch_two_way(transaction.code, &mut data)
            .await
            .to_payload()
    }

    async fn handle_one_way(&self, transaction: binderbinder::device::Transaction) {
        let mut data = GluonDataReader::from_payload(transaction.payload);
        self.dispatch_one_way(transaction.code, &mut data).await
    }
}
