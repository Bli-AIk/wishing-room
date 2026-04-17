use std::time::Instant;

use taled_core::Layer;

use crate::app_state::{
    AppState, TileClipboard, TileSelectionMode, TileSelectionTransfer, TileSelectionTransferMode,
    UndoActionKind, selection_bounds,
};
use crate::selection_transform::{
    apply_transfer_copy, capture_region_clipped, clear_region_tiles_masked,
    delete_transfer_and_exit, selection_cells_from_mask, selection_dimensions,
    selection_mask_from_cells, selection_region_from_cells, tile_layer_mut,
    write_region_tiles_clipped,
};

// ── Copy / Cut / Delete ─────────────────────────────────────────────

pub(crate) fn copy_tile_selection(state: &mut AppState) {
    // In transfer mode: place a copy at the current position without exiting
    if state.tile_selection_transfer.is_some() {
        let placed = apply_transfer_copy(state);
        if placed && let Some(t) = state.tile_selection_transfer.as_ref() {
            state.status = format!("Copied {}×{} at current position.", t.width, t.height);
        }
        return;
    }

    let Some((transfer, clipboard)) = capture_tile_selection_transfer(state) else {
        return;
    };
    let (w, h) = (transfer.width, transfer.height);

    state.tile_clipboard = Some(clipboard);
    state.tile_selection_transfer = Some(transfer);
    clear_preview_state(state);
    state.status = format!("Copied region {w}×{h}. Drag to place.");
}

pub(crate) fn cut_tile_selection(state: &mut AppState) {
    // If already in transfer mode, convert Copy→Cut or warn if already Cut
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        if matches!(transfer.mode, TileSelectionTransferMode::Cut) {
            state.status = "Already in cut-move mode.".to_string();
            return;
        }
        // Convert Copy transfer → Cut: clear source tiles now
        let (min_x, min_y, _, _) = selection_bounds(&transfer.source_selection);
        let source_layer = transfer.source_layer;
        let (w, h) = (transfer.width, transfer.height);
        let mask = transfer.source_mask.clone();
        let Some(session) = state.session.as_mut() else {
            state.status = "No map loaded.".to_string();
            return;
        };
        session.begin_history_batch();
        let clear_result = session.edit(|document| {
            let tile_layer = tile_layer_mut(document, source_layer)?;
            clear_region_tiles_masked(tile_layer, min_x, min_y, w, h, &mask)
        });
        match clear_result {
            Ok(()) => {
                state.canvas_dirty = true;
                state.tiles_dirty = true;
                if let Some(t) = state.tile_selection_transfer.as_mut() {
                    t.mode = TileSelectionTransferMode::Cut;
                }
                state.status = format!("Cut {w}×{h}. Drag to move, tap Done to place.");
            }
            Err(error) => {
                session.abort_history_batch();
                state.status = format!("Cut failed: {error}");
            }
        }
        return;
    }

    let Some((transfer, clipboard)) = capture_tile_selection_transfer(state) else {
        return;
    };
    let (min_x, min_y) = (
        transfer
            .source_selection
            .start_cell
            .0
            .min(transfer.source_selection.end_cell.0),
        transfer
            .source_selection
            .start_cell
            .1
            .min(transfer.source_selection.end_cell.1),
    );
    let (w, h) = (transfer.width, transfer.height);
    let mask = transfer.source_mask.clone();

    let Some(session) = state.session.as_mut() else {
        state.status = "No map loaded.".to_string();
        return;
    };
    session.begin_history_batch();
    let clear_result = session.edit(|document| {
        let tile_layer = tile_layer_mut(document, transfer.source_layer)?;
        clear_region_tiles_masked(tile_layer, min_x, min_y, w, h, &mask)
    });

    match clear_result {
        Ok(()) => {
            state.canvas_dirty = true;
            state.tiles_dirty = true;
            state.tile_clipboard = Some(clipboard);
            state.tile_selection_transfer = Some(TileSelectionTransfer {
                mode: TileSelectionTransferMode::Cut,
                ..transfer
            });
            clear_preview_state(state);
            state.status = format!("Cut region {w}×{h}. Drag to move.");
        }
        Err(error) => {
            if let Some(session) = state.session.as_mut() {
                session.abort_history_batch();
            }
            state.status = format!("Cut failed: {error}");
        }
    }
}

pub(crate) fn delete_selection(state: &mut AppState) {
    // In transfer mode: discard floating tiles, delete source, exit
    if state.tile_selection_transfer.is_some() {
        delete_transfer_and_exit(state);
        return;
    }

    let Some((layer_index, selection, cells)) = selected_tile_selection(state) else {
        state.status = "Select a tile region first.".to_string();
        return;
    };
    let (min_x, min_y, _, _) = selection_bounds(&selection);
    let (w, h) = selection_dimensions(&selection);
    let mask = selection_mask_from_cells(&selection, &cells);

    let Some(session) = state.session.as_mut() else {
        state.status = "No map loaded.".to_string();
        return;
    };
    let result = session.edit(move |document| {
        let tile_layer = tile_layer_mut(document, layer_index)?;
        clear_region_tiles_masked(tile_layer, min_x, min_y, w, h, &mask)
    });

    match result {
        Ok(()) => {
            state.canvas_dirty = true;
            state.tiles_dirty = true;
            state
                .undo_action_order
                .push(crate::app_state::UndoActionKind::DocumentEdit);
            state.redo_action_order.clear();
            state.selection_redo_stack.clear();
            dismiss_tile_selection(state);
            state.status = "Cleared selected region.".to_string();
        }
        Err(error) => state.status = format!("Delete failed: {error}"),
    }
}

// ── Transfer placement ──────────────────────────────────────────────

pub(crate) fn place_tile_selection_transfer(state: &mut AppState) {
    let Some(transfer) = state.tile_selection_transfer.clone() else {
        state.status = "Nothing to place.".to_string();
        return;
    };
    let Some(selection) = state.tile_selection else {
        state.status = "Move the selection before placing it.".to_string();
        return;
    };
    let target_layer = state.active_layer;
    let (min_x, min_y, _, _) = selection_bounds(&selection);

    if transfer.source_layer != target_layer {
        cancel_tile_selection_transfer(state);
        state.status = "Canceled: active layer changed.".to_string();
        return;
    }

    let result = if matches!(transfer.mode, TileSelectionTransferMode::Cut) {
        let Some(session) = state.session.as_mut() else {
            state.status = "No map loaded.".to_string();
            return;
        };
        let r = session.edit(|document| {
            let tile_layer = tile_layer_mut(document, target_layer)?;
            write_region_tiles_clipped(
                tile_layer,
                min_x,
                min_y,
                transfer.width,
                transfer.height,
                &transfer.tiles,
                Some(&transfer.mask),
            )
        });
        if r.is_ok() {
            session.finish_history_batch();
        } else {
            session.abort_history_batch();
        }
        r
    } else {
        let tiles = transfer.tiles.clone();
        let (w, h, mask) = (transfer.width, transfer.height, transfer.mask.clone());
        let Some(session) = state.session.as_mut() else {
            state.status = "No map loaded.".to_string();
            return;
        };
        session.edit(move |document| {
            let tile_layer = tile_layer_mut(document, target_layer)?;
            write_region_tiles_clipped(tile_layer, min_x, min_y, w, h, &tiles, Some(&mask))
        })
    };

    match result {
        Ok(()) => {
            state.canvas_dirty = true;
            state.tiles_dirty = true;
            state
                .undo_action_order
                .push(crate::app_state::UndoActionKind::DocumentEdit);
            state.redo_action_order.clear();
            state.selection_redo_stack.clear();
            let cells = selection_cells_from_mask(
                min_x,
                min_y,
                transfer.width,
                transfer.height,
                &transfer.mask,
            );
            state.tile_selection = selection_region_from_cells(&cells);
            state.tile_selection_cells = Some(cells);
            state.tile_selection_transfer = None;
            state.status = format!("Placed selection at ({min_x}, {min_y}).");
        }
        Err(error) => {
            state.tile_selection_transfer = None;
            state.status = format!("Place failed: {error}");
        }
    }
}

pub(crate) fn cancel_tile_selection_transfer(state: &mut AppState) {
    let Some(transfer) = state.tile_selection_transfer.take() else {
        return;
    };

    if matches!(transfer.mode, TileSelectionTransferMode::Cut) {
        let (min_x, min_y, _, _) = selection_bounds(&transfer.source_selection);
        let restore = {
            let Some(session) = state.session.as_mut() else {
                return;
            };
            session.edit(|document| {
                let tile_layer = tile_layer_mut(document, transfer.source_layer)?;
                write_region_tiles_clipped(
                    tile_layer,
                    min_x,
                    min_y,
                    transfer.width,
                    transfer.height,
                    &transfer.tiles,
                    Some(&transfer.source_mask),
                )
            })
        };
        if let Some(session) = state.session.as_mut() {
            session.abort_history_batch();
        }
        if restore.is_err() {
            state.status = "Cancel failed: could not restore cut region.".to_string();
        }
        state.canvas_dirty = true;
        state.tiles_dirty = true;
    }

    dismiss_tile_selection(state);
}

// ── Helpers ─────────────────────────────────────────────────────────

fn selected_tile_selection(
    state: &AppState,
) -> Option<(
    usize,
    crate::app_state::TileSelectionRegion,
    std::collections::BTreeSet<(i32, i32)>,
)> {
    let selection = state.tile_selection?;
    let cells = state.tile_selection_cells.clone()?;
    let layer = state
        .session
        .as_ref()
        .and_then(|s| s.document().map.layer(state.active_layer))
        .and_then(Layer::as_tile);
    layer.map(|_| (state.active_layer, selection, cells))
}

fn capture_tile_selection_transfer(
    state: &mut AppState,
) -> Option<(TileSelectionTransfer, TileClipboard)> {
    let (layer_index, selection, cells) = selected_tile_selection(state)?;
    let session = state.session.as_ref()?;
    let tile_layer = session
        .document()
        .map
        .layer(layer_index)
        .and_then(Layer::as_tile)?;

    let (min_x, min_y, _, _) = selection_bounds(&selection);
    let (width, height) = selection_dimensions(&selection);
    let mask = selection_mask_from_cells(&selection, &cells);
    let tiles = capture_region_clipped(tile_layer, min_x, min_y, width, height);

    let transfer = TileSelectionTransfer {
        source_layer: layer_index,
        source_selection: selection,
        source_mask: mask.clone(),
        width,
        height,
        tiles: tiles.clone(),
        mask: mask.clone(),
        mode: TileSelectionTransferMode::Copy,
    };
    let clipboard = TileClipboard {
        width,
        height,
        tiles,
        mask,
    };
    Some((transfer, clipboard))
}

fn dismiss_tile_selection(state: &mut AppState) {
    state.tile_selection_last_tap_at = None;
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.tile_selection = None;
    state.tile_selection_cells = None;
    state.tile_selection_transfer = None;
    state.canvas_dirty = true;
}

fn clear_preview_state(state: &mut AppState) {
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.tile_selection_closing = None;
    state.tile_selection_closing_cells = None;
    state.tile_selection_closing_started_at = None;
    state.tile_selection_last_tap_at = None;
    state.canvas_dirty = true;
}

// ── Selection dismissal ─────────────────────────────────────────────

/// Double-tap window for dismissing a selection.
const DOUBLE_TAP_WINDOW: std::time::Duration = std::time::Duration::from_millis(320);

/// Try to dismiss the current selection via double-tap or tap-outside.
/// Returns `true` if the selection was dismissed (caller should skip normal tool action).
pub(crate) fn try_dismiss_selection(state: &mut AppState, x: u32, y: u32) -> bool {
    let has_selection = state.tile_selection_cells.is_some();
    if !has_selection {
        return false;
    }
    let cell_i32 = (x as i32, y as i32);
    let inside = state
        .tile_selection_cells
        .as_ref()
        .is_some_and(|cells| cells.contains(&cell_i32));

    if inside {
        if let Some(last_tap) = state.tile_selection_last_tap_at
            && last_tap.elapsed() < DOUBLE_TAP_WINDOW
        {
            dismiss_selection(state);
            return true;
        }
        state.tile_selection_last_tap_at = Some(Instant::now());
        state.tile_selection_mode == TileSelectionMode::Replace
    } else {
        if state.tile_selection_mode == TileSelectionMode::Replace {
            dismiss_selection(state);
            return true;
        }
        false
    }
}

pub(crate) fn dismiss_selection(state: &mut AppState) {
    state
        .selection_undo_stack
        .push(state.tile_selection_cells.clone());
    state.selection_redo_stack.clear();
    state
        .undo_action_order
        .push(UndoActionKind::SelectionChange);
    state.redo_action_order.clear();

    state.tile_selection = None;
    state.tile_selection_cells = None;
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.tile_selection_last_tap_at = None;
    state.tile_selection_transfer = None;
    state.canvas_dirty = true;
    state.status = "Selection cleared.".to_string();
}
