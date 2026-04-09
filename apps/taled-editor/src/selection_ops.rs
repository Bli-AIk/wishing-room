use std::collections::BTreeSet;

use taled_core::Layer;

use crate::app_state::{
    AppState, TileClipboard, TileSelectionRegion, TileSelectionTransfer, TileSelectionTransferMode,
    selection_bounds,
};

// ── Copy / Cut / Delete ─────────────────────────────────────────────

pub(crate) fn copy_tile_selection(state: &mut AppState) {
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
    }

    dismiss_tile_selection(state);
}

// ── Helpers ─────────────────────────────────────────────────────────

fn selected_tile_selection(
    state: &AppState,
) -> Option<(usize, TileSelectionRegion, BTreeSet<(i32, i32)>)> {
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

fn selection_dimensions(region: &TileSelectionRegion) -> (u32, u32) {
    let (min_x, min_y, max_x, max_y) = selection_bounds(region);
    ((max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32)
}

fn selection_mask_from_cells(
    region: &TileSelectionRegion,
    cells: &BTreeSet<(i32, i32)>,
) -> Vec<bool> {
    let (min_x, min_y, max_x, max_y) = selection_bounds(region);
    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;
    let mut mask = Vec::with_capacity(width * height);
    for ly in 0..height {
        for lx in 0..width {
            mask.push(cells.contains(&(min_x + lx as i32, min_y + ly as i32)));
        }
    }
    mask
}

fn selection_cells_from_mask(
    origin_x: i32,
    origin_y: i32,
    width: u32,
    height: u32,
    mask: &[bool],
) -> BTreeSet<(i32, i32)> {
    let mut cells = BTreeSet::new();
    for ly in 0..height {
        for lx in 0..width {
            let idx = (ly * width + lx) as usize;
            if mask.get(idx).copied().unwrap_or(false) {
                cells.insert((origin_x + lx as i32, origin_y + ly as i32));
            }
        }
    }
    cells
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

fn capture_region_clipped(
    tile_layer: &taled_core::TileLayer,
    min_x: i32,
    min_y: i32,
    width: u32,
    height: u32,
) -> Vec<u32> {
    let mut tiles = Vec::with_capacity((width * height) as usize);
    for ly in 0..height {
        for lx in 0..width {
            let x = min_x + lx as i32;
            let y = min_y + ly as i32;
            let gid = if tile_layer_in_bounds(tile_layer, x, y) {
                tile_layer.tile_at(x as u32, y as u32).unwrap_or(0)
            } else {
                0
            };
            tiles.push(gid);
        }
    }
    tiles
}

fn write_region_tiles_clipped(
    tile_layer: &mut taled_core::TileLayer,
    min_x: i32,
    min_y: i32,
    width: u32,
    height: u32,
    tiles: &[u32],
    mask: Option<&[bool]>,
) -> taled_core::Result<()> {
    for ly in 0..height {
        for lx in 0..width {
            let idx = (ly * width + lx) as usize;
            if mask.is_some_and(|m| !m.get(idx).copied().unwrap_or(false)) {
                continue;
            }
            let gid = tiles[idx];
            let x = min_x + lx as i32;
            let y = min_y + ly as i32;
            if tile_layer_in_bounds(tile_layer, x, y) {
                tile_layer.set_tile(x as u32, y as u32, gid)?;
            }
        }
    }
    Ok(())
}

fn clear_region_tiles_masked(
    tile_layer: &mut taled_core::TileLayer,
    min_x: i32,
    min_y: i32,
    width: u32,
    height: u32,
    mask: &[bool],
) -> taled_core::Result<()> {
    for ly in 0..height {
        for lx in 0..width {
            let idx = (ly * width + lx) as usize;
            if !mask.get(idx).copied().unwrap_or(false) {
                continue;
            }
            let x = min_x + lx as i32;
            let y = min_y + ly as i32;
            if tile_layer_in_bounds(tile_layer, x, y) {
                tile_layer.set_tile(x as u32, y as u32, 0)?;
            }
        }
    }
    Ok(())
}

fn tile_layer_mut(
    document: &mut taled_core::EditorDocument,
    layer_index: usize,
) -> taled_core::Result<&mut taled_core::TileLayer> {
    let layer = document
        .map
        .layer_mut(layer_index)
        .ok_or_else(|| taled_core::EditorError::Invalid(format!("unknown layer {layer_index}")))?;
    if layer.locked() {
        return Err(taled_core::EditorError::Invalid(
            "layer is locked".to_string(),
        ));
    }
    layer
        .as_tile_mut()
        .ok_or_else(|| taled_core::EditorError::Invalid("not a tile layer".to_string()))
}

fn tile_layer_in_bounds(tile_layer: &taled_core::TileLayer, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && (x as u32) < tile_layer.width && (y as u32) < tile_layer.height
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
