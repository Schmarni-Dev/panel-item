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
    pub enum OwnedEvent {
        SetEnabled { enabled: bool },
        Destroy {},
    }
    impl crate::scenegraph::EventParser for OwnedEvent {
        const ASPECT_ID: u64 = 15801764205032075891u64;
        fn parse_signal(
            _client: &std::sync::Arc<crate::panel_item::PanelItemHandle>,
            signal_id: u64,
            _data: &[u8],
            _fds: Vec<std::os::fd::OwnedFd>,
        ) -> Result<Self, stardust_xr_wire::scenegraph::ScenegraphError> {
            match signal_id {
                13365497663235993822u64 => {
                    let (enabled): (bool) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? enabled, "Got signal from server, {}::{}", "Owned",
                        "set_enabled"
                    );
                    Ok(OwnedEvent::SetEnabled {
                        enabled: enabled,
                    })
                }
                8637450960623370830u64 => {
                    let (): () = stardust_xr_wire::flex::deserialize(_data, _fds)?;
                    tracing::trace!(
                        "Got signal from server, {}::{}", "Owned", "destroy"
                    );
                    Ok(OwnedEvent::Destroy {})
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
        PanelItemOutputSpatial { spaital_ref_uid: u64 },
        AbsolutePointerMotion {
            surface: Option<SurfaceId>,
            position: stardust_xr_wire::values::Vector2<f32>,
        },
        RelativePointerMotion {
            surface: Option<SurfaceId>,
            delta: stardust_xr_wire::values::Vector2<f32>,
        },
        PointerButton { surface: Option<SurfaceId>, button: u32, pressed: bool },
        PointerScroll {
            surface: Option<SurfaceId>,
            scroll_distance: stardust_xr_wire::values::Vector2<f32>,
            scroll_steps: stardust_xr_wire::values::Vector2<f32>,
        },
        PointerStopScroll { surface: Option<SurfaceId> },
        KeyboardKey {
            surface: Option<SurfaceId>,
            keymap_id: u64,
            key: u32,
            pressed: bool,
        },
        TouchDown {
            surface: Option<SurfaceId>,
            uid: u32,
            position: stardust_xr_wire::values::Vector2<f32>,
        },
        TouchMove {
            surface: Option<SurfaceId>,
            uid: u32,
            position: stardust_xr_wire::values::Vector2<f32>,
        },
        TouchUp { surface: Option<SurfaceId>, uid: u32 },
        ResetInput { surface: Option<SurfaceId> },
        CloseToplevel {},
        AutoSizeToplevel {},
        SetToplevelSize { size: stardust_xr_wire::values::Vector2<u32> },
        SetToplevelFocusedVisuals { focused: bool },
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
                6341428459395007981u64 => {
                    let (spaital_ref_uid): (u64) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? spaital_ref_uid, "Got signal from server, {}::{}", "PanelItem",
                        "panel_item_output_spatial"
                    );
                    Ok(PanelItemEvent::PanelItemOutputSpatial {
                        spaital_ref_uid: spaital_ref_uid,
                    })
                }
                16749501366142443858u64 => {
                    let (
                        surface,
                        position,
                    ): (Option<SurfaceId>, stardust_xr_wire::values::Vector2<f32>) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? surface, ? position, "Got signal from server, {}::{}",
                        "PanelItem", "absolute_pointer_motion"
                    );
                    Ok(PanelItemEvent::AbsolutePointerMotion {
                        surface: surface,
                        position: position,
                    })
                }
                8178111286759258039u64 => {
                    let (
                        surface,
                        delta,
                    ): (Option<SurfaceId>, stardust_xr_wire::values::Vector2<f32>) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? surface, ? delta, "Got signal from server, {}::{}",
                        "PanelItem", "relative_pointer_motion"
                    );
                    Ok(PanelItemEvent::RelativePointerMotion {
                        surface: surface,
                        delta: delta,
                    })
                }
                1617963334017359776u64 => {
                    let (surface, button, pressed): (Option<SurfaceId>, u32, bool) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? surface, ? button, ? pressed, "Got signal from server, {}::{}",
                        "PanelItem", "pointer_button"
                    );
                    Ok(PanelItemEvent::PointerButton {
                        surface: surface,
                        button: button,
                        pressed: pressed,
                    })
                }
                18077910517219850499u64 => {
                    let (
                        surface,
                        scroll_distance,
                        scroll_steps,
                    ): (
                        Option<SurfaceId>,
                        stardust_xr_wire::values::Vector2<f32>,
                        stardust_xr_wire::values::Vector2<f32>,
                    ) = stardust_xr_wire::flex::deserialize(_data, _fds)?;
                    tracing::trace!(
                        ? surface, ? scroll_distance, ? scroll_steps,
                        "Got signal from server, {}::{}", "PanelItem", "pointer_scroll"
                    );
                    Ok(PanelItemEvent::PointerScroll {
                        surface: surface,
                        scroll_distance: scroll_distance,
                        scroll_steps: scroll_steps,
                    })
                }
                13177724628894942354u64 => {
                    let (surface): (Option<SurfaceId>) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? surface, "Got signal from server, {}::{}", "PanelItem",
                        "pointer_stop_scroll"
                    );
                    Ok(PanelItemEvent::PointerStopScroll {
                        surface: surface,
                    })
                }
                18230480350930328965u64 => {
                    let (
                        surface,
                        keymap_id,
                        key,
                        pressed,
                    ): (Option<SurfaceId>, u64, u32, bool) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? surface, ? keymap_id, ? key, ? pressed,
                        "Got signal from server, {}::{}", "PanelItem", "keyboard_key"
                    );
                    Ok(PanelItemEvent::KeyboardKey {
                        surface: surface,
                        keymap_id: keymap_id,
                        key: key,
                        pressed: pressed,
                    })
                }
                10543081656468919422u64 => {
                    let (
                        surface,
                        uid,
                        position,
                    ): (
                        Option<SurfaceId>,
                        u32,
                        stardust_xr_wire::values::Vector2<f32>,
                    ) = stardust_xr_wire::flex::deserialize(_data, _fds)?;
                    tracing::trace!(
                        ? surface, ? uid, ? position, "Got signal from server, {}::{}",
                        "PanelItem", "touch_down"
                    );
                    Ok(PanelItemEvent::TouchDown {
                        surface: surface,
                        uid: uid,
                        position: position,
                    })
                }
                15126475688563381777u64 => {
                    let (
                        surface,
                        uid,
                        position,
                    ): (
                        Option<SurfaceId>,
                        u32,
                        stardust_xr_wire::values::Vector2<f32>,
                    ) = stardust_xr_wire::flex::deserialize(_data, _fds)?;
                    tracing::trace!(
                        ? surface, ? uid, ? position, "Got signal from server, {}::{}",
                        "PanelItem", "touch_move"
                    );
                    Ok(PanelItemEvent::TouchMove {
                        surface: surface,
                        uid: uid,
                        position: position,
                    })
                }
                6589027081119653997u64 => {
                    let (surface, uid): (Option<SurfaceId>, u32) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? surface, ? uid, "Got signal from server, {}::{}", "PanelItem",
                        "touch_up"
                    );
                    Ok(PanelItemEvent::TouchUp {
                        surface: surface,
                        uid: uid,
                    })
                }
                14629122800709746500u64 => {
                    let (surface): (Option<SurfaceId>) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? surface, "Got signal from server, {}::{}", "PanelItem",
                        "reset_input"
                    );
                    Ok(PanelItemEvent::ResetInput {
                        surface: surface,
                    })
                }
                11149391162473273576u64 => {
                    let (): () = stardust_xr_wire::flex::deserialize(_data, _fds)?;
                    tracing::trace!(
                        "Got signal from server, {}::{}", "PanelItem", "close_toplevel"
                    );
                    Ok(PanelItemEvent::CloseToplevel {})
                }
                7177229187692151305u64 => {
                    let (): () = stardust_xr_wire::flex::deserialize(_data, _fds)?;
                    tracing::trace!(
                        "Got signal from server, {}::{}", "PanelItem",
                        "auto_size_toplevel"
                    );
                    Ok(PanelItemEvent::AutoSizeToplevel {
                    })
                }
                8102855835344875634u64 => {
                    let (size): (stardust_xr_wire::values::Vector2<u32>) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? size, "Got signal from server, {}::{}", "PanelItem",
                        "set_toplevel_size"
                    );
                    Ok(PanelItemEvent::SetToplevelSize {
                        size: size,
                    })
                }
                3934600665134956080u64 => {
                    let (focused): (bool) = stardust_xr_wire::flex::deserialize(
                        _data,
                        _fds,
                    )?;
                    tracing::trace!(
                        ? focused, "Got signal from server, {}::{}", "PanelItem",
                        "set_toplevel_focused_visuals"
                    );
                    Ok(PanelItemEvent::SetToplevelFocusedVisuals {
                        focused: focused,
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
            position: impl Into<stardust_xr_wire::values::Vector2<f32>>,
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
            delta: impl Into<stardust_xr_wire::values::Vector2<f32>>,
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
            scroll_distance: impl Into<stardust_xr_wire::values::Vector2<f32>>,
            scroll_steps: impl Into<stardust_xr_wire::values::Vector2<f32>>,
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
            position: impl Into<stardust_xr_wire::values::Vector2<f32>>,
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
            position: impl Into<stardust_xr_wire::values::Vector2<f32>>,
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
            size: impl Into<stardust_xr_wire::values::Vector2<u32>>,
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
        keymap: &str,
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
