use std::{
    env,
    fs::OpenOptions,
    sync::{Arc, Mutex, OnceLock},
};

use binderbinder::{
    TransactionHandler,
    binder_object::{BinderObject, ToBinderObjectOrRef},
};
use derive_setters::Setters;
use gluon_wire::{GluonDataReader, drop_tracking::DropNotifier};
use pion_binder::PionBinderDevice;
use stardust_xr_asteroids::{CustomElement, FnWrapper, Transformable, ValidState};
use stardust_xr_fusion::{
    fields::{Field, FieldAspect as _, Shape},
    node::NodeError,
    spatial::{Spatial, SpatialAspect, Transform},
};
use tokio::sync::{RwLock, mpsc};

use crate::{
    asteroids::panel_shell::PanelShellHandler,
    protocol::{
        FieldRefId, PanelItem, PanelItemAcceptor, PanelItemAcceptorHandler as _, PanelItemProvider,
        PanelShell, SpatialRefId,
    },
};

#[derive_where::derive_where(Debug)]
#[derive(Setters)]
#[setters(into, strip_option)]
pub struct PanelItemAcceptorElement<State: ValidState> {
    #[setters(skip)]
    binder_dev: PionBinderDevice,
    transform: Transform,
    shape: Shape,
    on_create_item:
        FnWrapper<dyn Fn(&mut State, Arc<BinderObject<PanelShellHandler>>) + Send + Sync>,
}

impl<State: ValidState> CustomElement<State> for PanelItemAcceptorElement<State> {
    type Inner = Arc<BinderObject<PanelItemAcceptorHandler>>;

    type Resource = ();

    type Error = NodeError;

    fn create_inner(
        &self,
        _ctx: &stardust_xr_asteroids::Context,
        info: stardust_xr_asteroids::CreateInnerInfo,
        _resource: &mut Self::Resource,
    ) -> Result<Self::Inner, Self::Error> {
        let field = Field::create(
            info.parent_space,
            self.transform.clone(),
            self.shape.clone(),
        )?;
        let (tx, rx) = mpsc::unbounded_channel();
        let handler = self.binder_dev.register_object(PanelItemAcceptorHandler {
            field,
            field_id: OnceLock::new(),
            tx,
            rx: Mutex::new(rx),
            drop_notifs: RwLock::default(),
        });
        tokio::spawn({
            let dev = self.binder_dev.clone();
            let acceptor = PanelItemAcceptor::from_handler(&handler);
            async move {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(
                        directories::BaseDirs::new()
                            .unwrap()
                            .runtime_dir()
                            .unwrap()
                            .join(format!("{}.lock", env::var("WAYLAND_DISPLAY").unwrap())),
                    )
                    .unwrap();
                let binder_ref = dev.get_binder_ref_from_file(file).await.unwrap();
                let v = PanelItemProvider::from_object_or_ref(binder_ref);
                v.register_acceptor(acceptor);
                // TODO: figure out how to call drop_acceptor on drop
            }
        });

        Ok(handler)
    }

    fn diff(&self, old: &Self, inner: &mut Self::Inner, _resource: &mut Self::Resource) {
        self.apply_transform(old, &inner.field);
        if self.shape != old.shape {
            let _ = inner.field.set_shape(self.shape.clone());
        }
    }
    fn frame(
        &self,
        _context: &stardust_xr_asteroids::Context,
        _info: &stardust_xr_fusion::root::FrameInfo,
        state: &mut State,
        inner: &mut Self::Inner,
    ) {
        while let Ok(shell) = inner.rx.lock().unwrap().try_recv() {
            self.on_create_item.0(state, shell)
        }
    }

    fn spatial_aspect(&self, inner: &Self::Inner) -> stardust_xr_fusion::spatial::SpatialRef {
        inner.field.clone().as_spatial_ref()
    }
}
impl<State: ValidState> Transformable for PanelItemAcceptorElement<State> {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

#[derive(Debug)]
pub struct PanelItemAcceptorHandler {
    field: Field,
    field_id: OnceLock<FieldRefId>,
    tx: mpsc::UnboundedSender<Arc<BinderObject<PanelShellHandler>>>,
    rx: Mutex<mpsc::UnboundedReceiver<Arc<BinderObject<PanelShellHandler>>>>,
    drop_notifs: RwLock<Vec<DropNotifier>>,
}
impl crate::protocol::PanelItemAcceptorHandler for PanelItemAcceptorHandler {
    async fn accept(
        &self,
        item: PanelItem,
    ) -> (crate::protocol::PanelShell, crate::protocol::SpatialRefId) {
        let output_spatial = Spatial::create(&self.field, Transform::none()).unwrap();
        let id = output_spatial.export_spatial().await.unwrap();

        let handler = PanelShellHandler::new(item.clone(), output_spatial);
        let panel_shell = item
            .to_binder_object_or_ref()
            .device()
            .register_object(handler);
        self.tx.send(panel_shell.clone()).unwrap();
        (PanelShell::from_handler(&panel_shell), SpatialRefId { id })
    }

    async fn get_field(&self) -> crate::protocol::FieldRefId {
        if let Some(id) = self.field_id.get() {
            id.clone()
        } else {
            let id = self.field.export_field().await.unwrap();
            let id = FieldRefId { id };
            _ = self.field_id.set(id.clone());
            id
        }
    }

    async fn drop_notification_requested(&self, notifier: gluon_wire::drop_tracking::DropNotifier) {
        self.drop_notifs.write().await.push(notifier);
    }
}
impl TransactionHandler for PanelItemAcceptorHandler {
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
