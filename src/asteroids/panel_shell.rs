use std::sync::{Arc, Mutex};

use binderbinder::{TransactionHandler, binder_object::BinderObject};
use derive_setters::Setters;
use derive_where::derive_where;
use gluon_wire::{GluonDataReader, drop_tracking::DropNotifier};
use rand::random;
use stardust_xr_asteroids::{CustomElement, FnWrapper, Transformable, ValidState};
use stardust_xr_fusion::{
    drawable::DmatexSubmitInfo,
    node::{NodeError, NodeType},
    spatial::{Spatial, SpatialAspect, SpatialRef, Transform},
};
use tokio::sync::{RwLock, mpsc};

use crate::protocol::{ChildState, Geometry, PanelItem, PanelShellHandler as _, SurfaceId};

#[derive_where(Debug)]
#[derive(Setters)]
#[setters(into, strip_option)]
pub struct PanelShell<State: ValidState> {
    #[setters(skip)]
    handler: Arc<BinderObject<PanelShellHandler>>,
    #[setters(skip)]
    update_cursor_dmatex: FnWrapper<dyn Fn(&mut State, &PanelItem, DmatexSubmitInfo) + Send + Sync>,
    #[setters(skip)]
    update_surface_dmatex:
        FnWrapper<dyn Fn(&mut State, &PanelItem, SurfaceId, DmatexSubmitInfo, bool) + Send + Sync>,
    on_toplevel_fullscreen_changed: FnWrapper<dyn Fn(&mut State, &PanelItem, bool) + Send + Sync>,
    on_toplevel_title_changed: FnWrapper<dyn Fn(&mut State, &PanelItem, String) + Send + Sync>,
    on_toplevel_app_id_changed: FnWrapper<dyn Fn(&mut State, &PanelItem, String) + Send + Sync>,
    cursor_visuals_changed:
        FnWrapper<dyn Fn(&mut State, &PanelItem, Option<Geometry>) + Send + Sync>,
    new_child: FnWrapper<dyn Fn(&mut State, &PanelItem, ChildState) + Send + Sync>,
    child_moved: FnWrapper<dyn Fn(&mut State, &PanelItem, u64, Geometry) + Send + Sync>,
    child_removed: FnWrapper<dyn Fn(&mut State, &PanelItem, u64) + Send + Sync>,
    transform: Transform,
}

impl<State: ValidState> CustomElement<State> for PanelShell<State> {
    type Inner = SpatialRef;

    type Resource = ();

    type Error = NodeError;

    fn create_inner(
        &self,
        _asteroids_context: &stardust_xr_asteroids::Context,
        info: stardust_xr_asteroids::CreateInnerInfo,
        _resource: &mut Self::Resource,
    ) -> Result<Self::Inner, Self::Error> {
        Ok(info.parent_space.clone())
    }

    fn diff(&self, _old_self: &Self, inner: &mut Self::Inner, _resource: &mut Self::Resource) {
        // can't properly diff this since we don't know if this is the same spatial as last diff
        _ = self.handler.item_output_spatial.set_spatial_parent(inner);
        _ = self
            .handler
            .item_output_spatial
            .set_local_transform(self.transform.clone());
    }

    fn frame(
        &self,
        _context: &stardust_xr_asteroids::Context,
        _info: &stardust_xr_fusion::root::FrameInfo,
        state: &mut State,
        inner: &mut Self::Inner,
    ) {
        while let Ok(event) = self.handler.rx.lock().unwrap().try_recv() {
            match event {
                PanelShellEvent::UpdateCursorDmatex {
                    dmatex_uid,
                    acquire_point,
                    release_point,
                } => {
                    let dmatex_id: u64 = random();
                    stardust_xr_fusion::drawable::import_dmatex_uid(
                        &inner.client(),
                        dmatex_id,
                        dmatex_uid,
                    )
                    .unwrap();
                    self.update_cursor_dmatex.0(
                        state,
                        &self.handler.item,
                        DmatexSubmitInfo {
                            dmatex_id,
                            acquire_point,
                            release_point,
                        },
                    )
                }
                PanelShellEvent::UpdateSurfaceDmatex {
                    surface,
                    dmatex_uid,
                    acquire_point,
                    release_point,
                    opaque,
                } => {
                    let dmatex_id: u64 = random();
                    stardust_xr_fusion::drawable::import_dmatex_uid(
                        &inner.client(),
                        dmatex_id,
                        dmatex_uid,
                    )
                    .unwrap();
                    self.update_surface_dmatex.0(
                        state,
                        &self.handler.item,
                        surface,
                        DmatexSubmitInfo {
                            dmatex_id,
                            acquire_point,
                            release_point,
                        },
                        opaque,
                    )
                }
                PanelShellEvent::ToplevelFullscreen { fullscreen_active } => {
                    self.on_toplevel_fullscreen_changed.0(
                        state,
                        &self.handler.item,
                        fullscreen_active,
                    )
                }
                PanelShellEvent::ToplevelTitle { title } => {
                    self.on_toplevel_title_changed.0(state, &self.handler.item, title)
                }
                PanelShellEvent::ToplevelAppId { app_id } => {
                    self.on_toplevel_app_id_changed.0(state, &self.handler.item, app_id)
                }
                PanelShellEvent::SetCursorVisuals { geometry } => {
                    self.cursor_visuals_changed.0(state, &self.handler.item, geometry)
                }
                PanelShellEvent::CreateChild { child } => {
                    self.new_child.0(state, &self.handler.item, child)
                }
                PanelShellEvent::MoveChild { child_id, geometry } => {
                    self.child_moved.0(state, &self.handler.item, child_id, geometry)
                }
                PanelShellEvent::DestroyChild { child_id } => {
                    self.child_removed.0(state, &self.handler.item, child_id)
                }
            }
        }
    }

    fn spatial_aspect(&self, inner: &Self::Inner) -> stardust_xr_fusion::spatial::SpatialRef {
        inner.clone()
    }
}
impl<State: ValidState> Transformable for PanelShell<State> {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

#[derive(Debug)]
pub struct PanelShellHandler {
    tx: mpsc::UnboundedSender<PanelShellEvent>,
    rx: Mutex<mpsc::UnboundedReceiver<PanelShellEvent>>,
    item_output_spatial: Spatial,
    item: PanelItem,
    drop_notifs: RwLock<Vec<DropNotifier>>,
}
impl PanelShellHandler {
    pub fn new(item: PanelItem, item_output_spatial: Spatial) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx: Mutex::new(rx),
            item_output_spatial,
            item,
            drop_notifs: RwLock::default(),
        }
    }
    pub fn item(&self) -> &PanelItem {
        &self.item
    }
}

enum PanelShellEvent {
    UpdateCursorDmatex {
        dmatex_uid: u64,
        acquire_point: u64,
        release_point: u64,
    },
    UpdateSurfaceDmatex {
        surface: SurfaceId,
        dmatex_uid: u64,
        acquire_point: u64,
        release_point: u64,
        opaque: bool,
    },
    ToplevelFullscreen {
        fullscreen_active: bool,
    },
    ToplevelTitle {
        title: String,
    },
    ToplevelAppId {
        app_id: String,
    },
    SetCursorVisuals {
        geometry: Option<Geometry>,
    },
    CreateChild {
        child: ChildState,
    },
    MoveChild {
        child_id: u64,
        geometry: Geometry,
    },
    DestroyChild {
        child_id: u64,
    },
}

impl crate::protocol::PanelShellHandler for PanelShellHandler {
    fn update_cursor_dmatex(&self, dmatex_uid: u64, acquire_point: u64, release_point: u64) {
        self.tx
            .send(PanelShellEvent::UpdateCursorDmatex {
                dmatex_uid,
                acquire_point,
                release_point,
            })
            .unwrap();
    }

    fn update_surface_dmatex(
        &self,
        surface: SurfaceId,
        dmatex_uid: u64,
        acquire_point: u64,
        release_point: u64,
        opaque: bool,
    ) {
        self.tx
            .send(PanelShellEvent::UpdateSurfaceDmatex {
                surface,
                dmatex_uid,
                acquire_point,
                release_point,
                opaque,
            })
            .unwrap();
    }

    fn toplevel_fullscreen(&self, fullscreen_active: bool) {
        self.tx
            .send(PanelShellEvent::ToplevelFullscreen { fullscreen_active })
            .unwrap();
    }

    fn toplevel_title(&self, title: String) {
        self.tx
            .send(PanelShellEvent::ToplevelTitle { title })
            .unwrap();
    }

    fn toplevel_app_id(&self, app_id: String) {
        self.tx
            .send(PanelShellEvent::ToplevelAppId { app_id })
            .unwrap();
    }

    fn set_cursor_visuals(&self, geometry: Option<Geometry>) {
        self.tx
            .send(PanelShellEvent::SetCursorVisuals { geometry })
            .unwrap();
    }

    fn create_child(&self, child: ChildState) {
        self.tx
            .send(PanelShellEvent::CreateChild { child })
            .unwrap();
    }

    fn move_child(&self, child_id: u64, geometry: Geometry) {
        self.tx
            .send(PanelShellEvent::MoveChild { child_id, geometry })
            .unwrap();
    }

    fn destroy_child(&self, child_id: u64) {
        self.tx
            .send(PanelShellEvent::DestroyChild { child_id })
            .unwrap();
    }

    async fn drop_notification_requested(&self, notifier: gluon_wire::drop_tracking::DropNotifier) {
        self.drop_notifs.write().await.push(notifier);
    }
}
impl TransactionHandler for PanelShellHandler {
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
