use std::time::Instant;

use ply_engine::prelude::*;

use crate::app_state::TileSelectionRegion;
use crate::app_state::{AppState, PinchGesture, ShapeFillPreview, SingleTouchGesture, Tool};
use crate::edit_ops::{self, selection_cells_from_region};
use crate::obj_ops;
use crate::selection_ops;

/// Duration before a held touch starts continuous painting.
const LONG_PRESS_DURATION: std::time::Duration = std::time::Duration::from_millis(120);

/// Minimum finger distance to recognise a pinch gesture.
const MIN_PINCH_DISTANCE: f64 = 12.0;

/// Whether the current tool uses tile selection (Select, MagicWand, SelectSameTile).
fn is_selection_tool(tool: Tool) -> bool {
    matches!(tool, Tool::Select | Tool::MagicWand | Tool::SelectSameTile)
}

// ── Screen → Grid conversion ────────────────────────────────────────

/// Convert macroquad screen coordinates to a valid in-bounds grid cell.
pub(crate) fn cell_from_screen(
    state: &AppState,
    screen_x: f32,
    screen_y: f32,
    canvas_origin_y: f32,
) -> Option<(u32, u32)> {
    let (col, row) = signed_cell_from_screen(state, screen_x, screen_y, canvas_origin_y)?;
    let session = state.session.as_ref()?;
    let map = &session.document().map;
    if col >= 0 && row >= 0 && (col as u32) < map.width && (row as u32) < map.height {
        Some((col as u32, row as u32))
    } else {
        None
    }
}

/// Convert screen coordinates to Tiled world coordinates (unzoomed, top-down pixels).
fn world_from_screen(
    state: &AppState,
    screen_x: f32,
    screen_y: f32,
    canvas_origin_y: f32,
) -> Option<(f32, f32)> {
    state.session.as_ref()?;
    let zoom = state.zoom_percent as f32 / 100.0;
    if zoom <= 0.0 {
        return None;
    }
    let canvas_x = screen_x - state.pan_x;
    let canvas_y = screen_y - canvas_origin_y - state.pan_y;
    Some((canvas_x / zoom, canvas_y / zoom))
}

/// Convert screen coordinates to a possibly-negative grid cell.
fn signed_cell_from_screen(
    state: &AppState,
    screen_x: f32,
    screen_y: f32,
    canvas_origin_y: f32,
) -> Option<(i32, i32)> {
    let session = state.session.as_ref()?;
    let map = &session.document().map;
    let zoom = state.zoom_percent as f32 / 100.0;
    if zoom <= 0.0 {
        return None;
    }
    let canvas_x = screen_x - state.pan_x;
    let canvas_y = screen_y - canvas_origin_y - state.pan_y;
    let world_x = canvas_x / zoom;
    let world_y = canvas_y / zoom;
    let col = (world_x / map.tile_width as f32).floor() as i32;
    let row = (world_y / map.tile_height as f32).floor() as i32;
    Some((col, row))
}

// ── Canvas interaction entry point ──────────────────────────────────

/// Process pointer state within the canvas area each frame.
///
/// Called from `render_canvas` while inside the canvas-area children
/// closure.  `canvas_origin_y` is the top of the canvas area in layout
/// coordinates (header + tile strip height).
pub(crate) fn handle_canvas_interaction(ui: &mut Ui, state: &mut AppState, canvas_origin_y: f32) {
    if state.session.is_none() {
        return;
    }
    // Skip canvas touches while joystick, zoom slider, or viewfinder is being used.
    if state.joystick_active || state.zoom_slider_active || state.viewfinder_touch_active {
        return;
    }

    // Deferred centering: apply for a few frames to ensure screen dimensions stabilize.
    if state.pending_canvas_center > 0 {
        let sw = screen_width();
        if sw > 1.0 {
            if let Some(session) = state.session.as_ref() {
                let (px, py, dbg) = crate::session_ops::default_center_pan(
                    session,
                    state.zoom_percent,
                    state.safe_inset_top,
                );
                state.pan_x = px;
                state.pan_y = py;
                state.center_debug = format!("F{} {dbg}", state.pending_canvas_center);
            }
            state.pending_canvas_center -= 1;
            state.canvas_dirty = true;
        }
    }

    let (mx, my) = mouse_position();
    let touches = touches();
    let dpi = screen_dpi_scale();
    let tc = touches.len();
    let sw = screen_width();
    let sh = screen_height();
    let cd = state.center_debug.clone();

    // Pinch zoom with two fingers — only if both fingers are below the tile strip.
    if tc >= 2 {
        let strip_bottom = state.safe_inset_top + 56.0 + 114.0;
        let (t0, t1) = (touches[0].position / dpi, touches[1].position / dpi);
        if t0.y < strip_bottom && t1.y < strip_bottom {
            // Both touches in strip area — skip canvas pinch
            state.debug_info = format!("strip-pinch tc:{tc} [{cd}]");
            return;
        }
        finish_touch_edit_batch(state);
        state.single_touch_gesture = None;
        state.shape_fill_preview = None;
        handle_pinch(&touches, state, canvas_origin_y);
        let (t0, t1) = (touches[0].position / dpi, touches[1].position / dpi);
        state.debug_info = format!(
            "pinch tc:{tc} dpi:{dpi:.1} t0({:.0},{:.0}) t1({:.0},{:.0}) z:{} pan({:.0},{:.0}) [{cd}]",
            t0.x, t0.y, t1.x, t1.y, state.zoom_percent, state.pan_x, state.pan_y
        );
        return;
    }

    // Clear pinch gesture when fewer than 2 fingers to prevent snap-back on next pinch.
    state.pinch_gesture = None;

    let jp = ui.just_pressed();
    let p = ui.pressed();
    let jr = ui.just_released();

    // Single-touch / mouse
    if jp {
        state.pinch_gesture = None;
        start_single_gesture(state, mx, my, canvas_origin_y);
    } else if p {
        handle_drag(state, mx, my, canvas_origin_y);
    }

    if jr {
        handle_release(state, mx, my, canvas_origin_y);
    }

    state.debug_info = format!(
        "sw:{sw:.0} sh:{sh:.0} pan({:.0},{:.0}) t:{:?} jp:{} p:{} jr:{} tc:{tc} [{cd}]",
        state.pan_x, state.pan_y, state.tool, jp as u8, p as u8, jr as u8
    );
}

// ── Gesture lifecycle ───────────────────────────────────────────────

fn start_single_gesture(state: &mut AppState, mx: f32, my: f32, canvas_origin_y: f32) {
    let anchor = cell_from_screen(state, mx, my, canvas_origin_y);

    // In Replace mode, detect if the touch starts outside the current selection.
    // If so, the drag should NOT create a new selection — only dismiss the old one.
    let outside = is_selection_tool(state.tool)
        && state.tile_selection_mode == crate::app_state::TileSelectionMode::Replace
        && state.tile_selection_transfer.is_none()
        && state.tile_selection_cells.is_some()
        && anchor.is_none_or(|(x, y)| {
            state
                .tile_selection_cells
                .as_ref()
                .is_some_and(|cells| !cells.contains(&(x as i32, y as i32)))
        });

    state.single_touch_gesture = Some(SingleTouchGesture {
        pointer_id: 0,
        started_at: Instant::now(),
        drag_active: false,
        outside_existing_selection: outside,
        anchor_cell: if outside {
            None
        } else {
            anchor.map(|(x, y)| (x as i32, y as i32))
        },
        last_applied_cell: None,
        last_surface_x: mx as f64,
        last_surface_y: my as f64,
        pan_remainder_x: 0.0,
        pan_remainder_y: 0.0,
    });

    // Immediate action for single-tap tools
    match state.tool {
        Tool::Hand => {}
        Tool::Paint | Tool::Erase => {
            // Don't paint on initial press — wait for long-press or release
        }
        Tool::Fill
        | Tool::MagicWand
        | Tool::SelectSameTile
        | Tool::AddRectangle
        | Tool::AddPoint
        | Tool::InsertTile => {
            // These fire on release, not press
        }
        Tool::Select | Tool::ShapeFill => {
            // Drag-based — handled in handle_drag / handle_release
        }
        Tool::SelectObject => {
            // Record world position for potential drag. Selection fires on release.
            if let Some((wx, wy)) = world_from_screen(state, mx, my, canvas_origin_y) {
                state.obj_drag_origin = Some((wx, wy));
            }
        }
    }

    start_touch_edit_batch(state);
}

fn handle_drag(state: &mut AppState, mx: f32, my: f32, canvas_origin_y: f32) {
    let Some(gesture) = state.single_touch_gesture.as_mut() else {
        return;
    };

    let dx = mx as f64 - gesture.last_surface_x;
    let dy = my as f64 - gesture.last_surface_y;
    let tool = state.tool;

    match tool {
        Tool::Hand => {
            state.pan_x += dx as f32;
            state.pan_y += dy as f32;
            state.canvas_dirty = true;
            gesture.last_surface_x = mx as f64;
            gesture.last_surface_y = my as f64;
        }
        Tool::Paint | Tool::Erase => {
            let elapsed = gesture.started_at.elapsed();
            let last = gesture.last_applied_cell;
            let should_paint = elapsed >= LONG_PRESS_DURATION || gesture.drag_active;
            gesture.drag_active = gesture.drag_active || should_paint;
            gesture.last_surface_x = mx as f64;
            gesture.last_surface_y = my as f64;
            if should_paint
                && let Some(cell) = cell_from_screen(state, mx, my, canvas_origin_y)
                && last != Some(cell)
            {
                edit_ops::apply_cell_tool(state, cell.0, cell.1);
                if let Some(g) = state.single_touch_gesture.as_mut() {
                    g.last_applied_cell = Some(cell);
                }
            }
        }
        Tool::ShapeFill => {
            gesture.drag_active = true;
            gesture.last_surface_x = mx as f64;
            gesture.last_surface_y = my as f64;
            let anchor = gesture.anchor_cell;
            if let Some(anchor) = anchor
                && let Some(current) = cell_from_screen(state, mx, my, canvas_origin_y)
            {
                state.shape_fill_preview = Some(ShapeFillPreview {
                    start_cell: (anchor.0 as u32, anchor.1 as u32),
                    end_cell: current,
                });
            }
        }
        Tool::Select => {
            gesture.drag_active = true;
            gesture.last_surface_x = mx as f64;
            gesture.last_surface_y = my as f64;
            if state.tile_selection_transfer.is_some() {
                // In transfer mode: move the selection region
                if let Some(anchor) = gesture.anchor_cell
                    && let Some(current) = signed_cell_from_screen(state, mx, my, canvas_origin_y)
                    && let Some(transfer) = state.tile_selection_transfer.as_ref()
                {
                    let dx = current.0 - anchor.0;
                    let dy = current.1 - anchor.1;
                    let (src_min_x, src_min_y, _, _) =
                        crate::app_state::selection_bounds(&transfer.source_selection);
                    let new_min_x = src_min_x + dx;
                    let new_min_y = src_min_y + dy;
                    let w = transfer.width as i32;
                    let h = transfer.height as i32;
                    let region = TileSelectionRegion {
                        start_cell: (new_min_x, new_min_y),
                        end_cell: (new_min_x + w - 1, new_min_y + h - 1),
                    };
                    state.tile_selection = Some(region);
                    let cells = selection_cells_from_region(region);
                    state.tile_selection_cells = Some(cells);
                    state.canvas_dirty = true;
                }
            } else {
                let anchor = gesture.anchor_cell;
                if let Some(anchor) = anchor
                    && let Some(current) = signed_cell_from_screen(state, mx, my, canvas_origin_y)
                {
                    let region = TileSelectionRegion {
                        start_cell: anchor,
                        end_cell: current,
                    };
                    let cells = selection_cells_from_region(region);
                    state.tile_selection_preview_cells = Some(cells);
                    state.canvas_dirty = true;
                }
            }
        }
        _ => {
            gesture.last_surface_x = mx as f64;
            gesture.last_surface_y = my as f64;
        }
    }

    // Object drag (SelectObject tool with a selected object)
    if tool == Tool::SelectObject {
        handle_object_drag(state, mx, my, canvas_origin_y);
    }
}

/// Apply object drag movement while the finger is held down.
fn handle_object_drag(state: &mut AppState, mx: f32, my: f32, canvas_origin_y: f32) {
    let Some(obj_id) = state.selected_object else {
        return;
    };
    let Some((ox, oy)) = state.obj_drag_origin else {
        return;
    };
    let Some((wx, wy)) = world_from_screen(state, mx, my, canvas_origin_y) else {
        return;
    };
    let ddx = wx - ox;
    let ddy = wy - oy;

    // Lazily capture the original position on first drag movement.
    if state.obj_drag_start_pos.is_none() {
        let start = state
            .session
            .as_ref()
            .and_then(|s| s.document().map.layer(state.active_layer))
            .and_then(|l| l.as_object())
            .and_then(|ol| ol.objects.iter().find(|o| o.id == obj_id))
            .map(|o| (o.x, o.y));
        state.obj_drag_start_pos = start;
    }

    if let Some((sx, sy)) = state.obj_drag_start_pos {
        let (new_x, new_y) = crate::obj_ops::snap_position(state, sx + ddx, sy + ddy);
        if let Some(session) = state.session.as_mut()
            && let Some(layer) = session.document_mut().map.layer_mut(state.active_layer)
            && let Some(obj_layer) = layer.as_object_mut()
            && let Some(obj) = obj_layer.object_mut(obj_id)
        {
            obj.x = new_x;
            obj.y = new_y;
            state.canvas_dirty = true;
            // Force text input fields to resync with new position.
            state.obj_info_synced_for = None;
        }
    }
}

fn handle_release(state: &mut AppState, mx: f32, my: f32, canvas_origin_y: f32) {
    let gesture = state.single_touch_gesture.take();
    let cell = cell_from_screen(state, mx, my, canvas_origin_y);

    match state.tool {
        Tool::Paint | Tool::Erase => {
            let was_drag = gesture.as_ref().is_some_and(|g| g.drag_active);
            if !was_drag && let Some((x, y)) = cell {
                edit_ops::apply_cell_tool(state, x, y);
            }
        }
        Tool::Fill => {
            if let Some((x, y)) = cell {
                edit_ops::apply_cell_tool(state, x, y);
            }
        }
        Tool::ShapeFill => {
            if let Some(preview) = state.shape_fill_preview.take() {
                edit_ops::apply_shape_fill(
                    state,
                    preview.start_cell.0,
                    preview.start_cell.1,
                    preview.end_cell.0,
                    preview.end_cell.1,
                );
            }
        }
        Tool::MagicWand | Tool::SelectSameTile => {
            let outside = gesture
                .as_ref()
                .is_some_and(|g| g.outside_existing_selection);
            if outside {
                selection_ops::dismiss_selection(state);
            } else if let Some((x, y)) = cell {
                if selection_ops::try_dismiss_selection(state, x, y) {
                    // Selection was dismissed via double-tap or tap-outside
                } else {
                    edit_ops::apply_cell_tool(state, x, y);
                }
            }
        }
        Tool::Select if state.tile_selection_transfer.is_some() => {
            // Transfer mode: drag already moved the selection in handle_drag
            state.tile_selection_preview_cells = None;
        }
        Tool::Select => {
            state.tile_selection_preview_cells = None;
            let outside = gesture
                .as_ref()
                .is_some_and(|g| g.outside_existing_selection);
            if outside {
                selection_ops::dismiss_selection(state);
            } else {
                let was_drag = gesture.as_ref().is_some_and(|g| g.drag_active);
                if was_drag
                    && let Some(g) = &gesture
                    && let Some(anchor) = g.anchor_cell
                {
                    let end =
                        signed_cell_from_screen(state, mx, my, canvas_origin_y).unwrap_or(anchor);
                    edit_ops::select_tile_region(state, anchor.0, anchor.1, end.0, end.1);
                } else if let Some((x, y)) = cell
                    && !selection_ops::try_dismiss_selection(state, x, y)
                {
                    edit_ops::apply_cell_tool(state, x, y);
                }
            }
        }
        Tool::Hand => {}
        Tool::SelectObject => {
            let was_drag = gesture.as_ref().is_some_and(|g| g.drag_active);
            if !was_drag {
                // Tap (not drag) → hit-test and select object
                if let Some((wx, wy)) = world_from_screen(state, mx, my, canvas_origin_y) {
                    let hit = state
                        .session
                        .as_ref()
                        .and_then(|s| s.document().map.layer(state.active_layer))
                        .and_then(|l| l.as_object())
                        .and_then(|ol| crate::canvas_objects::hit_test_object(ol, wx, wy));
                    state.selected_object = hit;
                    state.canvas_dirty = true;
                }
            }
            // Clear drag state
            state.obj_drag_origin = None;
            state.obj_drag_start_pos = None;
        }
        Tool::InsertTile => {
            let was_drag = gesture.as_ref().is_some_and(|g| g.drag_active);
            if !was_drag
                && let Some((wx, wy)) = world_from_screen(state, mx, my, canvas_origin_y)
            {
                obj_ops::insert_tile_object(state, wx, wy);
            }
        }
        _ => {}
    }

    finish_touch_edit_batch(state);
}

// ── Pinch zoom ──────────────────────────────────────────────────────

fn handle_pinch(touches: &[Touch], state: &mut AppState, canvas_origin_y: f32) {
    if touches.len() < 2 {
        state.pinch_gesture = None;
        return;
    }
    // Touch positions are in physical pixels; convert to logical to match pan/layout.
    let dpi = screen_dpi_scale();
    let (t0, t1) = (touches[0].position / dpi, touches[1].position / dpi);
    let center_x = ((t0.x + t1.x) / 2.0) as f64;
    let center_y = ((t0.y + t1.y) / 2.0 - canvas_origin_y) as f64;
    let dist = ((t1.x - t0.x).powi(2) + (t1.y - t0.y).powi(2)).sqrt() as f64;

    if let Some(ref pinch) = state.pinch_gesture {
        let ratio = dist / pinch.initial_distance;
        let new_zoom_percent =
            ((pinch.initial_zoom_percent as f64 * ratio).round() as i32).clamp(25, 800);
        let new_zoom = new_zoom_percent as f64 / 100.0;
        state.zoom_percent = new_zoom_percent;
        // Adjust pan so the world point under the pinch center stays fixed
        state.pan_x = (center_x - pinch.world_center_x * new_zoom).round() as f32;
        state.pan_y = (center_y - pinch.world_center_y * new_zoom).round() as f32;
        state.canvas_dirty = true;
    } else if dist > MIN_PINCH_DISTANCE {
        let current_zoom = state.zoom_percent as f64 / 100.0;
        let world_cx = (center_x - state.pan_x as f64) / current_zoom;
        let world_cy = (center_y - state.pan_y as f64) / current_zoom;
        state.pinch_gesture = Some(PinchGesture {
            initial_distance: dist,
            initial_zoom_percent: state.zoom_percent,
            world_center_x: world_cx,
            world_center_y: world_cy,
        });
    }
}

// ── Edit batch ──────────────────────────────────────────────────────

fn start_touch_edit_batch(state: &mut AppState) {
    if !state.touch_edit_batch_active {
        if let Some(session) = state.session.as_mut() {
            session.begin_history_batch();
        }
        state.touch_edit_batch_active = true;
    }
}

fn finish_touch_edit_batch(state: &mut AppState) {
    if state.touch_edit_batch_active {
        if let Some(session) = state.session.as_mut()
            && session.finish_history_batch()
        {
            state
                .undo_action_order
                .push(crate::app_state::UndoActionKind::DocumentEdit);
            state.redo_action_order.clear();
            state.selection_redo_stack.clear();
        }
        state.touch_edit_batch_active = false;
        state.canvas_dirty = true;
    }
}
