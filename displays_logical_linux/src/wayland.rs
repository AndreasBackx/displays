use std::collections::{BTreeMap, BTreeSet};

use displays_logical_types::{
    LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState, LogicalDisplayUpdate,
};
use displays_types::{Orientation, Point};
use wayland_client::{
    event_created_child,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1::{self, ZwlrOutputConfigurationHeadV1},
    zwlr_output_configuration_v1::{self, ZwlrOutputConfigurationV1},
    zwlr_output_head_v1::{self, ZwlrOutputHeadV1},
    zwlr_output_manager_v1::{self, ZwlrOutputManagerV1},
    zwlr_output_mode_v1::{self, ZwlrOutputModeV1},
};

use crate::{
    error::{ApplyError, QueryError, WaylandError},
    logical_display_matches,
};

#[derive(Default)]
struct State {
    manager: Option<ZwlrOutputManagerV1>,
    heads: BTreeMap<u32, HeadState>,
    modes: BTreeMap<u32, ModeState>,
    head_proxies: BTreeMap<u32, ZwlrOutputHeadV1>,
    mode_proxies: BTreeMap<u32, ZwlrOutputModeV1>,
    done_serial: Option<u32>,
    config_status: Option<ConfigStatus>,
}

#[derive(Debug, Clone, Default)]
struct HeadState {
    name: Option<String>,
    description: Option<String>,
    make: Option<String>,
    model: Option<String>,
    serial_number: Option<String>,
    enabled: Option<bool>,
    x: Option<i32>,
    y: Option<i32>,
    transform: Option<Transform>,
    scale_milli: Option<u32>,
    current_mode: Option<u32>,
    modes: Vec<u32>,
    finished: bool,
}

#[derive(Debug, Clone, Default)]
struct ModeState {
    width: Option<i32>,
    height: Option<i32>,
    refresh: Option<i32>,
    preferred: bool,
    finished: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigStatus {
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Transform {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

impl Transform {
    fn orientation(self) -> Orientation {
        match self {
            Self::Normal | Self::Flipped => Orientation::Landscape,
            Self::Rotate90 | Self::Flipped90 => Orientation::Portrait,
            Self::Rotate180 | Self::Flipped180 => Orientation::LandscapeFlipped,
            Self::Rotate270 | Self::Flipped270 => Orientation::PortraitFlipped,
        }
    }

    fn swaps_dimensions(self) -> bool {
        matches!(
            self,
            Self::Rotate90 | Self::Rotate270 | Self::Flipped90 | Self::Flipped270
        )
    }
}

pub fn query() -> Result<BTreeSet<LogicalDisplay>, QueryError> {
    let snapshot = Snapshot::load()?;
    Ok(snapshot.into_logical_displays())
}

pub fn apply(
    updates: Vec<LogicalDisplayUpdate>,
    validate: bool,
) -> Result<Vec<LogicalDisplayUpdate>, ApplyError> {
    if updates.is_empty() {
        return Ok(updates);
    }

    let mut snapshot = Snapshot::load()?;
    let mut remaining_updates = updates.clone();
    let serial = snapshot.done_serial.ok_or_else(|| protocol_error("zwlr_output_manager_v1 did not emit a done serial"))?;
    let manager = snapshot.manager.clone().ok_or(WaylandError::MissingOutputManager)?;

    let mut event_queue = snapshot
        .event_queue
        .take()
        .ok_or_else(|| protocol_error("missing event queue"))?;
    let qh = event_queue.handle();

    let configuration = manager.create_configuration(serial, &qh, ());
    let heads = snapshot.heads.clone();
    let modes = snapshot.modes.clone();

    for (head_id, head) in heads {
        let head_proxy = snapshot
            .head_proxies
            .get(&head_id)
            .cloned()
            .ok_or_else(|| protocol_error(format!("missing head proxy for id {head_id}")))?;

        let logical_display = snapshot
            .head_to_display(head_id)
            .ok_or_else(|| protocol_error(format!("failed to convert head {head_id} to logical display")))?;

        let matching_index = remaining_updates
            .iter()
            .position(|update| logical_display_matches(&logical_display, &update.id));

        let update = matching_index.and_then(|index| remaining_updates.get(index).cloned());
        if desired_enabled(update.as_ref(), &head) {
            let config_head = configuration.enable_head(&head_proxy, &qh, ());

            if let Some(update) = update.as_ref() {
                apply_head_update(&config_head, update, &head, &snapshot.mode_proxies, &modes)?;
            } else {
                apply_current_head_state(&config_head, &head, &snapshot.mode_proxies, &modes)?;
            }
        } else {
            configuration.disable_head(&head_proxy);
        }

        if let Some(index) = matching_index {
            remaining_updates.remove(index);
        }
    }

    if validate {
        configuration.test();
    } else {
        configuration.apply();
    }

    loop {
        event_queue
            .blocking_dispatch(&mut snapshot.state)
            .map_err(|err| protocol_error(err.to_string()))?;

        match snapshot.state.config_status {
            Some(ConfigStatus::Succeeded) => return Ok(remaining_updates),
            Some(ConfigStatus::Failed) => return Err(ApplyError::Rejected),
            Some(ConfigStatus::Cancelled) => return Err(ApplyError::Cancelled),
            None => {}
        }
    }
}

struct Snapshot {
    state: State,
    manager: Option<ZwlrOutputManagerV1>,
    head_proxies: BTreeMap<u32, ZwlrOutputHeadV1>,
    mode_proxies: BTreeMap<u32, ZwlrOutputModeV1>,
    heads: BTreeMap<u32, HeadState>,
    modes: BTreeMap<u32, ModeState>,
    done_serial: Option<u32>,
    event_queue: Option<EventQueue<State>>,
}

impl Snapshot {
    fn load() -> Result<Self, WaylandError> {
        require_wayland_display().map_err(|_| WaylandError::MissingWaylandDisplay)?;
        let connection = Connection::connect_to_env().map_err(|err| WaylandError::Connect {
            message: err.to_string(),
        })?;
        let (globals, mut event_queue) = registry_queue_init::<State>(&connection).map_err(|err| {
            WaylandError::Connect {
                message: err.to_string(),
            }
        })?;
        let qh = event_queue.handle();
        let mut state = State::default();

        let manager = globals
            .bind::<ZwlrOutputManagerV1, _, _>(&qh, 1..=4, ())
            .map_err(|_| WaylandError::MissingOutputManager)?;
        state.manager = Some(manager.clone());

        loop {
            event_queue
                .blocking_dispatch(&mut state)
                .map_err(|err| WaylandError::Protocol {
                    message: err.to_string(),
                })?;

            if state.done_serial.is_some() {
                break;
            }
        }

        Ok(Self {
            manager: state.manager.clone(),
            head_proxies: state.head_proxies.clone(),
            mode_proxies: state.mode_proxies.clone(),
            heads: state.heads.clone(),
            modes: state.modes.clone(),
            done_serial: state.done_serial,
            event_queue: Some(event_queue),
            state,
        })
    }

    fn into_logical_displays(self) -> BTreeSet<LogicalDisplay> {
        self.heads
            .keys()
            .filter_map(|head_id| self.head_to_display(*head_id))
            .collect()
    }

    fn head_to_display(&self, head_id: u32) -> Option<LogicalDisplay> {
        let head = self.heads.get(&head_id)?;
        let connector_name = head.name.clone();
        let name = friendly_head_name(head)
            .or_else(|| connector_name.clone())
            .unwrap_or_else(|| format!("wayland-head-{head_id}"));
        let transform = head.transform.unwrap_or(Transform::Normal);
        let logical_size = head
            .current_mode
            .and_then(|mode_id| self.modes.get(&mode_id))
            .and_then(|mode| logical_size(mode, transform, head.scale_milli.unwrap_or(1000)));

        Some(LogicalDisplay {
            metadata: LogicalDisplayMetadata {
                name: name.clone(),
                path: format!(
                    "wayland:wlr:{}",
                    connector_name.unwrap_or_else(|| name.clone())
                ),
                manufacturer: head.make.clone(),
                model: head.model.clone(),
                serial_number: head.serial_number.clone(),
            },
            state: LogicalDisplayState {
                is_enabled: head.enabled.unwrap_or(false),
                orientation: transform.orientation(),
                width: logical_size.map(|(width, _)| width),
                height: logical_size.map(|(_, height)| height),
                pixel_format: None,
                position: match (head.x, head.y) {
                    (Some(x), Some(y)) => Some(Point { x, y }),
                    _ => None,
                },
            },
        })
    }
}

fn friendly_head_name(head: &HeadState) -> Option<String> {
    let description = head
        .description
        .as_deref()
        .filter(|description| !description.is_empty());

    match (head.make.as_deref(), head.model.as_deref()) {
        (_, Some(model)) if description == Some(model) => Some(model.to_string()),
        (Some(make), Some(model)) if !make.is_empty() && !model.is_empty() => {
            Some(format!("{make} {model}"))
        }
        (_, Some(model)) if !model.is_empty() => Some(model.to_string()),
        _ => description.map(ToString::to_string),
    }
}

fn desired_enabled(update: Option<&LogicalDisplayUpdate>, head: &HeadState) -> bool {
    update
        .and_then(|update| update.content.is_enabled)
        .unwrap_or(head.enabled.unwrap_or(false))
}

fn apply_current_head_state(
    config_head: &ZwlrOutputConfigurationHeadV1,
    head: &HeadState,
    mode_proxies: &BTreeMap<u32, ZwlrOutputModeV1>,
    _modes: &BTreeMap<u32, ModeState>,
) -> Result<(), ApplyError> {
    if let Some(mode_id) = head.current_mode {
        let mode = mode_proxies
            .get(&mode_id)
            .ok_or_else(|| protocol_error(format!("missing mode proxy for id {mode_id}")))?;
        config_head.set_mode(mode);
    }
    if let (Some(x), Some(y)) = (head.x, head.y) {
        config_head.set_position(x, y);
    }
    if let Some(transform) = head.transform {
        config_head.set_transform(transform_to_raw(transform));
    }
    if let Some(scale_milli) = head.scale_milli {
        config_head.set_scale(scale_milli as f64 / 1000.0);
    }
    Ok(())
}

fn apply_head_update(
    config_head: &ZwlrOutputConfigurationHeadV1,
    update: &LogicalDisplayUpdate,
    head: &HeadState,
    mode_proxies: &BTreeMap<u32, ZwlrOutputModeV1>,
    modes: &BTreeMap<u32, ModeState>,
) -> Result<(), ApplyError> {
    if let Some(position) = update.content.position.as_ref() {
        config_head.set_position(position.x, position.y);
    } else if let (Some(x), Some(y)) = (head.x, head.y) {
        config_head.set_position(x, y);
    }

    let transform = update
        .content
        .orientation
        .clone()
        .map(orientation_to_transform)
        .unwrap_or(head.transform.unwrap_or(Transform::Normal));
    config_head.set_transform(transform_to_raw(transform));

    let requested_size = match (update.content.width, update.content.height) {
        (Some(width), Some(height)) => Some((width, height)),
        (None, None) => None,
        _ => return Err(ApplyError::UnsupportedLogicalSize),
    };

    let (mode_id, scale_milli) = if let Some((width, height)) = requested_size {
        find_exact_mode_and_scale(width, height, transform, head, modes)
            .ok_or(ApplyError::UnsupportedLogicalSize)?
    } else {
        (
            head.current_mode
                .ok_or_else(|| protocol_error("enabled output has no current mode"))?,
            head.scale_milli.unwrap_or(1000),
        )
    };

    let mode = mode_proxies
        .get(&mode_id)
        .ok_or_else(|| protocol_error(format!("missing mode proxy for id {mode_id}")))?;
    config_head.set_mode(mode);
    config_head.set_scale(scale_milli as f64 / 1000.0);

    Ok(())
}

fn find_exact_mode_and_scale(
    width: u32,
    height: u32,
    transform: Transform,
    head: &HeadState,
    modes: &BTreeMap<u32, ModeState>,
) -> Option<(u32, u32)> {
    let current_scale = head.scale_milli.unwrap_or(1000);
    head.modes.iter().find_map(|mode_id| {
        let mode = modes.get(mode_id)?;
        let logical = logical_size(mode, transform, current_scale)?;
        if logical == (width, height) {
            Some((*mode_id, current_scale))
        } else {
            None
        }
    })
}

fn logical_size(mode: &ModeState, transform: Transform, scale_milli: u32) -> Option<(u32, u32)> {
    let mut width = mode.width? as u32;
    let mut height = mode.height? as u32;
    if transform.swaps_dimensions() {
        std::mem::swap(&mut width, &mut height);
    }

    if scale_milli == 0 {
        return None;
    }
    if width.checked_mul(1000)? % scale_milli != 0 || height.checked_mul(1000)? % scale_milli != 0 {
        return None;
    }

    Some((width * 1000 / scale_milli, height * 1000 / scale_milli))
}

fn orientation_to_transform(value: Orientation) -> Transform {
    match value {
        Orientation::Landscape => Transform::Normal,
        Orientation::Portrait => Transform::Rotate90,
        Orientation::LandscapeFlipped => Transform::Rotate180,
        Orientation::PortraitFlipped => Transform::Rotate270,
    }
}

fn transform_to_raw(value: Transform) -> wl_output::Transform {
    match value {
        Transform::Normal => wl_output::Transform::Normal,
        Transform::Rotate90 => wl_output::Transform::_90,
        Transform::Rotate180 => wl_output::Transform::_180,
        Transform::Rotate270 => wl_output::Transform::_270,
        Transform::Flipped => wl_output::Transform::Flipped,
        Transform::Flipped90 => wl_output::Transform::Flipped90,
        Transform::Flipped180 => wl_output::Transform::Flipped180,
        Transform::Flipped270 => wl_output::Transform::Flipped270,
    }
}

fn protocol_error(message: impl Into<String>) -> WaylandError {
    WaylandError::Protocol {
        message: message.into(),
    }
}

fn require_wayland_display() -> Result<String, ()> {
    std::env::var("WAYLAND_DISPLAY").map_err(|_| ())
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrOutputManagerV1, ()> for State {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrOutputManagerV1,
        event: zwlr_output_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_output_manager_v1::Event::Head { head } => {
                let id = head.id().protocol_id();
                state.heads.entry(id).or_default();
                state.head_proxies.insert(id, head);
            }
            zwlr_output_manager_v1::Event::Done { serial } => {
                state.done_serial = Some(serial);
            }
            zwlr_output_manager_v1::Event::Finished => {}
            _ => {}
        }
    }

    event_created_child!(State, ZwlrOutputManagerV1, [
        zwlr_output_manager_v1::EVT_HEAD_OPCODE => (ZwlrOutputHeadV1, ())
    ]);
}

impl Dispatch<ZwlrOutputHeadV1, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &ZwlrOutputHeadV1,
        event: zwlr_output_head_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let head = state.heads.entry(proxy.id().protocol_id()).or_default();
        match event {
            zwlr_output_head_v1::Event::Name { name } => head.name = Some(name),
            zwlr_output_head_v1::Event::Description { description } => {
                head.description = Some(description)
            }
            zwlr_output_head_v1::Event::Make { make } => head.make = Some(make),
            zwlr_output_head_v1::Event::Model { model } => head.model = Some(model),
            zwlr_output_head_v1::Event::SerialNumber { serial_number } => {
                head.serial_number = Some(serial_number)
            }
            zwlr_output_head_v1::Event::Enabled { enabled } => head.enabled = Some(enabled != 0),
            zwlr_output_head_v1::Event::Position { x, y } => {
                head.x = Some(x);
                head.y = Some(y);
            }
            zwlr_output_head_v1::Event::Transform { transform } => {
                head.transform = Some(match transform {
                    WEnum::Value(wl_output::Transform::Normal) => Transform::Normal,
                    WEnum::Value(wl_output::Transform::_90) => Transform::Rotate90,
                    WEnum::Value(wl_output::Transform::_180) => Transform::Rotate180,
                    WEnum::Value(wl_output::Transform::_270) => Transform::Rotate270,
                    WEnum::Value(wl_output::Transform::Flipped) => Transform::Flipped,
                    WEnum::Value(wl_output::Transform::Flipped90) => Transform::Flipped90,
                    WEnum::Value(wl_output::Transform::Flipped180) => Transform::Flipped180,
                    WEnum::Value(wl_output::Transform::Flipped270) => Transform::Flipped270,
                    _ => Transform::Normal,
                })
            }
            zwlr_output_head_v1::Event::Scale { scale } => {
                head.scale_milli = Some((scale * 1000.0).round() as u32)
            }
            zwlr_output_head_v1::Event::Mode { mode } => {
                let mode_id = mode.id().protocol_id();
                head.modes.push(mode_id);
                state.modes.entry(mode_id).or_default();
                state.mode_proxies.insert(mode_id, mode);
            }
            zwlr_output_head_v1::Event::CurrentMode { mode } => {
                head.current_mode = Some(mode.id().protocol_id())
            }
            zwlr_output_head_v1::Event::Finished => head.finished = true,
            _ => {}
        }
    }

    event_created_child!(State, ZwlrOutputHeadV1, [
        zwlr_output_head_v1::EVT_MODE_OPCODE => (ZwlrOutputModeV1, ())
    ]);
}

impl Dispatch<ZwlrOutputModeV1, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &ZwlrOutputModeV1,
        event: zwlr_output_mode_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let mode = state.modes.entry(proxy.id().protocol_id()).or_default();
        match event {
            zwlr_output_mode_v1::Event::Size { width, height } => {
                mode.width = Some(width);
                mode.height = Some(height);
            }
            zwlr_output_mode_v1::Event::Refresh { refresh } => mode.refresh = Some(refresh),
            zwlr_output_mode_v1::Event::Preferred => mode.preferred = true,
            zwlr_output_mode_v1::Event::Finished => mode.finished = true,
            _ => {}
        }
    }
}

impl Dispatch<ZwlrOutputConfigurationV1, ()> for State {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrOutputConfigurationV1,
        event: zwlr_output_configuration_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_output_configuration_v1::Event::Succeeded => {
                state.config_status = Some(ConfigStatus::Succeeded)
            }
            zwlr_output_configuration_v1::Event::Failed => {
                state.config_status = Some(ConfigStatus::Failed)
            }
            zwlr_output_configuration_v1::Event::Cancelled => {
                state.config_status = Some(ConfigStatus::Cancelled)
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwlrOutputConfigurationHeadV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrOutputConfigurationHeadV1,
        _event: zwlr_output_configuration_head_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}
