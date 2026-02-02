use derive_setters::Setters;
use stardust_xr_asteroids::{CustomElement, FnWrapper, Transformable, ValidState};
use stardust_xr_fusion::{
    fields::{Field, FieldAspect as _, Shape},
    node::NodeError,
    spatial::Transform,
};
use thiserror::Error;

use crate::{dbus::{PanelItemAcceptor, PanelItemAcceptorCreateError}, panel_item::PanelItemHandle};

#[derive_where::derive_where(Debug, PartialEq)]
#[derive(Setters)]
#[setters(into, strip_option)]
pub struct PanelItemAcceptorElement<State:ValidState> {
    transform: Transform,
    shape: Shape,
    on_create_item: FnWrapper<dyn Fn(&mut State, PanelItemHandle, PanelItemInitData) + Send + Sync>,
}

impl<State: ValidState> CustomElement<State> for PanelItemAcceptorElement<State> {
    type Inner = PanelItemAcceptor;

    type Resource = ();

    type Error = PanelItemAcceptorElementError;

    fn create_inner(
        &self,
        ctx: &stardust_xr_asteroids::Context,
        info: stardust_xr_asteroids::CreateInnerInfo,
        _resource: &mut Self::Resource,
    ) -> Result<Self::Inner, Self::Error> {
        Ok(PanelItemAcceptor::new_and_hope(
            &ctx.dbus_connection,
            info.element_path
                .to_str()
                .ok_or(PanelItemAcceptorElementError::PathToStr)?,
            Field::create(info.parent_space, self.transform, self.shape.clone())
                .map_err(PanelItemAcceptorElementError::FieldCreation)?,
        )?)
    }

    fn diff(&self, old: &Self, inner: &mut Self::Inner, _resource: &mut Self::Resource) {
        self.apply_transform(old, inner.field());
        if self.shape != old.shape {
            let _ = inner.field().set_shape(self.shape.clone());
        }
    }
    fn frame(
        &self,
        _context: &stardust_xr_asteroids::Context,
        _info: &stardust_xr_fusion::root::FrameInfo,
        _state: &mut State,
        _inner: &mut Self::Inner,
    ) {
    }

    fn spatial_aspect(&self, inner: &Self::Inner) -> stardust_xr_fusion::spatial::SpatialRef {
        inner.field().clone().as_spatial_ref()
    }
}
impl Transformable for PanelItemAcceptorElement {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

#[derive(Error, Debug)]
pub enum PanelItemAcceptorElementError {
    #[error("failed to create underlying acceptor: {0}")]
    CreateError(#[from] PanelItemAcceptorCreateError),
    #[error("unable to convert element path to str")]
    PathToStr,
    #[error("failed to create field: {0}")]
    FieldCreation(NodeError),
}
