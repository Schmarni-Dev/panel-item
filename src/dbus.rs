use std::{os::fd::OwnedFd, sync::Arc};

use stardust_xr_fusion::{
    fields::{Field, FieldRef},
    node::NodeError,
    objects::{FieldObject, SpatialObject},
    query_impl::ClientQueryContext,
};
use stardust_xr_gluon::{
    impl_queryable_for_proxy,
    query::{QueryContext, Queryable},
};
use stardust_xr_wire::flex::FlexSerializeError;
use thiserror::Error;
use tokio::{net::UnixStream, sync::mpsc};
use zbus::{Connection, fdo, zvariant::OwnedObjectPath};

use crate::panel_item::PanelItemHandle;

pub struct PanelItemAcceptor {
    path: OwnedObjectPath,
    connection: Connection,
    recv: mpsc::UnboundedReceiver<Arc<PanelItemHandle>>,
    field: Field,
}
impl PanelItemAcceptor {
    pub fn field(&self) -> &Field {
        &self.field
    }
    pub async fn new(
        connection: Connection,
        path: &str,
        field: Field,
    ) -> Result<Self, PanelItemAcceptorCreateError> {
        let path =
            OwnedObjectPath::try_from(path).map_err(PanelItemAcceptorCreateError::InvalidPath)?;
        let (sender, recv) = mpsc::unbounded_channel();
        connection
            .object_server()
            .at(
                path.as_ref(),
                FieldObject::new(field.clone())
                    .await
                    .map_err(PanelItemAcceptorCreateError::NodeExportFailed)?,
            )
            .await
            .unwrap();
        connection
            .object_server()
            .at(
                path.as_ref(),
                SpatialObject::new(field.clone().as_spatial())
                    .await
                    .map_err(PanelItemAcceptorCreateError::NodeExportFailed)?,
            )
            .await
            .unwrap();
        connection
            .object_server()
            .at(path.as_ref(), PanelItemAcceptorInterface { sender })
            .await
            .unwrap();
        Ok(Self {
            path,
            connection,
            recv,
            field,
        })
    }
    /// we can't properly check for errors like this
    pub fn new_and_hope(
        connection: &Connection,
        path: &str,
        field: Field,
    ) -> Result<Self, PanelItemAcceptorCreateError> {
        let path =
            OwnedObjectPath::try_from(path).map_err(PanelItemAcceptorCreateError::InvalidPath)?;
        let (sender, recv) = mpsc::unbounded_channel();
        tokio::spawn({
            let path = path.clone();
            let connection = connection.clone();
            let field = field.clone();
            async move {
                let Ok(spatial) = SpatialObject::new(field.clone().as_spatial()).await else {
                    return;
                };
                let Ok(field) = FieldObject::new(field.clone()).await else {
                    return;
                };
                connection
                    .object_server()
                    .at(path.as_ref(), field)
                    .await
                    .unwrap();
                connection
                    .object_server()
                    .at(path.as_ref(), spatial)
                    .await
                    .unwrap();
                connection
                    .object_server()
                    .at(path.as_ref(), PanelItemAcceptorInterface { sender })
                    .await
                    .unwrap();
            }
        });
        Ok(Self {
            path,
            connection: connection.clone(),
            recv,
            field,
        })
    }
    pub fn try_recv(&mut self) -> Option<Arc<PanelItemHandle>> {
        self.recv.try_recv().ok()
    }
    pub async fn recv(&mut self) -> Option<Arc<PanelItemHandle>> {
        self.recv.recv().await
    }
}

impl Drop for PanelItemAcceptor {
    fn drop(&mut self) {
        let obj_server = self.connection.object_server().clone();
        let path = self.path.clone();
        tokio::spawn(async move {
            _ = obj_server
                .remove::<PanelItemAcceptorInterface, _>(&path)
                .await;
            _ = obj_server.remove::<FieldObject, _>(&path).await;
            _ = obj_server.remove::<SpatialObject, _>(path).await;
        });
    }
}
pub struct PanelItemAcceptorProxy {
    proxy: PanelItemAcceptorInterfaceProxy<'static>,
    field_ref: FieldRef,
}
impl PanelItemAcceptorProxy {
    pub async fn connect(&self) -> Result<Arc<PanelItemHandle>, PanelItemConnectError> {
        let (stream, remote_stream) =
            UnixStream::pair().map_err(PanelItemConnectError::UnableToCreateUnixStream)?;
        self.proxy
            .accept_panel_item(
                OwnedFd::from(
                    remote_stream
                        .into_std()
                        .map_err(PanelItemConnectError::UnableToConvertStream)?,
                )
                .into(),
            )
            .await
            .map_err(PanelItemConnectError::ConnectionFailed)?;
        Ok(PanelItemHandle::from_stream(stream))
    }
    pub fn field(&self) -> &FieldRef {
        &self.field_ref
    }
}
impl<Ctx: QueryContext + ClientQueryContext> Queryable<Ctx> for PanelItemAcceptorProxy {
    async fn try_new(
        connection: &zbus::Connection,
        ctx: &std::sync::Arc<Ctx>,
        object: &stardust_xr_gluon::ObjectInfo,
        contains_interface: &(impl Fn(&zbus::names::InterfaceName) -> bool + Send + Sync),
    ) -> Option<Self> {
        let proxy =
            PanelItemAcceptorInterfaceProxy::try_new(connection, ctx, object, contains_interface)
                .await?;
        let field_ref = FieldRef::try_new(connection, ctx, object, contains_interface).await?;
        Some(Self { proxy, field_ref })
    }
}
#[derive(Error, Debug)]
pub enum PanelItemConnectError {
    #[error("failed to create unix stream pair: {0}")]
    UnableToCreateUnixStream(tokio::io::Error),
    #[error("failed to convert unix stream to std: {0}")]
    UnableToConvertStream(tokio::io::Error),
    #[error("failed properly open connection to panel item acceptor: {0}")]
    ConnectionFailed(fdo::Error),
}
#[derive(Error, Debug)]
pub enum PanelItemAcceptorCreateError {
    #[error("invalid path provided: {0}")]
    InvalidPath(zbus::zvariant::Error),
    #[error("unable to export spatial or field ref: {0}")]
    NodeExportFailed(NodeError),
}
#[derive(Error, Debug)]
pub enum SendError {
    #[error("failed to serializee message: {0}")]
    Serialize(#[from] FlexSerializeError),
    #[error("failed to write to stream: {0}")]
    WriteFailed(#[from] tokio::io::Error),
}

pub struct PanelItemAcceptorInterface {
    sender: mpsc::UnboundedSender<Arc<PanelItemHandle>>,
}

#[zbus::interface(name = "org.stardustxr.PanelItemAcceptor", proxy)]
impl PanelItemAcceptorInterface {
    /// Returns a unix socket
    pub async fn accept_panel_item(&self, stream: zbus::zvariant::OwnedFd) -> fdo::Result<()> {
        let stream =
            UnixStream::try_from(std::os::unix::net::UnixStream::from(OwnedFd::from(stream)))
                .map_err(|err| fdo::Error::Failed(format!("Unable to open stream fd: {err}")))?;
        _ = self.sender.send(PanelItemHandle::from_stream(stream));

        Ok(())
    }
}

impl_queryable_for_proxy!(PanelItemAcceptorInterfaceProxy);
