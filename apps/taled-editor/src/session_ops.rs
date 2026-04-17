use std::collections::BTreeSet;

use ply_engine::prelude::*;
use taled_core::{EditorSession, Layer};

use crate::app_state::{AppState, TileSelectionRegion, UndoActionKind};
use crate::embedded_samples::{
    DEFAULT_EMBEDDED_SAMPLE_PATH, embedded_sample, embedded_sample_assets,
};

pub(crate) fn load_embedded_sample(state: &mut AppState) {
    load_sample_by_path(state, DEFAULT_EMBEDDED_SAMPLE_PATH);
}

pub(crate) fn load_sample_by_path(state: &mut AppState, path: &str) {
    crate::logging::append(&format!("loading sample: {path}"));
    match EditorSession::load_embedded(path, embedded_sample_assets()) {
        Ok(session) => {
            let label = embedded_sample(path).map_or(path, |s| s.title);
            state.status = format!("Loaded embedded sample {label} ({path}).");
            crate::logging::append(&format!("loaded ok: {}", state.status));
            install_session(state, session);
        }
        Err(error) => {
            state.status = format!("Embedded demo load failed: {error}");
            crate::logging::append(&format!("load FAILED: {}", state.status));
        }
    }
}

pub(crate) fn load_filesystem_map(state: &mut AppState, path: &str) -> bool {
    crate::logging::append(&format!("loading filesystem map: {path}"));
    match EditorSession::load(path) {
        Ok(session) => {
            state.status = format!("Loaded {path}.");
            crate::logging::append(&format!("loaded ok: {}", state.status));
            install_session(state, session);
            true
        }
        Err(error) => {
            state.status = format!("Load failed: {error}");
            crate::logging::append(&format!("load FAILED: {}", state.status));
            false
        }
    }
}

fn install_session(state: &mut AppState, session: EditorSession) {
    let selected_gid = default_selected_gid(&session);
    state.active_layer = 0;
    state.selected_gid = selected_gid;
    state.selected_cell = None;
    state.selected_object = None;
    state.shape_fill_preview = None;
    state.tile_selection = None;
    state.tile_selection_cells = None;
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.tile_selection_closing = None;
    state.tile_selection_closing_cells = None;
    state.tile_selection_closing_started_at = None;
    state.tile_selection_last_tap_at = None;
    state.tile_selection_transfer = None;
    state.layers_panel_expanded = false;
    state.zoom_percent = 100;
    state.pan_x = 0.0;
    state.pan_y = 0.0;
    state.pending_canvas_center = 3;
    state.camera_transition_active = false;
    state.active_touch_points.clear();
    state.single_touch_gesture = None;
    state.pinch_gesture = None;
    state.touch_edit_batch_active = false;
    state.canvas_dirty = true;
    state.tiles_dirty = true;
    state.undo_action_order.clear();
    state.redo_action_order.clear();
    state.selection_undo_stack.clear();
    state.selection_redo_stack.clear();
    state.session = Some(session);
    state.thumb_pending = true;
    crate::canvas::load_tileset_textures(state);
}

pub(crate) fn adjust_zoom(state: &mut AppState, delta: i32) {
    // Zoom around the viewport center to keep the map visually stable.
    let host_w = screen_width();
    let host_h = screen_height() - crate::canvas::CANVAS_ORIGIN_Y - state.safe_inset_top - 140.0;
    let current_zoom = state.zoom_percent as f32 / 100.0;
    let new_zoom_percent = (state.zoom_percent + delta).clamp(25, 800);
    let new_zoom = new_zoom_percent as f32 / 100.0;
    let cx = host_w * 0.5;
    let cy = host_h * 0.5;
    let world_cx = (cx - state.pan_x) / current_zoom;
    let world_cy = (cy - state.pan_y) / current_zoom;
    state.zoom_percent = new_zoom_percent;
    state.pan_x = (cx - world_cx * new_zoom).round();
    state.pan_y = (cy - world_cy * new_zoom).round();
    state.canvas_dirty = true;
}

#[allow(dead_code)]
pub(crate) fn apply_undo(state: &mut AppState) {
    match state.undo_action_order.last().copied() {
        Some(UndoActionKind::SelectionChange) => {
            state.undo_action_order.pop();
            let prev = state.selection_undo_stack.pop().unwrap_or(None);
            state
                .selection_redo_stack
                .push(state.tile_selection_cells.clone());
            state
                .redo_action_order
                .push(UndoActionKind::SelectionChange);
            restore_selection_cells(state, prev);
            state.status = "Undo selection.".to_string();
        }
        Some(UndoActionKind::DocumentEdit) => {
            let Some(session) = state.session.as_mut() else {
                return;
            };
            if session.undo() {
                state.undo_action_order.pop();
                state.redo_action_order.push(UndoActionKind::DocumentEdit);
                normalize_after_history_change(state);
                state.canvas_dirty = true;
                state.tiles_dirty = true;
                state.status = "Undo applied.".to_string();
            }
        }
        None => {
            // Fallback: try document undo for edits made before tracking started.
            let Some(session) = state.session.as_mut() else {
                state.status = "Nothing to undo.".to_string();
                return;
            };
            if session.undo() {
                normalize_after_history_change(state);
                state.canvas_dirty = true;
                state.tiles_dirty = true;
                state.status = "Undo applied.".to_string();
            } else {
                state.status = "Nothing to undo.".to_string();
            }
        }
    }
}

#[allow(dead_code)]
pub(crate) fn apply_redo(state: &mut AppState) {
    match state.redo_action_order.last().copied() {
        Some(UndoActionKind::SelectionChange) => {
            state.redo_action_order.pop();
            let next = state.selection_redo_stack.pop().unwrap_or(None);
            state
                .selection_undo_stack
                .push(state.tile_selection_cells.clone());
            state
                .undo_action_order
                .push(UndoActionKind::SelectionChange);
            restore_selection_cells(state, next);
            state.status = "Redo selection.".to_string();
        }
        Some(UndoActionKind::DocumentEdit) => {
            let Some(session) = state.session.as_mut() else {
                return;
            };
            if session.redo() {
                state.redo_action_order.pop();
                state.undo_action_order.push(UndoActionKind::DocumentEdit);
                normalize_after_history_change(state);
                state.canvas_dirty = true;
                state.tiles_dirty = true;
                state.status = "Redo applied.".to_string();
            }
        }
        None => {
            let Some(session) = state.session.as_mut() else {
                state.status = "Nothing to redo.".to_string();
                return;
            };
            if session.redo() {
                normalize_after_history_change(state);
                state.canvas_dirty = true;
                state.tiles_dirty = true;
                state.status = "Redo applied.".to_string();
            } else {
                state.status = "Nothing to redo.".to_string();
            }
        }
    }
}

fn restore_selection_cells(state: &mut AppState, cells: Option<BTreeSet<(i32, i32)>>) {
    state.tile_selection = cells.as_ref().and_then(selection_region_from_cells);
    state.tile_selection_cells = cells;
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.tile_selection_last_tap_at = None;
    state.tile_selection_transfer = None;
    state.canvas_dirty = true;
}

fn selection_region_from_cells(cells: &BTreeSet<(i32, i32)>) -> Option<TileSelectionRegion> {
    let &(mut min_x, mut min_y) = cells.iter().next()?;
    let (mut max_x, mut max_y) = (min_x, min_y);
    for &(x, y) in cells {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    Some(TileSelectionRegion {
        start_cell: (min_x, min_y),
        end_cell: (max_x, max_y),
    })
}

#[allow(dead_code)]
fn normalize_after_history_change(state: &mut AppState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let layer_count = session.document().map.layers.len();
    if layer_count == 0 {
        state.active_layer = 0;
        state.selected_object = None;
        state.selected_cell = None;
        state.shape_fill_preview = None;
        state.tile_selection = None;
        state.tile_selection_cells = None;
        state.tile_selection_preview = None;
        state.tile_selection_preview_cells = None;
        state.tile_selection_closing = None;
        state.tile_selection_closing_cells = None;
        state.tile_selection_closing_started_at = None;
        state.tile_selection_last_tap_at = None;
        state.tile_selection_transfer = None;
        return;
    }
    if state.active_layer >= layer_count {
        state.active_layer = layer_count - 1;
    }
    if let Some(object_id) = state.selected_object {
        let exists = session
            .document()
            .map
            .layer(state.active_layer)
            .and_then(Layer::as_object)
            .and_then(|l| l.object(object_id))
            .is_some();
        if !exists {
            state.selected_object = None;
        }
    }
    state.shape_fill_preview = None;
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.tile_selection_closing = None;
    state.tile_selection_closing_cells = None;
    state.tile_selection_closing_started_at = None;
    state.tile_selection_last_tap_at = None;
    state.tile_selection_transfer = None;
}

pub(crate) fn default_center_pan(
    session: &EditorSession,
    zoom_percent: i32,
    safe_inset_top: f32,
) -> (f32, f32, String) {
    let host_w = screen_width();
    let sh = screen_height();
    let host_h = sh - crate::canvas::CANVAS_ORIGIN_Y - safe_inset_top - 140.0;
    let map = &session.document().map;
    let zoom = zoom_percent as f32 / 100.0;
    let map_w = map.total_pixel_width() as f32 * zoom;
    let map_h = map.total_pixel_height() as f32 * zoom;
    let px = ((host_w - map_w) * 0.5).round();
    let py = ((host_h - map_h) * 0.5).round();
    let dbg = format!(
        "sw:{host_w:.0} sh:{sh:.0} hw:{host_w:.0} hh:{host_h:.0} mw:{map_w:.0} mh:{map_h:.0} → px:{px:.0} py:{py:.0}"
    );
    (px, py, dbg)
}

fn default_selected_gid(session: &EditorSession) -> u32 {
    let map = &session.document().map;
    for layer in map.layers.iter().filter_map(Layer::as_tile) {
        for gid in layer.tiles.iter().copied() {
            if gid == 0 {
                continue;
            }
            let Some(reference) = map.tile_reference_for_gid(gid) else {
                continue;
            };
            if reference.tileset.tileset.name != "collision" {
                return gid;
            }
        }
    }
    map.tilesets
        .iter()
        .find(|ts| ts.tileset.name != "collision" && ts.tileset.tile_count > 1)
        .or_else(|| map.tilesets.first())
        .map(|ts| ts.first_gid)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use taled_core::EditorSession;

    use super::default_selected_gid;
    use crate::embedded_samples::embedded_sample_assets;

    fn embedded_session(path: &str) -> EditorSession {
        EditorSession::load_embedded(path, embedded_sample_assets())
            .expect("embedded sample should load")
    }

    #[test]
    fn default_selected_gid_prefers_used_non_collision_tile_for_theater() {
        let session = embedded_session("maps/017-2.tmx");
        let gid = default_selected_gid(&session);
        let reference = session
            .document()
            .map
            .tile_reference_for_gid(gid)
            .expect("selected gid should resolve");
        assert_ne!(gid, 0);
        assert_ne!(reference.tileset.tileset.name, "collision");
    }

    #[test]
    fn default_selected_gid_prefers_used_non_collision_tile_for_frontier() {
        let session = embedded_session("maps/081-3.tmx");
        let gid = default_selected_gid(&session);
        let reference = session
            .document()
            .map
            .tile_reference_for_gid(gid)
            .expect("selected gid should resolve");
        assert_ne!(gid, 0);
        assert_ne!(reference.tileset.tileset.name, "collision");
        assert_ne!(reference.local_id, 0);
    }
}
