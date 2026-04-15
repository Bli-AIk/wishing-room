use std::collections::{BTreeSet, VecDeque};

use taled_core::Layer;

use crate::app_state::{
    AppState, ShapeFillMode, TileSelectionMode, TileSelectionRegion, Tool, selection_bounds,
};

// ── Edit wrappers ───────────────────────────────────────────────────

fn apply_tile_edit<F>(state: &mut AppState, edit: F)
where
    F: FnOnce(&mut taled_core::EditorDocument) -> taled_core::Result<()>,
{
    let Some(session) = state.session.as_mut() else {
        state.status = "No map loaded.".to_string();
        return;
    };
    match session.edit(edit) {
        Ok(()) => {
            state.canvas_dirty = true;
            state.tiles_dirty = true;
        }
        Err(error) => state.status = format!("Edit failed: {error}"),
    }
}

// ── Core tool dispatch ──────────────────────────────────────────────

pub(crate) fn apply_cell_tool(state: &mut AppState, x: u32, y: u32) {
    state.selected_cell = Some((x, y));
    let layer_index = state.active_layer;
    match state.tool {
        Tool::Hand => {}
        Tool::Paint => {
            let gid = state.selected_gid;
            apply_tile_edit(state, move |document| {
                let layer = tile_layer_mut(document, layer_index)?;
                layer.set_tile(x, y, gid)?;
                Ok(())
            });
        }
        Tool::Erase => {
            apply_tile_edit(state, move |document| {
                let layer = tile_layer_mut(document, layer_index)?;
                layer.set_tile(x, y, 0)?;
                Ok(())
            });
        }
        Tool::Fill => apply_fill(state, x, y),
        Tool::ShapeFill => apply_shape_fill(state, x, y, x, y),
        Tool::Select => {
            if state
                .session
                .as_ref()
                .and_then(|s| s.document().map.layer(layer_index))
                .is_some_and(|l| l.as_tile().is_some())
            {
                select_tile_region(state, x as i32, y as i32, x as i32, y as i32);
            }
        }
        Tool::MagicWand => {
            apply_magic_wand_selection(state, x, y);
        }
        Tool::SelectSameTile => {
            apply_select_same_tile_selection(state, x, y);
        }
        Tool::AddRectangle | Tool::AddPoint | Tool::InsertTile => {
            // Object tools handled in touch_ops
        }
        Tool::SelectObject => {
            // Object selection handled in touch_ops
        }
    }
}

// ── Selection operations ────────────────────────────────────────────

pub(crate) fn select_tile_region(
    state: &mut AppState,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
) {
    let region = TileSelectionRegion {
        start_cell: (start_x, start_y),
        end_cell: (end_x, end_y),
    };
    let cells = selection_cells_from_region(region);
    set_tile_selection(state, cells);
    let w = (start_x - end_x).unsigned_abs() + 1;
    let h = (start_y - end_y).unsigned_abs() + 1;
    state.status = format!("Selected {w}×{h} region.");
}

fn set_tile_selection(state: &mut AppState, cells: BTreeSet<(i32, i32)>) {
    // Record previous selection for undo.
    state
        .selection_undo_stack
        .push(state.tile_selection_cells.clone());
    state.selection_redo_stack.clear();
    state
        .undo_action_order
        .push(crate::app_state::UndoActionKind::SelectionChange);
    state.redo_action_order.clear();

    let mode = state.tile_selection_mode;
    let merged = match mode {
        TileSelectionMode::Replace => cells.clone(),
        TileSelectionMode::Add => {
            let mut merged = state.tile_selection_cells.clone().unwrap_or_default();
            merged.extend(&cells);
            merged
        }
        TileSelectionMode::Subtract => {
            let mut existing = state.tile_selection_cells.clone().unwrap_or_default();
            for c in &cells {
                existing.remove(c);
            }
            existing
        }
        TileSelectionMode::Intersect => {
            let existing = state.tile_selection_cells.clone().unwrap_or_default();
            existing.intersection(&cells).copied().collect()
        }
    };
    if merged.is_empty() {
        state.tile_selection = None;
        state.tile_selection_cells = None;
    } else {
        state.tile_selection = selection_region_from_cells(&merged);
        state.tile_selection_cells = Some(merged);
    }
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.canvas_dirty = true;
}

fn selection_region_from_cells(cells: &BTreeSet<(i32, i32)>) -> Option<TileSelectionRegion> {
    let first = cells.iter().next()?;
    let (mut min_x, mut min_y) = *first;
    let (mut max_x, mut max_y) = *first;
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

fn apply_magic_wand_selection(state: &mut AppState, x: u32, y: u32) {
    let session = state.session.as_ref();
    let Some(layer) = session
        .and_then(|s| s.document().map.layer(state.active_layer))
        .and_then(Layer::as_tile)
    else {
        return;
    };
    let target_gid = layer.tile_at(x, y).unwrap_or(0);
    let cells = magic_wand_cells(layer, x, y, target_gid);
    let cells_i32: BTreeSet<(i32, i32)> = cells
        .into_iter()
        .map(|(cx, cy)| (cx as i32, cy as i32))
        .collect();
    let count = cells_i32.len();
    set_tile_selection(state, cells_i32);
    state.status = format!("Magic Wand selected {count} tiles.");
}

fn apply_select_same_tile_selection(state: &mut AppState, x: u32, y: u32) {
    let session = state.session.as_ref();
    let Some(layer) = session
        .and_then(|s| s.document().map.layer(state.active_layer))
        .and_then(Layer::as_tile)
    else {
        return;
    };
    let target_gid = layer.tile_at(x, y).unwrap_or(0);
    let mut cells = BTreeSet::new();
    for row in 0..layer.height {
        for col in 0..layer.width {
            if layer.tile_at(col, row) == Some(target_gid) {
                cells.insert((col as i32, row as i32));
            }
        }
    }
    let count = cells.len();
    set_tile_selection(state, cells);
    state.status = format!("Selected {count} tiles of same type.");
}

fn magic_wand_cells(
    layer: &taled_core::TileLayer,
    x: u32,
    y: u32,
    target_gid: u32,
) -> BTreeSet<(u32, u32)> {
    let mut visited = vec![false; layer.tiles.len()];
    let mut result = BTreeSet::new();
    let mut queue = VecDeque::from([(x, y)]);
    while let Some((cx, cy)) = queue.pop_front() {
        let Some(idx) = layer.index_of(cx, cy) else {
            continue;
        };
        if visited[idx] {
            continue;
        }
        visited[idx] = true;
        if layer.tiles[idx] != target_gid {
            continue;
        }
        result.insert((cx, cy));
        if cx > 0 {
            queue.push_back((cx - 1, cy));
        }
        if cx + 1 < layer.width {
            queue.push_back((cx + 1, cy));
        }
        if cy > 0 {
            queue.push_back((cx, cy - 1));
        }
        if cy + 1 < layer.height {
            queue.push_back((cx, cy + 1));
        }
    }
    result
}

// ── Fill operations ─────────────────────────────────────────────────

fn apply_fill(state: &mut AppState, x: u32, y: u32) {
    let layer_index = state.active_layer;
    let replacement_gid = state.selected_gid;
    apply_tile_edit(state, move |document| {
        let layer = tile_layer_mut(document, layer_index)?;
        let target_gid = layer
            .tile_at(x, y)
            .ok_or_else(|| taled_core::EditorError::Invalid(format!("out of bounds: {x},{y}")))?;
        if target_gid == replacement_gid {
            return Ok(());
        }
        let mut queue = VecDeque::from([(x, y)]);
        let mut visited = vec![false; layer.tiles.len()];
        while let Some((cx, cy)) = queue.pop_front() {
            let Some(idx) = layer.index_of(cx, cy) else {
                continue;
            };
            if visited[idx] {
                continue;
            }
            visited[idx] = true;
            if layer.tiles[idx] != target_gid {
                continue;
            }
            layer.tiles[idx] = replacement_gid;
            if cx > 0 {
                queue.push_back((cx - 1, cy));
            }
            if cx + 1 < layer.width {
                queue.push_back((cx + 1, cy));
            }
            if cy > 0 {
                queue.push_back((cx, cy - 1));
            }
            if cy + 1 < layer.height {
                queue.push_back((cx, cy + 1));
            }
        }
        Ok(())
    });
}

pub(crate) fn apply_shape_fill(
    state: &mut AppState,
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,
) {
    let layer_index = state.active_layer;
    let gid = state.selected_gid;
    let mode = state.shape_fill_mode;
    let cells = shape_fill_cells(mode, start_x, start_y, end_x, end_y);
    let count = cells.len();
    apply_tile_edit(state, move |document| {
        let layer = tile_layer_mut(document, layer_index)?;
        for (x, y) in &cells {
            layer.set_tile(*x, *y, gid)?;
        }
        Ok(())
    });
    let label = match mode {
        ShapeFillMode::Rectangle => "Rectangle",
        ShapeFillMode::Ellipse => "Ellipse",
    };
    state.status = format!("{label} filled {count} tiles.");
}

// ── Utility helpers ─────────────────────────────────────────────────

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

pub(crate) fn selection_cells_from_region(region: TileSelectionRegion) -> BTreeSet<(i32, i32)> {
    let (min_x, min_y, max_x, max_y) = selection_bounds(&region);
    let mut cells = BTreeSet::new();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            cells.insert((x, y));
        }
    }
    cells
}

fn shape_fill_cells(
    mode: ShapeFillMode,
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,
) -> BTreeSet<(u32, u32)> {
    let min_x = start_x.min(end_x);
    let max_x = start_x.max(end_x);
    let min_y = start_y.min(end_y);
    let max_y = start_y.max(end_y);
    let mut cells = BTreeSet::new();
    match mode {
        ShapeFillMode::Rectangle => {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    cells.insert((x, y));
                }
            }
        }
        ShapeFillMode::Ellipse => {
            fill_ellipse_cells(&mut cells, min_x, min_y, max_x, max_y);
        }
    }
    cells
}

fn fill_ellipse_cells(
    cells: &mut BTreeSet<(u32, u32)>,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
) {
    let w = (max_x - min_x + 1) as f32;
    let h = (max_y - min_y + 1) as f32;
    let cx = min_x as f32 + w / 2.0;
    let cy = min_y as f32 + h / 2.0;
    let rx = w / 2.0;
    let ry = h / 2.0;
    for ty in min_y..=max_y {
        for tx in min_x..=max_x {
            let nx = (tx as f32 + 0.5 - cx) / rx.max(f32::EPSILON);
            let ny = (ty as f32 + 0.5 - cy) / ry.max(f32::EPSILON);
            if nx * nx + ny * ny <= 1.0 {
                cells.insert((tx, ty));
            }
        }
    }
}

/// Cancel tile selection transfer (e.g., when switching tools).
#[allow(dead_code)]
pub(crate) fn cancel_tile_selection_transfer(state: &mut AppState) {
    crate::selection_ops::cancel_tile_selection_transfer(state);
}
