use std::collections::BTreeSet;

use taled_core::Layer;

use crate::app_state::{
    AppState, TileClipboard, TileSelectionRegion, TileSelectionTransferMode, selection_bounds,
};

// ── Public flip / rotate API ────────────────────────────────────────

pub(crate) fn flip_tile_selection_x(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        transfer.tiles = flip_tiles_h(transfer.width, transfer.height, &transfer.tiles);
        transfer.mask = flip_mask_h(transfer.width, transfer.height, &transfer.mask);
        sync_clipboard_from_transfer(state);
        state.canvas_dirty = true;
        state.status = "Flipped X.".to_string();
        return;
    }
    flip_in_place(state, flip_tiles_h, flip_mask_h, "X");
}

pub(crate) fn flip_tile_selection_y(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        transfer.tiles = flip_tiles_v(transfer.width, transfer.height, &transfer.tiles);
        transfer.mask = flip_mask_v(transfer.width, transfer.height, &transfer.mask);
        sync_clipboard_from_transfer(state);
        state.canvas_dirty = true;
        state.status = "Flipped Y.".to_string();
        return;
    }
    flip_in_place(state, flip_tiles_v, flip_mask_v, "Y");
}

pub(crate) fn rotate_tile_selection_cw(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        let (ow, oh) = (transfer.width, transfer.height);
        transfer.tiles = rotate_tiles_cw(ow, oh, &transfer.tiles);
        transfer.mask = rotate_mask_cw(ow, oh, &transfer.mask);
        transfer.width = oh;
        transfer.height = ow;
        sync_clipboard_from_transfer(state);
        resize_transfer_selection(state);
        state.canvas_dirty = true;
        state.status = "Rotated 90° CW.".to_string();
        return;
    }
    rotate_in_place(state);
}

// ── Transfer-mode helpers (called from selection_ops) ───────────────

pub(crate) fn apply_transfer_copy(state: &mut AppState) -> bool {
    let Some(transfer) = state.tile_selection_transfer.clone() else {
        return false;
    };
    let Some(selection) = state.tile_selection else {
        return false;
    };
    let target_layer = state.active_layer;
    let (min_x, min_y, _, _) = selection_bounds(&selection);
    let tiles = transfer.tiles.clone();
    let (w, h) = (transfer.width, transfer.height);
    let mask = transfer.mask.clone();

    let Some(session) = state.session.as_mut() else {
        return false;
    };
    let result = session.edit(move |document| {
        let tl = tile_layer_mut(document, target_layer)?;
        write_region_tiles_clipped(tl, min_x, min_y, w, h, &tiles, Some(&mask))
    });
    if result.is_ok() {
        state.canvas_dirty = true;
        state.tiles_dirty = true;
        state
            .undo_action_order
            .push(crate::app_state::UndoActionKind::DocumentEdit);
        state.redo_action_order.clear();
        state.selection_redo_stack.clear();
    }
    result.is_ok()
}

pub(crate) fn delete_transfer_and_exit(state: &mut AppState) {
    let Some(transfer) = state.tile_selection_transfer.take() else {
        return;
    };
    if matches!(transfer.mode, TileSelectionTransferMode::Cut) {
        if let Some(session) = state.session.as_mut() {
            session.abort_history_batch();
        }
        state.canvas_dirty = true;
        state.tiles_dirty = true;
    } else {
        let (min_x, min_y, _, _) = selection_bounds(&transfer.source_selection);
        let mask = transfer.source_mask.clone();
        let source_layer = transfer.source_layer;
        let (w, h) = (transfer.width, transfer.height);
        let result = state.session.as_mut().and_then(|session| {
            session
                .edit(|document| {
                    let tl = tile_layer_mut(document, source_layer)?;
                    clear_region_tiles_masked(tl, min_x, min_y, w, h, &mask)
                })
                .ok()
        });
        if result.is_some() {
            state.canvas_dirty = true;
            state.tiles_dirty = true;
            state
                .undo_action_order
                .push(crate::app_state::UndoActionKind::DocumentEdit);
            state.redo_action_order.clear();
            state.selection_redo_stack.clear();
        }
    }
    state.tile_selection = None;
    state.tile_selection_cells = None;
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.status = "Deleted source and exited move mode.".to_string();
}

// ── Selection geometry helpers ──────────────────────────────────────

pub(crate) fn selection_dimensions(region: &TileSelectionRegion) -> (u32, u32) {
    let (min_x, min_y, max_x, max_y) = selection_bounds(region);
    ((max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32)
}

pub(crate) fn selection_mask_from_cells(
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

pub(crate) fn selection_cells_from_mask(
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

pub(crate) fn selection_region_from_cells(
    cells: &BTreeSet<(i32, i32)>,
) -> Option<TileSelectionRegion> {
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

// ── Tile I/O helpers ────────────────────────────────────────────────

pub(crate) fn capture_region_clipped(
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

pub(crate) fn write_region_tiles_clipped(
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

pub(crate) fn clear_region_tiles_masked(
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

pub(crate) fn tile_layer_mut(
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

pub(crate) fn tile_layer_in_bounds(tile_layer: &taled_core::TileLayer, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && (x as u32) < tile_layer.width && (y as u32) < tile_layer.height
}

// ── Tile transform primitives ───────────────────────────────────────

fn flip_tiles_h(w: u32, h: u32, tiles: &[u32]) -> Vec<u32> {
    let mut out = vec![0; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            out[(y * w + x) as usize] = tiles[(y * w + (w - 1 - x)) as usize];
        }
    }
    out
}

fn flip_tiles_v(w: u32, h: u32, tiles: &[u32]) -> Vec<u32> {
    let mut out = vec![0; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            out[(y * w + x) as usize] = tiles[((h - 1 - y) * w + x) as usize];
        }
    }
    out
}

fn rotate_tiles_cw(w: u32, h: u32, tiles: &[u32]) -> Vec<u32> {
    let mut out = vec![0; (w * h) as usize];
    for sy in 0..h {
        for sx in 0..w {
            let dx = h - 1 - sy;
            let dy = sx;
            out[(dy * h + dx) as usize] = tiles[(sy * w + sx) as usize];
        }
    }
    out
}

fn flip_mask_h(w: u32, h: u32, mask: &[bool]) -> Vec<bool> {
    let mut out = vec![false; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            out[(y * w + x) as usize] = mask
                .get((y * w + (w - 1 - x)) as usize)
                .copied()
                .unwrap_or(false);
        }
    }
    out
}

fn flip_mask_v(w: u32, h: u32, mask: &[bool]) -> Vec<bool> {
    let mut out = vec![false; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            out[(y * w + x) as usize] = mask
                .get(((h - 1 - y) * w + x) as usize)
                .copied()
                .unwrap_or(false);
        }
    }
    out
}

fn rotate_mask_cw(w: u32, h: u32, mask: &[bool]) -> Vec<bool> {
    let mut out = vec![false; (w * h) as usize];
    for sy in 0..h {
        for sx in 0..w {
            let dx = h - 1 - sy;
            let dy = sx;
            out[(dy * h + dx) as usize] =
                mask.get((sy * w + sx) as usize).copied().unwrap_or(false);
        }
    }
    out
}

// ── In-place transforms (non-transfer mode) ─────────────────────────

fn flip_in_place(
    state: &mut AppState,
    flip_tiles: fn(u32, u32, &[u32]) -> Vec<u32>,
    flip_mask: fn(u32, u32, &[bool]) -> Vec<bool>,
    axis: &str,
) {
    let Some((layer_index, selection, cells)) = selected_tile_selection(state) else {
        state.status = "Select a region first.".to_string();
        return;
    };
    let (min_x, min_y, _, _) = selection_bounds(&selection);
    let (w, h) = selection_dimensions(&selection);
    let mask = selection_mask_from_cells(&selection, &cells);
    let next_mask = flip_mask(w, h, &mask);
    let mask_for_edit = mask.clone();
    let next_mask_edit = next_mask.clone();

    let Some(session) = state.session.as_mut() else {
        state.status = "No map loaded.".to_string();
        return;
    };
    let result = session.edit(move |document| {
        let tl = tile_layer_mut(document, layer_index)?;
        let snapshot = capture_region_clipped(tl, min_x, min_y, w, h);
        let flipped = flip_tiles(w, h, &snapshot);
        clear_region_tiles_masked(tl, min_x, min_y, w, h, &mask_for_edit)?;
        write_region_tiles_clipped(tl, min_x, min_y, w, h, &flipped, Some(&next_mask_edit))
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
            let next_cells = selection_cells_from_mask(min_x, min_y, w, h, &next_mask);
            state.tile_selection_cells = Some(next_cells);
            state.status = format!("Flipped {axis}.");
        }
        Err(error) => state.status = format!("Flip failed: {error}"),
    }
}

fn rotate_in_place(state: &mut AppState) {
    let Some((layer_index, selection, cells)) = selected_tile_selection(state) else {
        state.status = "Select a region first.".to_string();
        return;
    };
    let (min_x, min_y, _, _) = selection_bounds(&selection);
    let (w, h) = selection_dimensions(&selection);
    let (nw, nh) = (h, w);
    let mask = selection_mask_from_cells(&selection, &cells);
    let next_mask = rotate_mask_cw(w, h, &mask);
    let mask_for_edit = mask.clone();
    let next_mask_edit = next_mask.clone();

    let Some(session) = state.session.as_mut() else {
        state.status = "No map loaded.".to_string();
        return;
    };
    let result = session.edit(move |document| {
        let tl = tile_layer_mut(document, layer_index)?;
        let snapshot = capture_region_clipped(tl, min_x, min_y, w, h);
        let rotated = rotate_tiles_cw(w, h, &snapshot);
        clear_region_tiles_masked(tl, min_x, min_y, w, h, &mask_for_edit)?;
        write_region_tiles_clipped(tl, min_x, min_y, nw, nh, &rotated, Some(&next_mask_edit))
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
            let next_cells = selection_cells_from_mask(min_x, min_y, nw, nh, &next_mask);
            state.tile_selection = selection_region_from_cells(&next_cells);
            state.tile_selection_cells = Some(next_cells);
            state.status = "Rotated 90° CW.".to_string();
        }
        Err(error) => state.status = format!("Rotate failed: {error}"),
    }
}

// ── Private helpers ─────────────────────────────────────────────────

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

fn sync_clipboard_from_transfer(state: &mut AppState) {
    if let Some(t) = state.tile_selection_transfer.as_ref() {
        state.tile_clipboard = Some(TileClipboard {
            width: t.width,
            height: t.height,
            tiles: t.tiles.clone(),
            mask: t.mask.clone(),
        });
    }
}

fn resize_transfer_selection(state: &mut AppState) {
    let Some(transfer) = state.tile_selection_transfer.as_ref() else {
        return;
    };
    let Some(selection) = state.tile_selection else {
        return;
    };
    let (min_x, min_y, _, _) = selection_bounds(&selection);
    let cells = selection_cells_from_mask(
        min_x,
        min_y,
        transfer.width,
        transfer.height,
        &transfer.mask,
    );
    state.tile_selection = selection_region_from_cells(&cells);
    state.tile_selection_cells = Some(cells);
}
