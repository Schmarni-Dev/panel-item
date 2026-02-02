#![allow(async_fn_in_trait, unused_parens, clippy::all)]
use crate::node::NodeType;
pub(crate) trait AddAspect<A> {
    fn add_aspect(
        registry: &crate::scenegraph::NodeRegistry,
        node_id: u64,
        aspect_id: u64,
    );
}
#[allow(unused_imports)]
use node::*;
pub mod node {
    #[allow(unused_imports)]
    use super::*;
    pub(crate) const INTERFACE_VERSION: u32 = 1u32;
    #[derive(Debug)]
    pub enum OwnedEvent {}
    ///This node was created by the current client and can be disabled/destroyed
    pub trait OwnedAspect: crate::node::NodeType + std::fmt::Debug {
        ///Set if this node is enabled or not. Disabled drawables won't render, input handlers won't receive input, etc.
        fn set_enabled(
            &self,
            enabled: bool,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (bool) = (enabled);
            self.node()
                .send_signal(15801764205032075891u64, 13365497663235993822u64, &data)?;
            let (enabled) = data;
            tracing::trace!(
                ? enabled, "Sent signal to server, {}::{}", "Owned", "set_enabled"
            );
            Ok(())
        }
        ///Destroy this node immediately. Not all nodes will have this method, those that don't can be dropped client-side without issue.
        fn destroy(&self) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: () = ();
            self.node()
                .send_signal(15801764205032075891u64, 8637450960623370830u64, &data)?;
            let () = data;
            tracing::trace!("Sent signal to server, {}::{}", "Owned", "destroy");
            Ok(())
        }
    }
}
#[allow(unused_imports)]
use panel_item::*;
pub mod panel_item {
    #[allow(unused_imports)]
    use super::*;
    pub(crate) const INTERFACE_VERSION: u32 = 1u32;
    pub(crate) const INTERFACE_NODE_ID: u64 = 12u64;
    ///
    #[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
    #[serde(tag = "t", content = "c")]
    pub enum SurfaceId {
        Toplevel(()),
        Child(u64),
    }
    ///The origin and size of the surface's "solid" part.
    #[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Geometry {
        pub origin: stardust_xr_wire::values::Vector2<i32>,
        pub size: stardust_xr_wire::values::Vector2<u32>,
    }
    ///The state of the panel item's toplevel.
    #[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct ToplevelState {
        ///The UID of the panel item of the parent of this toplevel, if it exists
        pub parent: Option<u64>,
        ///Equivalent to the window title
        pub title: Option<String>,
        ///Application identifier, see <https://standards.freedesktop.org/desktop-entry-spec/>
        pub app_id: Option<String>,
        ///Current size in pixels
        pub size: stardust_xr_wire::values::Vector2<u32>,
        ///Minimum size in pixels
        pub min_size: Option<stardust_xr_wire::values::Vector2<f32>>,
        ///Maximum size in pixels
        pub max_size: Option<stardust_xr_wire::values::Vector2<f32>>,
        ///Surface geometry
        pub logical_rectangle: Geometry,
    }
    ///Data on positioning a child.
    #[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct ChildInfo {
        pub id: u64,
        ///If this is None, the Toplevel is the parent
        pub parent: Option<u64>,
        pub geometry: Geometry,
        ///Relative to parent. 0 is same level, -1 is below, 1 is above, etc.
        pub z_order: i32,
        ///Whether this child receives input or is purely visual.
        pub receives_input: bool,
        pub input_regions: Vec<stardust_xr_wire::values::Vector2<f32>>,
    }
    ///The init data for the panel item.
    #[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct PanelItemInitData {
        pub cursor: Option<Geometry>,
        pub toplevel: ToplevelState,
        pub children: Vec<ChildInfo>,
    }
    ///An item that represents a toplevel 2D window's surface (base window) and all its children (context menus, modals, etc.).
    #[derive(Debug, Clone)]
    pub struct PanelItem {
        pub(crate) core: std::sync::Arc<crate::node::NodeCore>,
        pub(crate) panel_item_event: std::sync::Arc<
            std::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<PanelItemEvent>>,
        >,
    }
    impl PanelItem {
        pub(crate) fn from_id(
            client: &std::sync::Arc<crate::panel_item::PanelItemHandle>,
            id: u64,
            owned: bool,
        ) -> Self {
            let core = std::sync::Arc::new(
                crate::node::NodeCore::new(client.clone(), id, owned),
            );
            let panel_item_event = std::sync::Arc::new(
                client.registry.add_aspect(id, 16007573185838633179u64).into(),
            );
            PanelItem {
                core,
                panel_item_event,
            }
        }
    }
    impl crate::node::NodeType for PanelItem {
        fn node(&self) -> &crate::node::NodeCore {
            &self.core
        }
    }
    impl std::hash::Hash for PanelItem {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.core.id.hash(state);
        }
    }
    impl std::cmp::PartialEq for PanelItem {
        fn eq(&self, other: &Self) -> bool {
            self.core.id == other.core.id
        }
    }
    impl std::cmp::Eq for PanelItem {}
    impl serde::Serialize for PanelItem {
        fn serialize<S: serde::Serializer>(
            &self,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            serializer.serialize_u64(self.core.id)
        }
    }
    impl PanelItemAspect for PanelItem {
        fn recv_panel_item_event(&self) -> Option<PanelItemEvent> {
            self.panel_item_event.lock().unwrap().try_recv().ok()
        }
    }
    #[derive(Debug)]
    pub enum PanelItemEvent {
        UpdateCursorDmatex { dmatex_uid: u64, acquire_point: u64, release_point: u64 },
        UpdateSurfaceDmatex {
            child: Option<u64>,
            dmatex_uid: u64,
            acquire_point: u64,
            release_point: u64,
        },
        ToplevelTitleChanged { title: String },
        ToplevelAppIdChanged { app_id: String },
        ToplevelFullscreenActive { active: bool },
        ToplevelMoveRequest {},
        ToplevelResizeRequest { up: bool, down: bool, left: bool, right: bool },
        ToplevelSizeChanged { size: stardust_xr_wire::values::Vector2<u32> },
        ExclusiveKeyboardGrab { child: u64 },
        SetCursor { geometry: Geometry },
        HideCursor {},
        CreateChild { uid: u64, info: ChildInfo },
        RepositionChild { uid: u64, geometry: Geometry },
        DestroyChild { uid: u64 },
    }
    impl crate::scenegraph::EventParser for PanelItemEvent {
        const ASPECT_ID: u64 = 16007573185838633179u64;
        fn parse_signal(
            _client: &std::sync::Arc<crate::panel_item::PanelItemHandle>,
            signal_id: u64,
            _data: &[u8],
            _fds: Vec<std::os::fd::OwnedFd>,
        ) -> Result<Self, stardust_xr_wire::scenegraph::ScenegraphError> {
            match signal_id {
                2382773608549259152u64 => {
                    let (dmatex_uid, acquire_point, release_point): (u64, u64, u64) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? dmatex_uid, ? acquire_point, ? release_point,
                        "Got signal from server, {}::{}", "PanelItem",
                        "update_cursor_dmatex"
                    );
                    Ok(PanelItemEvent::UpdateCursorDmatex {
                        dmatex_uid: dmatex_uid,
                        acquire_point: acquire_point,
                        release_point: release_point,
                    })
                }
                14640500090763198919u64 => {
                    let (
                        child,
                        dmatex_uid,
                        acquire_point,
                        release_point,
                    ): (Option<u64>, u64, u64, u64) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? child, ? dmatex_uid, ? acquire_point, ? release_point,
                        "Got signal from server, {}::{}", "PanelItem",
                        "update_surface_dmatex"
                    );
                    Ok(PanelItemEvent::UpdateSurfaceDmatex {
                        child: child,
                        dmatex_uid: dmatex_uid,
                        acquire_point: acquire_point,
                        release_point: release_point,
                    })
                }
                566483566315648641u64 => {
                    let (title): (String) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? title, "Got signal from server, {}::{}", "PanelItem",
                        "toplevel_title_changed"
                    );
                    Ok(PanelItemEvent::ToplevelTitleChanged {
                        title: title,
                    })
                }
                8706869778156655494u64 => {
                    let (app_id): (String) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? app_id, "Got signal from server, {}::{}", "PanelItem",
                        "toplevel_app_id_changed"
                    );
                    Ok(PanelItemEvent::ToplevelAppIdChanged {
                        app_id: app_id,
                    })
                }
                11059551561818960198u64 => {
                    let (active): (bool) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? active, "Got signal from server, {}::{}", "PanelItem",
                        "toplevel_fullscreen_active"
                    );
                    Ok(PanelItemEvent::ToplevelFullscreenActive {
                        active: active,
                    })
                }
                3715781852227007625u64 => {
                    let (): () = stardust_xr_wire::flex::deserialize(_data, _fds)?;
                    tracing::trace!(
                        "Got signal from server, {}::{}", "PanelItem",
                        "toplevel_move_request"
                    );
                    Ok(PanelItemEvent::ToplevelMoveRequest {
                    })
                }
                4540754955116125050u64 => {
                    let (up, down, left, right): (bool, bool, bool, bool) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? up, ? down, ? left, ? right, "Got signal from server, {}::{}",
                        "PanelItem", "toplevel_resize_request"
                    );
                    Ok(PanelItemEvent::ToplevelResizeRequest {
                        up: up,
                        down: down,
                        left: left,
                        right: right,
                    })
                }
                3665525014775618530u64 => {
                    let (size): (stardust_xr_wire::values::Vector2<u32>) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? size, "Got signal from server, {}::{}", "PanelItem",
                        "toplevel_size_changed"
                    );
                    Ok(PanelItemEvent::ToplevelSizeChanged {
                        size: size,
                    })
                }
                5830507316813672853u64 => {
                    let (child): (u64) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? child, "Got signal from server, {}::{}", "PanelItem",
                        "exclusive_keyboard_grab"
                    );
                    Ok(PanelItemEvent::ExclusiveKeyboardGrab {
                        child: child,
                    })
                }
                6092877811616586203u64 => {
                    let (geometry): (Geometry) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? geometry, "Got signal from server, {}::{}", "PanelItem",
                        "set_cursor"
                    );
                    Ok(PanelItemEvent::SetCursor {
                        geometry: geometry,
                    })
                }
                12365625385177885025u64 => {
                    let (): () = stardust_xr_wire::flex::deserialize(_data, _fds)?;
                    tracing::trace!(
                        "Got signal from server, {}::{}", "PanelItem", "hide_cursor"
                    );
                    Ok(PanelItemEvent::HideCursor {})
                }
                13878060402106144481u64 => {
                    let (uid, info): (u64, ChildInfo) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? uid, ? info, "Got signal from server, {}::{}", "PanelItem",
                        "create_child"
                    );
                    Ok(PanelItemEvent::CreateChild {
                        uid: uid,
                        info: info,
                    })
                }
                4614990113965355127u64 => {
                    let (uid, geometry): (u64, Geometry) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? uid, ? geometry, "Got signal from server, {}::{}", "PanelItem",
                        "reposition_child"
                    );
                    Ok(PanelItemEvent::RepositionChild {
                        uid: uid,
                        geometry: geometry,
                    })
                }
                7048616010698587017u64 => {
                    let (uid): (u64) = stardust_xr_wire::flex::deserialize(_data, _fds)?;
                    tracing::trace!(
                        ? uid, "Got signal from server, {}::{}", "PanelItem",
                        "destroy_child"
                    );
                    Ok(PanelItemEvent::DestroyChild {
                        uid: uid,
                    })
                }
                _ => Err(stardust_xr_wire::scenegraph::ScenegraphError::MemberNotFound),
            }
        }
        fn parse_method(
            _client: &std::sync::Arc<crate::panel_item::PanelItemHandle>,
            method_id: u64,
            _data: &[u8],
            _fds: Vec<std::os::fd::OwnedFd>,
            response: stardust_xr_wire::messenger::MethodResponse,
        ) -> Result<Self, stardust_xr_wire::scenegraph::ScenegraphError> {
            match method_id {
                _ => {
                    let _ = response
                        .send(
                            Err(
                                stardust_xr_wire::scenegraph::ScenegraphError::MemberNotFound,
                            ),
                        );
                    Err(stardust_xr_wire::scenegraph::ScenegraphError::MemberNotFound)
                }
            }
        }
    }
    ///An item that represents a toplevel 2D window's surface (base window) and all its children (context menus, modals, etc.).
    pub trait PanelItemAspect: crate::node::NodeType + std::fmt::Debug {
        fn recv_panel_item_event(&self) -> Option<PanelItemEvent>;
        ///Where the PanelItem will appear on disconnect.
        fn panel_item_output_spatial(
            &self,
            spaital_ref_uid: u64,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (u64) = (spaital_ref_uid);
            self.node()
                .send_signal(16007573185838633179u64, 6341428459395007981u64, &data)?;
            let (spaital_ref_uid) = data;
            tracing::trace!(
                ? spaital_ref_uid, "Sent signal to server, {}::{}", "PanelItem",
                "panel_item_output_spatial"
            );
            Ok(())
        }
        ///Send an event to set the pointer's position (in pixels, relative to top-left of surface). This will activate the pointer.
        fn absolute_pointer_motion(
            &self,
            surface: Option<SurfaceId>,
            position: stardust_xr_wire::values::Vector2<f32>,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (Option<SurfaceId>, stardust_xr_wire::values::Vector2<f32>) = (
                surface
                    .map(|o| Ok::<_, stardust_xr_fusion::node::NodeError>(o))
                    .transpose()?,
                position.into(),
            );
            self.node()
                .send_signal(16007573185838633179u64, 16749501366142443858u64, &data)?;
            let (surface, position) = data;
            tracing::trace!(
                ? surface, ? position, "Sent signal to server, {}::{}", "PanelItem",
                "absolute_pointer_motion"
            );
            Ok(())
        }
        ///Send an event that the pointer moved a relative amount (in pixels).
        fn relative_pointer_motion(
            &self,
            surface: Option<SurfaceId>,
            delta: stardust_xr_wire::values::Vector2<f32>,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (Option<SurfaceId>, stardust_xr_wire::values::Vector2<f32>) = (
                surface
                    .map(|o| Ok::<_, stardust_xr_fusion::node::NodeError>(o))
                    .transpose()?,
                delta.into(),
            );
            self.node()
                .send_signal(16007573185838633179u64, 8178111286759258039u64, &data)?;
            let (surface, delta) = data;
            tracing::trace!(
                ? surface, ? delta, "Sent signal to server, {}::{}", "PanelItem",
                "relative_pointer_motion"
            );
            Ok(())
        }
        ///Send an event to set a pointer button's state if the pointer's active. The `button` is from the `input_event_codes` crate (e.g. BTN_LEFT for left click).
        fn pointer_button(
            &self,
            surface: Option<SurfaceId>,
            button: u32,
            pressed: bool,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (Option<SurfaceId>, u32, bool) = (
                surface
                    .map(|o| Ok::<_, stardust_xr_fusion::node::NodeError>(o))
                    .transpose()?,
                button,
                pressed,
            );
            self.node()
                .send_signal(16007573185838633179u64, 1617963334017359776u64, &data)?;
            let (surface, button, pressed) = data;
            tracing::trace!(
                ? surface, ? button, ? pressed, "Sent signal to server, {}::{}",
                "PanelItem", "pointer_button"
            );
            Ok(())
        }
        /**Send an event to scroll the pointer if it's active.
Scroll distance is a value in pixels corresponding to the `distance` the surface should be scrolled.
Scroll steps is a value in columns/rows corresponding to the wheel clicks of a mouse or such. This also supports fractions of a wheel click.*/
        fn pointer_scroll(
            &self,
            surface: Option<SurfaceId>,
            scroll_distance: stardust_xr_wire::values::Vector2<f32>,
            scroll_steps: stardust_xr_wire::values::Vector2<f32>,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (
                Option<SurfaceId>,
                stardust_xr_wire::values::Vector2<f32>,
                stardust_xr_wire::values::Vector2<f32>,
            ) = (
                surface
                    .map(|o| Ok::<_, stardust_xr_fusion::node::NodeError>(o))
                    .transpose()?,
                scroll_distance.into(),
                scroll_steps.into(),
            );
            self.node()
                .send_signal(16007573185838633179u64, 18077910517219850499u64, &data)?;
            let (surface, scroll_distance, scroll_steps) = data;
            tracing::trace!(
                ? surface, ? scroll_distance, ? scroll_steps,
                "Sent signal to server, {}::{}", "PanelItem", "pointer_scroll"
            );
            Ok(())
        }
        ///Send an event to stop scrolling the pointer.
        fn pointer_stop_scroll(
            &self,
            surface: Option<SurfaceId>,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (Option<SurfaceId>) = (surface
                .map(|o| Ok::<_, stardust_xr_fusion::node::NodeError>(o))
                .transpose()?);
            self.node()
                .send_signal(16007573185838633179u64, 13177724628894942354u64, &data)?;
            let (surface) = data;
            tracing::trace!(
                ? surface, "Sent signal to server, {}::{}", "PanelItem",
                "pointer_stop_scroll"
            );
            Ok(())
        }
        ///Send a key press or release event with the given keymap ID.
        fn keyboard_key(
            &self,
            surface: Option<SurfaceId>,
            keymap_id: u64,
            key: u32,
            pressed: bool,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (Option<SurfaceId>, u64, u32, bool) = (
                surface
                    .map(|o| Ok::<_, stardust_xr_fusion::node::NodeError>(o))
                    .transpose()?,
                keymap_id,
                key,
                pressed,
            );
            self.node()
                .send_signal(16007573185838633179u64, 18230480350930328965u64, &data)?;
            let (surface, keymap_id, key, pressed) = data;
            tracing::trace!(
                ? surface, ? keymap_id, ? key, ? pressed,
                "Sent signal to server, {}::{}", "PanelItem", "keyboard_key"
            );
            Ok(())
        }
        ///Put a touch down on this surface with the unique ID `uid` at `position` (in pixels) from top left corner of the surface.
        fn touch_down(
            &self,
            surface: Option<SurfaceId>,
            uid: u32,
            position: stardust_xr_wire::values::Vector2<f32>,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (Option<SurfaceId>, u32, stardust_xr_wire::values::Vector2<f32>) = (
                surface
                    .map(|o| Ok::<_, stardust_xr_fusion::node::NodeError>(o))
                    .transpose()?,
                uid,
                position.into(),
            );
            self.node()
                .send_signal(16007573185838633179u64, 10543081656468919422u64, &data)?;
            let (surface, uid, position) = data;
            tracing::trace!(
                ? surface, ? uid, ? position, "Sent signal to server, {}::{}",
                "PanelItem", "touch_down"
            );
            Ok(())
        }
        ///Move an existing touch point.
        fn touch_move(
            &self,
            surface: Option<SurfaceId>,
            uid: u32,
            position: stardust_xr_wire::values::Vector2<f32>,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (Option<SurfaceId>, u32, stardust_xr_wire::values::Vector2<f32>) = (
                surface
                    .map(|o| Ok::<_, stardust_xr_fusion::node::NodeError>(o))
                    .transpose()?,
                uid,
                position.into(),
            );
            self.node()
                .send_signal(16007573185838633179u64, 15126475688563381777u64, &data)?;
            let (surface, uid, position) = data;
            tracing::trace!(
                ? surface, ? uid, ? position, "Sent signal to server, {}::{}",
                "PanelItem", "touch_move"
            );
            Ok(())
        }
        ///Release a touch from its surface.
        fn touch_up(
            &self,
            surface: Option<SurfaceId>,
            uid: u32,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (Option<SurfaceId>, u32) = (
                surface
                    .map(|o| Ok::<_, stardust_xr_fusion::node::NodeError>(o))
                    .transpose()?,
                uid,
            );
            self.node()
                .send_signal(16007573185838633179u64, 6589027081119653997u64, &data)?;
            let (surface, uid) = data;
            tracing::trace!(
                ? surface, ? uid, "Sent signal to server, {}::{}", "PanelItem",
                "touch_up"
            );
            Ok(())
        }
        ///Reset all input, such as pressed keys and pointer clicks and touches. Useful for when it's newly captured into an item acceptor to make sure no input gets stuck.
        fn reset_input(
            &self,
            surface: Option<SurfaceId>,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (Option<SurfaceId>) = (surface
                .map(|o| Ok::<_, stardust_xr_fusion::node::NodeError>(o))
                .transpose()?);
            self.node()
                .send_signal(16007573185838633179u64, 14629122800709746500u64, &data)?;
            let (surface) = data;
            tracing::trace!(
                ? surface, "Sent signal to server, {}::{}", "PanelItem", "reset_input"
            );
            Ok(())
        }
        ///Try to close the toplevel.
        fn close_toplevel(&self) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: () = ();
            self.node()
                .send_signal(16007573185838633179u64, 11149391162473273576u64, &data)?;
            let () = data;
            tracing::trace!(
                "Sent signal to server, {}::{}", "PanelItem", "close_toplevel"
            );
            Ok(())
        }
        ///Request a resize of the surface to whatever size the 2D app wants.
        fn auto_size_toplevel(&self) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: () = ();
            self.node()
                .send_signal(16007573185838633179u64, 7177229187692151305u64, &data)?;
            let () = data;
            tracing::trace!(
                "Sent signal to server, {}::{}", "PanelItem", "auto_size_toplevel"
            );
            Ok(())
        }
        ///Request a resize of the surface (in pixels).
        fn set_toplevel_size(
            &self,
            size: stardust_xr_wire::values::Vector2<u32>,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (stardust_xr_wire::values::Vector2<u32>) = (size.into());
            self.node()
                .send_signal(16007573185838633179u64, 8102855835344875634u64, &data)?;
            let (size) = data;
            tracing::trace!(
                ? size, "Sent signal to server, {}::{}", "PanelItem", "set_toplevel_size"
            );
            Ok(())
        }
        ///Tell the toplevel to appear focused visually if true, or unfocused if false.
        fn set_toplevel_focused_visuals(
            &self,
            focused: bool,
        ) -> stardust_xr_fusion::node::NodeResult<()> {
            let data: (bool) = (focused);
            self.node()
                .send_signal(16007573185838633179u64, 3934600665134956080u64, &data)?;
            let (focused) = data;
            tracing::trace!(
                ? focused, "Sent signal to server, {}::{}", "PanelItem",
                "set_toplevel_focused_visuals"
            );
            Ok(())
        }
    }
    ///Register a keymap UID with the server to easily identify it later
    pub async fn register_keymap(
        _client: &std::sync::Arc<crate::panel_item::PanelItemHandle>,
        keymap: String,
    ) -> stardust_xr_fusion::node::NodeResult<u64> {
        let data = (keymap);
        {
            let (keymap) = &data;
            tracing::trace!(
                ? keymap, "Called method on server, {}::{}", "Interface",
                "register_keymap"
            );
        }
        let (serialized_data, fds) = stardust_xr_wire::flex::serialize(&data)?;
        let (message, message_fds) = _client
            .message_sender_handle
            .method(12u64, 0u64, 13267771052011565359u64, &serialized_data, fds)
            .await?
            .map_err(|e| stardust_xr_fusion::node::NodeError::ReturnedError {
                e,
            })?
            .into_components();
        let result: u64 = stardust_xr_wire::flex::deserialize(&message, message_fds)?;
        let deserialized = result;
        tracing::trace!(
            "return" = ? deserialized, "Method return from server, {}::{}", "Interface",
            "register_keymap"
        );
        Ok(deserialized)
    }
    ///Get the keymap string representation from a UID
    pub async fn get_keymap(
        _client: &std::sync::Arc<crate::panel_item::PanelItemHandle>,
        keymap_id: u64,
    ) -> stardust_xr_fusion::node::NodeResult<String> {
        let data = (keymap_id);
        {
            let (keymap_id) = &data;
            tracing::trace!(
                ? keymap_id, "Called method on server, {}::{}", "Interface", "get_keymap"
            );
        }
        let (serialized_data, fds) = stardust_xr_wire::flex::serialize(&data)?;
        let (message, message_fds) = _client
            .message_sender_handle
            .method(12u64, 0u64, 18393315648981916968u64, &serialized_data, fds)
            .await?
            .map_err(|e| stardust_xr_fusion::node::NodeError::ReturnedError {
                e,
            })?
            .into_components();
        let result: String = stardust_xr_wire::flex::deserialize(&message, message_fds)?;
        let deserialized = result;
        tracing::trace!(
            "return" = ? deserialized, "Method return from server, {}::{}", "Interface",
            "get_keymap"
        );
        Ok(deserialized)
    }
}
