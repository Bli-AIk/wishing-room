use std::time::{Duration, Instant};

use dioxus::prelude::*;
use taled_core::ObjectShape;

#[cfg(target_os = "android")]
use crate::platform::log;
use crate::{
    app_state::{
        ActiveTouchPointer, AppState, PinchGesture, ShapeFillPreview, SingleTouchGesture,
        TileSelectionHandle, TileSelectionRegion, Tool,
    },
    edit_ops::{
        apply_cell_tool, apply_shape_fill_rect, place_tile_selection_transfer, select_tile_region,
    },
};

const LONG_PRESS_DURATION: Duration = Duration::from_millis(260);
const SYNTHETIC_CLICK_SUPPRESSION: Duration = Duration::from_millis(450);
const MIN_PINCH_DISTANCE: f64 = 12.0;
const TILE_SELECTION_HANDLE_INTERIOR_BIAS: f64 = 0.5;
const TILE_SELECTION_HANDLE_HIT_RADIUS: f64 = 18.0;

pub(crate) fn should_ignore_synthetic_click(state: &mut AppState) -> bool {
    let Some(deadline) = state.suppress_click_until else {
        return false;
    };
    if Instant::now() <= deadline {
        true
    } else {
        state.suppress_click_until = None;
        false
    }
}

pub(crate) fn handle_touch_pointer_down(state: &mut AppState, event: Event<PointerData>) {
    if event.pointer_type() != "touch" {
        return;
    }
    event.prevent_default();
    suppress_synthetic_click(state);

    let point = touch_surface_point(state, &event);
    upsert_touch_point(state, event.pointer_id(), point.x, point.y);
    log_touch_probe(state, &event, "down", point.x, point.y);

    if state.active_touch_points.len() >= 2 {
        finish_touch_edit_batch(state);
        state.single_touch_gesture = None;
        state.shape_fill_preview = None;
        state.tile_selection_preview = None;
        initialize_pinch_gesture(state);
        return;
    }

    state.pinch_gesture = None;
    start_touch_edit_batch(state);
    let selection_resize_handle = if selects_tile_region(state) {
        if state.tile_selection_transfer.is_some() {
            None
        } else {
            selection_resize_handle_from_surface(state, point.x, point.y)
        }
    } else {
        None
    };
    let hit_cell = cell_from_surface(state, point.x, point.y);
    let selection_move_drag_offset = if state.tile_selection_transfer.is_some() {
        state.tile_selection.and_then(|selection| {
            hit_cell.map(|cell| selection_drag_offset(selection, cell))
        })
    } else {
        None
    };
    let anchor_cell = selection_resize_handle
        .and_then(|handle| {
            state
                .tile_selection
                .map(|selection| selection_resize_anchor_cell(selection, handle))
        })
        .or_else(|| {
            if state.tile_selection_transfer.is_some() {
                hit_cell.and_then(|cell| selection_move_origin_from_cell(state, cell))
            } else {
                None
            }
        })
        .or(hit_cell);
    state.single_touch_gesture = Some(SingleTouchGesture {
        pointer_id: event.pointer_id(),
        started_at: Instant::now(),
        drag_active: false,
        anchor_cell,
        resize_handle: selection_resize_handle,
        selection_move_drag_offset,
        last_applied_cell: None,
        last_surface_x: point.x,
        last_surface_y: point.y,
    });
    state.shape_fill_preview = if state.tool == Tool::ShapeFill {
        anchor_cell.map(|cell| ShapeFillPreview {
            start_cell: cell,
            end_cell: cell,
        })
    } else {
        None
    };
    state.tile_selection_preview = if selects_tile_region(state) && state.tile_selection_transfer.is_none()
    {
        anchor_cell.map(|cell| TileSelectionRegion {
            start_cell: cell,
            end_cell: cell,
        })
    } else {
        None
    };
    if state.tile_selection_transfer.is_some()
        && let Some(selection) = anchor_cell.map(|origin| selection_from_origin(state, origin))
    {
        state.tile_selection = Some(selection);
    }
}

pub(crate) fn handle_touch_pointer_move(state: &mut AppState, event: Event<PointerData>) {
    if event.pointer_type() != "touch" {
        return;
    }
    event.prevent_default();
    suppress_synthetic_click(state);

    let point = touch_surface_point(state, &event);
    upsert_touch_point(state, event.pointer_id(), point.x, point.y);

    if state.active_touch_points.len() >= 2 {
        update_pinch_gesture(state);
        return;
    }

    if state.tool == Tool::Hand {
        let (delta_x, delta_y) = {
            let Some(gesture) = state.single_touch_gesture.as_mut() else {
                return;
            };
            if gesture.pointer_id != event.pointer_id() {
                return;
            }

            let delta_x = point.x - gesture.last_surface_x;
            let delta_y = point.y - gesture.last_surface_y;
            gesture.last_surface_x = point.x;
            gesture.last_surface_y = point.y;
            gesture.drag_active = true;
            (delta_x, delta_y)
        };

        if delta_x.abs() >= 0.5 || delta_y.abs() >= 0.5 {
            state.pan_x += delta_x.round() as i32;
            state.pan_y += delta_y.round() as i32;
            log_touch_resolution(state, "hand-pan", point.x, point.y);
        }
        return;
    }

    if selects_tile_region(state) && state.tile_selection_transfer.is_some() {
        let hit_cell = clamped_cell_from_surface(state, point.x, point.y);
        let drag_offset = state
            .single_touch_gesture
            .as_ref()
            .and_then(|gesture| {
                (gesture.pointer_id == event.pointer_id()).then_some(gesture.selection_move_drag_offset)
            })
            .flatten();
        let Some(hit_cell) = hit_cell else {
            return;
        };
        let origin = selection_move_origin_from_hit(state, hit_cell, drag_offset);
        let Some(gesture) = state.single_touch_gesture.as_mut() else {
            return;
        };
        if gesture.pointer_id != event.pointer_id() {
            return;
        }
        gesture.drag_active = true;
        state.tile_selection = Some(selection_from_origin(state, origin));
        return;
    }

    if selects_tile_region(state) {
        let resize_handle = state
            .single_touch_gesture
            .as_ref()
            .and_then(|gesture| gesture.resize_handle);
        let hit_cell = selection_end_cell_from_surface(state, point.x, point.y, resize_handle);
        let Some(gesture) = state.single_touch_gesture.as_mut() else {
            return;
        };
        if gesture.pointer_id != event.pointer_id() {
            return;
        }
        if hit_cell.is_some() {
            gesture.drag_active = true;
        }
        state.tile_selection_preview = match (gesture.anchor_cell, hit_cell) {
            (Some(start_cell), Some(end_cell)) => Some(TileSelectionRegion {
                start_cell,
                end_cell,
            }),
            (Some(start_cell), None) => Some(TileSelectionRegion {
                start_cell,
                end_cell: start_cell,
            }),
            _ => None,
        };
        return;
    }

    if state.tool == Tool::ShapeFill {
        let hit_cell = clamped_cell_from_surface(state, point.x, point.y);
        let Some(gesture) = state.single_touch_gesture.as_mut() else {
            return;
        };
        if gesture.pointer_id != event.pointer_id() {
            return;
        }
        if hit_cell.is_some() {
            gesture.drag_active = true;
        }
        state.shape_fill_preview = match (gesture.anchor_cell, hit_cell) {
            (Some(start_cell), Some(end_cell)) => Some(ShapeFillPreview {
                start_cell,
                end_cell,
            }),
            (Some(start_cell), None) => Some(ShapeFillPreview {
                start_cell,
                end_cell: start_cell,
            }),
            _ => None,
        };
        return;
    }

    if !tool_supports_drag(state.tool) {
        return;
    }

    let cell = cell_from_surface(state, point.x, point.y);
    let should_apply = {
        let Some(gesture) = state.single_touch_gesture.as_mut() else {
            return;
        };
        if gesture.pointer_id != event.pointer_id() {
            return;
        }
        if !gesture.drag_active && gesture.started_at.elapsed() < LONG_PRESS_DURATION {
            return;
        }
        let Some(cell) = cell else {
            return;
        };
        if gesture.last_applied_cell == Some(cell) {
            false
        } else {
            gesture.drag_active = true;
            gesture.last_applied_cell = Some(cell);
            true
        }
    };

    if should_apply {
        apply_touch_tool(state, point.x, point.y, None, None);
    }
}

pub(crate) fn handle_touch_pointer_up(state: &mut AppState, event: Event<PointerData>) {
    if event.pointer_type() != "touch" {
        return;
    }
    event.prevent_default();
    suppress_synthetic_click(state);

    let point = touch_surface_point(state, &event);
    let should_apply = finalize_single_touch_if_needed(state, event.pointer_id(), point.x, point.y);
    let (anchor_cell, resize_handle) = state
        .single_touch_gesture
        .as_ref()
        .map(|gesture| (gesture.anchor_cell, gesture.resize_handle))
        .unwrap_or((None, None));
    log_touch_probe(state, &event, "up", point.x, point.y);

    remove_touch_point(state, event.pointer_id());
    if state.active_touch_points.len() < 2 {
        state.pinch_gesture = None;
    }
    state.single_touch_gesture = None;
    state.shape_fill_preview = None;
    state.tile_selection_preview = None;

    if should_apply {
        apply_touch_tool(state, point.x, point.y, anchor_cell, resize_handle);
    }

    finish_touch_edit_batch(state);
}

pub(crate) fn handle_touch_pointer_cancel(state: &mut AppState, event: Event<PointerData>) {
    if event.pointer_type() != "touch" {
        return;
    }
    event.prevent_default();
    suppress_synthetic_click(state);
    remove_touch_point(state, event.pointer_id());
    state.single_touch_gesture = None;
    state.shape_fill_preview = None;
    state.tile_selection_preview = None;
    if state.active_touch_points.len() < 2 {
        state.pinch_gesture = None;
    }
    abort_touch_edit_batch(state);
}

fn finalize_single_touch_if_needed(state: &mut AppState, pointer_id: i32, x: f64, y: f64) -> bool {
    if state.tool == Tool::Hand {
        return false;
    }

    let Some(gesture) = state.single_touch_gesture.clone() else {
        return false;
    };
    if gesture.pointer_id != pointer_id {
        return false;
    }
    if selects_tile_region(state) {
        if state.tile_selection_transfer.is_some() {
            return state.tile_selection.is_some();
        }
        return gesture.anchor_cell.is_some()
            && selection_end_cell_from_surface(state, x, y, gesture.resize_handle).is_some();
    }
    if state.tool == Tool::ShapeFill {
        return gesture.anchor_cell.is_some() && clamped_cell_from_surface(state, x, y).is_some();
    }
    if gesture.drag_active {
        if !tool_supports_drag(state.tool) {
            return false;
        }
        let Some(cell) = cell_from_surface(state, x, y) else {
            return false;
        };
        return gesture.last_applied_cell != Some(cell);
    }
    true
}

fn apply_touch_tool(
    state: &mut AppState,
    x: f64,
    y: f64,
    anchor_cell: Option<(u32, u32)>,
    resize_handle: Option<TileSelectionHandle>,
) {
    log_touch_resolution(state, "apply", x, y);
    match state.tool {
        Tool::Hand => {}
        Tool::Select => {
            if state.tile_selection_transfer.is_some() {
                place_tile_selection_transfer(state);
                return;
            }
            if selects_tile_region(state) {
                let Some((end_x, end_y)) =
                    selection_end_cell_from_surface(state, x, y, resize_handle)
                else {
                    return;
                };
                let (start_x, start_y) = anchor_cell.unwrap_or((end_x, end_y));
                select_tile_region(state, start_x, start_y, end_x, end_y);
            } else {
                select_at_point(state, x, y);
            }
        }
        Tool::ShapeFill => {
            let Some((end_x, end_y)) = clamped_cell_from_surface(state, x, y) else {
                return;
            };
            let (start_x, start_y) = anchor_cell.unwrap_or((end_x, end_y));
            state.selected_cell = Some((end_x, end_y));
            apply_shape_fill_rect(state, start_x, start_y, end_x, end_y);
        }
        _ => {
            let Some((cell_x, cell_y)) = cell_from_surface(state, x, y) else {
                return;
            };
            apply_cell_tool(state, cell_x, cell_y);
        }
    }
}

fn select_at_point(state: &mut AppState, x: f64, y: f64) {
    let Some((world_x, world_y)) = world_coordinates_from_surface(state, x, y) else {
        state.selected_cell = None;
        state.selected_object = None;
        state.tile_selection = None;
        state.tile_selection_preview = None;
        return;
    };

    if let Some((layer_index, object_id)) = hit_test_object(state, world_x, world_y) {
        state.active_layer = layer_index;
        state.selected_object = Some(object_id);
        state.selected_cell = cell_from_surface(state, x, y);
        state.tile_selection = None;
        state.tile_selection_preview = None;
        state.status = format!("Selected object {object_id}.");
        return;
    }

    state.selected_object = None;
    state.tile_selection = None;
    state.tile_selection_preview = None;
    state.selected_cell = cell_from_surface(state, x, y);
    if let Some((cell_x, cell_y)) = state.selected_cell {
        state.status = format!("Selected cell ({cell_x}, {cell_y}).");
    }
}

fn selects_tile_region(state: &AppState) -> bool {
    state.tool == Tool::Select
        && state
            .session
            .as_ref()
            .and_then(|session| session.document().map.layer(state.active_layer))
            .is_some_and(|layer| layer.as_tile().is_some())
}

fn selection_resize_handle_from_surface(
    state: &AppState,
    surface_x: f64,
    surface_y: f64,
) -> Option<TileSelectionHandle> {
    let selection = state.tile_selection?;
    let session = state.session.as_ref()?;
    let map = &session.document().map;
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    let zoom = f64::from(state.zoom_percent) / 100.0;
    let pan_x = f64::from(state.pan_x);
    let pan_y = f64::from(state.pan_y);
    let corners = [
        (
            TileSelectionHandle::TopLeft,
            pan_x + f64::from(min_x * map.tile_width) * zoom,
            pan_y + f64::from(min_y * map.tile_height) * zoom,
        ),
        (
            TileSelectionHandle::TopRight,
            pan_x + f64::from((max_x + 1) * map.tile_width) * zoom,
            pan_y + f64::from(min_y * map.tile_height) * zoom,
        ),
        (
            TileSelectionHandle::BottomLeft,
            pan_x + f64::from(min_x * map.tile_width) * zoom,
            pan_y + f64::from((max_y + 1) * map.tile_height) * zoom,
        ),
        (
            TileSelectionHandle::BottomRight,
            pan_x + f64::from((max_x + 1) * map.tile_width) * zoom,
            pan_y + f64::from((max_y + 1) * map.tile_height) * zoom,
        ),
    ];

    corners
        .into_iter()
        .find(|(_, handle_x, handle_y)| {
            (surface_x - handle_x).abs() <= TILE_SELECTION_HANDLE_HIT_RADIUS
                && (surface_y - handle_y).abs() <= TILE_SELECTION_HANDLE_HIT_RADIUS
        })
        .map(|(handle, _, _)| handle)
}

fn selection_end_cell_from_surface(
    state: &AppState,
    x: f64,
    y: f64,
    resize_handle: Option<TileSelectionHandle>,
) -> Option<(u32, u32)> {
    let adjusted_point = adjusted_selection_resize_surface_point(x, y, resize_handle);

    clamped_cell_from_surface(state, adjusted_point.0, adjusted_point.1)
}

fn adjusted_selection_resize_surface_point(
    x: f64,
    y: f64,
    resize_handle: Option<TileSelectionHandle>,
) -> (f64, f64) {
    match resize_handle {
        Some(TileSelectionHandle::TopLeft) => (
            x + TILE_SELECTION_HANDLE_INTERIOR_BIAS,
            y + TILE_SELECTION_HANDLE_INTERIOR_BIAS,
        ),
        Some(TileSelectionHandle::TopRight) => (
            x - TILE_SELECTION_HANDLE_INTERIOR_BIAS,
            y + TILE_SELECTION_HANDLE_INTERIOR_BIAS,
        ),
        Some(TileSelectionHandle::BottomLeft) => (
            x + TILE_SELECTION_HANDLE_INTERIOR_BIAS,
            y - TILE_SELECTION_HANDLE_INTERIOR_BIAS,
        ),
        Some(TileSelectionHandle::BottomRight) => (
            x - TILE_SELECTION_HANDLE_INTERIOR_BIAS,
            y - TILE_SELECTION_HANDLE_INTERIOR_BIAS,
        ),
        None => (x, y),
    }
}

fn selection_resize_anchor_cell(
    selection: TileSelectionRegion,
    handle: TileSelectionHandle,
) -> (u32, u32) {
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    match handle {
        TileSelectionHandle::TopLeft => (max_x, max_y),
        TileSelectionHandle::TopRight => (min_x, max_y),
        TileSelectionHandle::BottomLeft => (max_x, min_y),
        TileSelectionHandle::BottomRight => (min_x, min_y),
    }
}

fn selection_contains_cell(selection: TileSelectionRegion, cell: (u32, u32)) -> bool {
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    cell.0 >= min_x && cell.0 <= max_x && cell.1 >= min_y && cell.1 <= max_y
}

fn selection_drag_offset(selection: TileSelectionRegion, cell: (u32, u32)) -> (u32, u32) {
    let (min_x, min_y, _, _) = selection_bounds(selection);
    if selection_contains_cell(selection, cell) {
        (cell.0 - min_x, cell.1 - min_y)
    } else {
        (0, 0)
    }
}

fn selection_move_origin_from_cell(state: &AppState, cell: (u32, u32)) -> Option<(u32, u32)> {
    let drag_offset = state
        .tile_selection
        .map(|selection| selection_drag_offset(selection, cell))
        .unwrap_or((0, 0));
    Some(selection_move_origin_from_hit(state, cell, Some(drag_offset)))
}

fn selection_move_origin_from_hit(
    state: &AppState,
    cell: (u32, u32),
    drag_offset: Option<(u32, u32)>,
) -> (u32, u32) {
    let transfer = state
        .tile_selection_transfer
        .as_ref()
        .expect("selection move origin requires transfer state");
    let offset = drag_offset.unwrap_or((0, 0));
    let origin_x = cell.0.saturating_sub(offset.0);
    let origin_y = cell.1.saturating_sub(offset.1);
    clamp_selection_origin(state, origin_x, origin_y, transfer.width, transfer.height)
}

fn selection_from_origin(state: &AppState, origin: (u32, u32)) -> TileSelectionRegion {
    let transfer = state
        .tile_selection_transfer
        .as_ref()
        .expect("selection-from-origin requires transfer state");
    TileSelectionRegion {
        start_cell: origin,
        end_cell: (
            origin.0 + transfer.width.saturating_sub(1),
            origin.1 + transfer.height.saturating_sub(1),
        ),
    }
}

fn clamp_selection_origin(
    state: &AppState,
    origin_x: u32,
    origin_y: u32,
    width: u32,
    height: u32,
) -> (u32, u32) {
    let Some(session) = state.session.as_ref() else {
        return (origin_x, origin_y);
    };
    let map = &session.document().map;
    let max_x = map.width.saturating_sub(width);
    let max_y = map.height.saturating_sub(height);
    (origin_x.min(max_x), origin_y.min(max_y))
}

fn selection_bounds(selection: TileSelectionRegion) -> (u32, u32, u32, u32) {
    (
        selection.start_cell.0.min(selection.end_cell.0),
        selection.start_cell.1.min(selection.end_cell.1),
        selection.start_cell.0.max(selection.end_cell.0),
        selection.start_cell.1.max(selection.end_cell.1),
    )
}

fn hit_test_object(state: &AppState, world_x: f64, world_y: f64) -> Option<(usize, u32)> {
    let session = state.session.as_ref()?;
    let tile_width = session.document().map.tile_width as f64;
    let tile_height = session.document().map.tile_height as f64;
    let point_radius = tile_width.min(tile_height).max(16.0) * 0.45;

    for (layer_index, layer) in session.document().map.layers.iter().enumerate().rev() {
        let Some(object_layer) = layer.as_object() else {
            continue;
        };
        if !object_layer.visible {
            continue;
        }
        for object in object_layer.objects.iter().rev() {
            if !object.visible {
                continue;
            }
            let object_x = f64::from(object.x);
            let object_y = f64::from(object.y);
            let hit = match object.shape {
                ObjectShape::Rectangle => {
                    let object_width = f64::from(object.width).max(tile_width);
                    let object_height = f64::from(object.height).max(tile_height);
                    let max_x = object_x + object_width;
                    let max_y = object_y + object_height;
                    world_x >= object_x
                        && world_x <= max_x
                        && world_y >= object_y
                        && world_y <= max_y
                }
                ObjectShape::Point => {
                    let dx = world_x - object_x;
                    let dy = world_y - object_y;
                    dx * dx + dy * dy <= point_radius * point_radius
                }
            };
            if hit {
                return Some((layer_index, object.id));
            }
        }
    }

    None
}

fn tool_supports_drag(tool: Tool) -> bool {
    matches!(tool, Tool::Paint | Tool::Erase)
}

fn tool_batches_history(tool: Tool) -> bool {
    matches!(tool, Tool::Paint | Tool::Erase)
}

fn start_touch_edit_batch(state: &mut AppState) {
    if state.touch_edit_batch_active || !tool_batches_history(state.tool) {
        return;
    }
    let Some(session) = state.session.as_mut() else {
        return;
    };
    session.begin_history_batch();
    state.touch_edit_batch_active = true;
}

fn finish_touch_edit_batch(state: &mut AppState) {
    if !state.touch_edit_batch_active {
        return;
    }
    if let Some(session) = state.session.as_mut()
        && session.finish_history_batch()
    {
        state.status = "Edit applied.".to_string();
    }
    state.touch_edit_batch_active = false;
}

fn abort_touch_edit_batch(state: &mut AppState) {
    if !state.touch_edit_batch_active {
        return;
    }
    if let Some(session) = state.session.as_mut() {
        session.abort_history_batch();
    }
    state.touch_edit_batch_active = false;
}

fn suppress_synthetic_click(state: &mut AppState) {
    state.suppress_click_until = Some(Instant::now() + SYNTHETIC_CLICK_SUPPRESSION);
}

fn touch_surface_point(
    state: &AppState,
    event: &Event<PointerData>,
) -> dioxus::html::geometry::ElementPoint {
    if let Some((left, top)) = state.canvas_stage_client_origin {
        let point = event.client_coordinates();
        let (scroll_left, scroll_top) = state.canvas_host_scroll_offset;
        #[cfg(target_os = "android")]
        {
            let _ = left;
            return dioxus::html::geometry::ElementPoint::new(
                point.x + scroll_left,
                point.y - top + scroll_top,
            );
        }
        #[cfg(not(target_os = "android"))]
        return dioxus::html::geometry::ElementPoint::new(
            point.x - left + scroll_left,
            point.y - top + scroll_top,
        );
    }

    event.element_coordinates()
}

fn upsert_touch_point(state: &mut AppState, pointer_id: i32, x: f64, y: f64) {
    if let Some(pointer) = state
        .active_touch_points
        .iter_mut()
        .find(|pointer| pointer.pointer_id == pointer_id)
    {
        pointer.x = x;
        pointer.y = y;
        return;
    }
    state
        .active_touch_points
        .push(ActiveTouchPointer { pointer_id, x, y });
    state
        .active_touch_points
        .sort_by_key(|pointer| pointer.pointer_id);
}

fn remove_touch_point(state: &mut AppState, pointer_id: i32) {
    state
        .active_touch_points
        .retain(|pointer| pointer.pointer_id != pointer_id);
}

fn initialize_pinch_gesture(state: &mut AppState) {
    let Some((first, second)) = first_two_touch_points(state) else {
        state.pinch_gesture = None;
        return;
    };
    let zoom = state.zoom_percent as f64 / 100.0;
    let center_x = (first.x + second.x) * 0.5;
    let center_y = (first.y + second.y) * 0.5;
    let distance = touch_distance(first, second).max(MIN_PINCH_DISTANCE);
    let world_center_x = (center_x - f64::from(state.pan_x)) / zoom;
    let world_center_y = (center_y - f64::from(state.pan_y)) / zoom;

    state.pinch_gesture = Some(PinchGesture {
        initial_distance: distance,
        initial_zoom_percent: state.zoom_percent,
        world_center_x,
        world_center_y,
    });
    log_pinch_probe(
        state,
        "pinch-start",
        first,
        second,
        center_x,
        center_y,
        distance,
    );
}

fn update_pinch_gesture(state: &mut AppState) {
    let Some(gesture) = state.pinch_gesture else {
        initialize_pinch_gesture(state);
        return;
    };
    let Some((first, second)) = first_two_touch_points(state) else {
        state.pinch_gesture = None;
        return;
    };

    let current_center_x = (first.x + second.x) * 0.5;
    let current_center_y = (first.y + second.y) * 0.5;
    let current_distance = touch_distance(first, second).max(MIN_PINCH_DISTANCE);
    let scale = current_distance / gesture.initial_distance;
    let new_zoom_percent =
        ((f64::from(gesture.initial_zoom_percent) * scale).round() as i32).clamp(25, 400);
    let new_zoom = f64::from(new_zoom_percent) / 100.0;

    state.zoom_percent = new_zoom_percent;
    state.pan_x = (current_center_x - gesture.world_center_x * new_zoom).round() as i32;
    state.pan_y = (current_center_y - gesture.world_center_y * new_zoom).round() as i32;
    state.status = format!("Zoom {}%.", state.zoom_percent);
    log_pinch_probe(
        state,
        "pinch-update",
        first,
        second,
        current_center_x,
        current_center_y,
        current_distance,
    );
}

#[cfg(target_os = "android")]
fn log_touch_probe(
    state: &AppState,
    event: &Event<PointerData>,
    phase: &'static str,
    surface_x: f64,
    surface_y: f64,
) {
    let client = event.client_coordinates();
    let element = event.element_coordinates();
    let (origin_x, origin_y) = state
        .canvas_stage_client_origin
        .unwrap_or((f64::NAN, f64::NAN));
    let (scroll_left, scroll_top) = state.canvas_host_scroll_offset;
    let tile_size = state
        .session
        .as_ref()
        .map(|session| {
            let map = &session.document().map;
            (map.tile_width, map.tile_height)
        })
        .unwrap_or((0, 0));

    log(format!(
        "touch:{phase} tool={:?} pid={} touches={} client=({:.1},{:.1}) element=({:.1},{:.1}) surface=({:.1},{:.1}) origin=({:.1},{:.1}) scroll=({:.1},{:.1}) pan=({}, {}) zoom={} world={} cell={} tile={}x{}",
        state.tool,
        event.pointer_id(),
        state.active_touch_points.len(),
        client.x,
        client.y,
        element.x,
        element.y,
        surface_x,
        surface_y,
        origin_x,
        origin_y,
        scroll_left,
        scroll_top,
        state.pan_x,
        state.pan_y,
        state.zoom_percent,
        format_world_pair(world_coordinates_from_surface(state, surface_x, surface_y)),
        format_cell(cell_from_surface(state, surface_x, surface_y)),
        tile_size.0,
        tile_size.1,
    ));
}

#[cfg(not(target_os = "android"))]
fn log_touch_probe(
    _state: &AppState,
    _event: &Event<PointerData>,
    _phase: &'static str,
    _surface_x: f64,
    _surface_y: f64,
) {
}

#[cfg(target_os = "android")]
fn log_touch_resolution(state: &AppState, phase: &'static str, surface_x: f64, surface_y: f64) {
    log(format!(
        "touch:{phase} tool={:?} surface=({:.1},{:.1}) pan=({}, {}) zoom={} world={} cell={}",
        state.tool,
        surface_x,
        surface_y,
        state.pan_x,
        state.pan_y,
        state.zoom_percent,
        format_world_pair(world_coordinates_from_surface(state, surface_x, surface_y)),
        format_cell(cell_from_surface(state, surface_x, surface_y)),
    ));
}

#[cfg(not(target_os = "android"))]
fn log_touch_resolution(_state: &AppState, _phase: &'static str, _surface_x: f64, _surface_y: f64) {
}

#[cfg(target_os = "android")]
fn log_pinch_probe(
    state: &AppState,
    phase: &'static str,
    first: ActiveTouchPointer,
    second: ActiveTouchPointer,
    center_x: f64,
    center_y: f64,
    distance: f64,
) {
    log(format!(
        "touch:{phase} first=({:.1},{:.1}) second=({:.1},{:.1}) center=({:.1},{:.1}) distance={:.2} pan=({}, {}) zoom={}",
        first.x,
        first.y,
        second.x,
        second.y,
        center_x,
        center_y,
        distance,
        state.pan_x,
        state.pan_y,
        state.zoom_percent,
    ));
}

#[cfg(not(target_os = "android"))]
fn log_pinch_probe(
    _state: &AppState,
    _phase: &'static str,
    _first: ActiveTouchPointer,
    _second: ActiveTouchPointer,
    _center_x: f64,
    _center_y: f64,
    _distance: f64,
) {
}

#[cfg(target_os = "android")]
fn format_world_pair(world: Option<(f64, f64)>) -> String {
    match world {
        Some((x, y)) => format!("({x:.1},{y:.1})"),
        None => "none".to_string(),
    }
}

#[cfg(not(target_os = "android"))]
#[allow(dead_code)]
fn format_world_pair(_world: Option<(f64, f64)>) -> String {
    String::new()
}

#[cfg(target_os = "android")]
fn format_cell(cell: Option<(u32, u32)>) -> String {
    match cell {
        Some((x, y)) => format!("({x},{y})"),
        None => "none".to_string(),
    }
}

#[cfg(not(target_os = "android"))]
#[allow(dead_code)]
fn format_cell(_cell: Option<(u32, u32)>) -> String {
    String::new()
}

fn first_two_touch_points(state: &AppState) -> Option<(ActiveTouchPointer, ActiveTouchPointer)> {
    let first = *state.active_touch_points.first()?;
    let second = *state.active_touch_points.get(1)?;
    Some((first, second))
}

fn touch_distance(first: ActiveTouchPointer, second: ActiveTouchPointer) -> f64 {
    let dx = first.x - second.x;
    let dy = first.y - second.y;
    (dx * dx + dy * dy).sqrt()
}

fn cell_from_surface(state: &AppState, x: f64, y: f64) -> Option<(u32, u32)> {
    let session = state.session.as_ref()?;
    let map = &session.document().map;
    let (world_x, world_y) = world_coordinates_from_surface(state, x, y)?;
    if world_x < 0.0 || world_y < 0.0 {
        return None;
    }
    let cell_x = (world_x / f64::from(map.tile_width)).floor() as u32;
    let cell_y = (world_y / f64::from(map.tile_height)).floor() as u32;
    if cell_x < map.width && cell_y < map.height {
        Some((cell_x, cell_y))
    } else {
        None
    }
}

fn clamped_cell_from_surface(state: &AppState, x: f64, y: f64) -> Option<(u32, u32)> {
    let session = state.session.as_ref()?;
    let map = &session.document().map;
    if map.width == 0 || map.height == 0 {
        return None;
    }
    let zoom = f64::from(state.zoom_percent) / 100.0;
    if zoom <= 0.0 {
        return None;
    }

    let max_world_x = f64::from(map.total_pixel_width()) - f64::EPSILON;
    let max_world_y = f64::from(map.total_pixel_height()) - f64::EPSILON;
    let world_x = ((x - f64::from(state.pan_x)) / zoom).clamp(0.0, max_world_x.max(0.0));
    let world_y = ((y - f64::from(state.pan_y)) / zoom).clamp(0.0, max_world_y.max(0.0));
    let cell_x = (world_x / f64::from(map.tile_width)).floor() as u32;
    let cell_y = (world_y / f64::from(map.tile_height)).floor() as u32;

    Some((cell_x.min(map.width - 1), cell_y.min(map.height - 1)))
}

fn world_coordinates_from_surface(state: &AppState, x: f64, y: f64) -> Option<(f64, f64)> {
    let session = state.session.as_ref()?;
    let map = &session.document().map;
    let zoom = f64::from(state.zoom_percent) / 100.0;
    if zoom <= 0.0 {
        return None;
    }
    let world_x = (x - f64::from(state.pan_x)) / zoom;
    let world_y = (y - f64::from(state.pan_y)) / zoom;
    if world_x > f64::from(map.total_pixel_width()) || world_y > f64::from(map.total_pixel_height())
    {
        return None;
    }
    Some((world_x, world_y))
}

#[cfg(test)]
mod tests {
    use super::{
        adjusted_selection_resize_surface_point, initialize_pinch_gesture,
        selection_resize_anchor_cell, touch_distance, update_pinch_gesture,
    };
    use crate::app_state::{
        ActiveTouchPointer, AppState, TileSelectionHandle, TileSelectionRegion,
    };

    #[test]
    fn touch_distance_uses_euclidean_length() {
        let first = ActiveTouchPointer {
            pointer_id: 1,
            x: 10.0,
            y: 20.0,
        };
        let second = ActiveTouchPointer {
            pointer_id: 2,
            x: 13.0,
            y: 24.0,
        };
        assert!((touch_distance(first, second) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pinch_uses_initial_zoom_percent_as_baseline() {
        let mut state = AppState {
            zoom_percent: 100,
            active_touch_points: vec![
                ActiveTouchPointer {
                    pointer_id: 1,
                    x: 10.0,
                    y: 10.0,
                },
                ActiveTouchPointer {
                    pointer_id: 2,
                    x: 30.0,
                    y: 10.0,
                },
            ],
            ..AppState::default()
        };
        initialize_pinch_gesture(&mut state);

        state.zoom_percent = 150;
        state.active_touch_points = vec![
            ActiveTouchPointer {
                pointer_id: 1,
                x: 0.0,
                y: 10.0,
            },
            ActiveTouchPointer {
                pointer_id: 2,
                x: 40.0,
                y: 10.0,
            },
        ];

        update_pinch_gesture(&mut state);

        assert_eq!(state.zoom_percent, 200);
    }

    #[test]
    fn selection_handles_resize_from_the_opposite_corner() {
        let selection = TileSelectionRegion {
            start_cell: (2, 3),
            end_cell: (5, 7),
        };

        assert_eq!(
            selection_resize_anchor_cell(selection, TileSelectionHandle::TopLeft),
            (5, 7)
        );
        assert_eq!(
            selection_resize_anchor_cell(selection, TileSelectionHandle::TopRight),
            (2, 7)
        );
        assert_eq!(
            selection_resize_anchor_cell(selection, TileSelectionHandle::BottomLeft),
            (5, 3)
        );
        assert_eq!(
            selection_resize_anchor_cell(selection, TileSelectionHandle::BottomRight),
            (2, 3)
        );
    }

    #[test]
    fn resize_handle_adjustment_biases_towards_the_selected_region() {
        assert_eq!(
            adjusted_selection_resize_surface_point(
                192.0,
                256.0,
                Some(TileSelectionHandle::TopLeft)
            ),
            (192.5, 256.5)
        );
        assert_eq!(
            adjusted_selection_resize_surface_point(
                192.0,
                256.0,
                Some(TileSelectionHandle::BottomRight)
            ),
            (191.5, 255.5)
        );
    }
}
