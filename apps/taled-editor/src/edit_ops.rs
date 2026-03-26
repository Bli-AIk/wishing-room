use std::{
    collections::BTreeSet,
    collections::VecDeque,
    time::{Duration, Instant},
};

use taled_core::{
    EditorError, EditorSession, Layer, MapObject, ObjectShape, Property, PropertyValue,
};

use crate::app_state::{
    AppState, TileClipboard, TileSelectionMode, TileSelectionRegion, TileSelectionTransfer,
    TileSelectionTransferMode, Tool, selection_bounds, selection_cells_from_mask,
    selection_cells_from_region, selection_mask_from_cells, selection_region_from_cells,
};

const TILE_SELECTION_DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(320);

fn set_tile_selection_cells(state: &mut AppState, cells: BTreeSet<(i32, i32)>) {
    if let Some(region) = selection_region_from_cells(&cells) {
        state.tile_selection = Some(region);
        state.tile_selection_cells = Some(cells);
        state.tile_selection_closing = None;
        state.tile_selection_closing_cells = None;
        state.tile_selection_closing_started_at = None;
        state.tile_selection_last_tap_at = None;
        state.tile_selection_preview = None;
        state.tile_selection_preview_cells = None;
        state.selected_object = None;
        state.selected_cell = None;
    } else {
        dismiss_tile_selection(state);
    }
}

pub(crate) fn apply_tile_selection_mode_cells(
    state: &mut AppState,
    region_cells: BTreeSet<(i32, i32)>,
) {
    let next_cells = match state.tile_selection_mode {
        TileSelectionMode::Replace => region_cells,
        TileSelectionMode::Add => state
            .tile_selection_cells
            .clone()
            .unwrap_or_default()
            .union(&region_cells)
            .copied()
            .collect(),
        TileSelectionMode::Subtract => state
            .tile_selection_cells
            .clone()
            .unwrap_or_default()
            .difference(&region_cells)
            .copied()
            .collect(),
        TileSelectionMode::Intersect => {
            let current = state.tile_selection_cells.clone().unwrap_or_default();
            if current.is_empty() {
                BTreeSet::new()
            } else {
                current.intersection(&region_cells).copied().collect()
            }
        }
    };

    if next_cells.is_empty() {
        dismiss_tile_selection(state);
        state.status = format!(
            "{} selection is empty.",
            selection_mode_label(state.tile_selection_mode)
        );
        return;
    }

    set_tile_selection_cells(state, next_cells.clone());
    let region = selection_region_from_cells(&next_cells).expect("selection bounds");
    let (min_x, min_y, max_x, max_y) = selection_bounds(region);
    state.status = format!(
        "{} selection spanning ({}, {}) to ({}, {}).",
        selection_mode_label(state.tile_selection_mode),
        min_x,
        min_y,
        max_x,
        max_y
    );
}

pub(crate) fn apply_tile_selection_mode_region(
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
    apply_tile_selection_mode_cells(state, selection_cells_from_region(region));
}

fn selection_mode_label(mode: TileSelectionMode) -> &'static str {
    match mode {
        TileSelectionMode::Replace => "Replace",
        TileSelectionMode::Add => "Add",
        TileSelectionMode::Subtract => "Subtract",
        TileSelectionMode::Intersect => "Intersect",
    }
}

pub(crate) fn toggle_layer_visibility(state: &mut AppState, layer_index: usize) {
    apply_edit(state, |document| {
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        layer.set_visible(!layer.visible());
        Ok(())
    });
}

pub(crate) fn toggle_layer_lock(state: &mut AppState, layer_index: usize) {
    apply_edit(state, |document| {
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        layer.set_locked(!layer.locked());
        Ok(())
    });
}

pub(crate) fn rename_layer(state: &mut AppState, layer_index: usize, name: String) {
    apply_edit(state, move |document| {
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        *layer.name_mut() = name;
        Ok(())
    });
}

pub(crate) fn apply_cell_tool(state: &mut AppState, x: u32, y: u32) {
    state.selected_cell = Some((x, y));
    let layer_index = state.active_layer;
    match state.tool {
        Tool::Hand => {}
        Tool::Paint => {
            let gid = state.selected_gid;
            apply_edit(state, move |document| {
                let layer = document.map.layer_mut(layer_index).ok_or_else(|| {
                    EditorError::Invalid(format!("unknown layer index {layer_index}"))
                })?;
                if layer.locked() {
                    return Err(EditorError::Invalid("layer is locked".to_string()));
                }
                let tile_layer = layer.as_tile_mut().ok_or_else(|| {
                    EditorError::Invalid("active layer is not a tile layer".to_string())
                })?;
                tile_layer.set_tile(x, y, gid)?;
                Ok(())
            });
        }
        Tool::Erase => {
            apply_edit(state, move |document| {
                let layer = document.map.layer_mut(layer_index).ok_or_else(|| {
                    EditorError::Invalid(format!("unknown layer index {layer_index}"))
                })?;
                if layer.locked() {
                    return Err(EditorError::Invalid("layer is locked".to_string()));
                }
                let tile_layer = layer.as_tile_mut().ok_or_else(|| {
                    EditorError::Invalid("active layer is not a tile layer".to_string())
                })?;
                tile_layer.set_tile(x, y, 0)?;
                Ok(())
            });
        }
        Tool::Fill => apply_fill(state, x, y),
        Tool::ShapeFill => apply_shape_fill_rect(state, x, y, x, y),
        Tool::Select => {
            if state
                .session
                .as_ref()
                .and_then(|session| session.document().map.layer(layer_index))
                .is_some_and(|layer| layer.as_tile().is_some())
            {
                if handle_tile_selection_tap(state, x, y) {
                    return;
                }
                select_tile_region(state, x as i32, y as i32, x as i32, y as i32);
            }
        }
        Tool::MagicWand => {
            let _ = apply_magic_wand_selection(state, x, y, None);
        }
        Tool::SelectSameTile => {
            let _ = apply_select_same_tile_selection(state, x, y, None);
        }
        Tool::AddRectangle => create_object_at(state, ObjectShape::Rectangle, x, y),
        Tool::AddPoint => create_object_at(state, ObjectShape::Point, x, y),
    }
}

pub(crate) fn select_tile_region(
    state: &mut AppState,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
) {
    let region = crate::app_state::TileSelectionRegion {
        start_cell: (start_x, start_y),
        end_cell: (end_x, end_y),
    };
    set_tile_selection_cells(state, selection_cells_from_region(region));
    let width = (start_x - end_x).unsigned_abs() + 1;
    let height = (start_y - end_y).unsigned_abs() + 1;
    state.status = format!(
        "Selected region {}x{} from ({}, {}) to ({}, {}).",
        width, height, start_x, start_y, end_x, end_y
    );
}

pub(crate) fn preview_magic_wand_selection(
    state: &mut AppState,
    x: u32,
    y: u32,
    sampled_gids: &BTreeSet<u32>,
) -> bool {
    preview_matching_tile_selection(state, Tool::MagicWand, x, y, sampled_gids)
}

pub(crate) fn preview_select_same_tile_selection(
    state: &mut AppState,
    x: u32,
    y: u32,
    sampled_gids: &BTreeSet<u32>,
) -> bool {
    preview_matching_tile_selection(state, Tool::SelectSameTile, x, y, sampled_gids)
}

pub(crate) fn apply_magic_wand_selection(
    state: &mut AppState,
    x: u32,
    y: u32,
    sampled_gids: Option<&BTreeSet<u32>>,
) -> bool {
    apply_matching_tile_selection(state, Tool::MagicWand, x, y, sampled_gids)
}

pub(crate) fn apply_select_same_tile_selection(
    state: &mut AppState,
    x: u32,
    y: u32,
    sampled_gids: Option<&BTreeSet<u32>>,
) -> bool {
    apply_matching_tile_selection(state, Tool::SelectSameTile, x, y, sampled_gids)
}

fn preview_matching_tile_selection(
    state: &mut AppState,
    tool: Tool,
    x: u32,
    y: u32,
    sampled_gids: &BTreeSet<u32>,
) -> bool {
    let Some(preview_cells) = matching_tile_selection_cells(state, tool, x, y, sampled_gids) else {
        state.tile_selection_preview = None;
        state.tile_selection_preview_cells = None;
        return false;
    };
    let Some(preview_region) = selection_region_from_cells(&preview_cells) else {
        state.tile_selection_preview = None;
        state.tile_selection_preview_cells = None;
        return false;
    };
    state.tile_selection_preview = Some(preview_region);
    state.tile_selection_preview_cells = Some(preview_cells);
    true
}

fn apply_matching_tile_selection(
    state: &mut AppState,
    tool: Tool,
    x: u32,
    y: u32,
    sampled_gids: Option<&BTreeSet<u32>>,
) -> bool {
    let gids = sampled_gids
        .cloned()
        .filter(|gids| !gids.is_empty())
        .or_else(|| active_tile_gid(state, x, y).map(|gid| BTreeSet::from([gid])));
    let Some(gids) = gids else {
        state.status = "No tile under the cursor.".to_string();
        return false;
    };
    let Some(selection_cells) = matching_tile_selection_cells(state, tool, x, y, &gids) else {
        state.status = "Nothing matched the current tile.".to_string();
        return false;
    };

    apply_tile_selection_mode_cells(state, selection_cells);
    true
}

fn matching_tile_selection_cells(
    state: &AppState,
    tool: Tool,
    x: u32,
    y: u32,
    sampled_gids: &BTreeSet<u32>,
) -> Option<BTreeSet<(i32, i32)>> {
    let tile_layer = active_tile_layer(state)?;
    if !tile_layer_contains_cell(tile_layer, x as i32, y as i32) {
        return None;
    }

    match tool {
        Tool::MagicWand => Some(magic_wand_cells(tile_layer, x, y, sampled_gids)),
        Tool::SelectSameTile => Some(select_same_tile_cells(tile_layer, sampled_gids)),
        _ => None,
    }
}

fn active_tile_layer(state: &AppState) -> Option<&taled_core::TileLayer> {
    state
        .session
        .as_ref()
        .and_then(|session| session.document().map.layer(state.active_layer))
        .and_then(Layer::as_tile)
}

pub(crate) fn active_tile_gid(state: &AppState, x: u32, y: u32) -> Option<u32> {
    active_tile_layer(state)?
        .tile_at(x, y)
        .filter(|gid| *gid != 0)
}

fn magic_wand_cells(
    tile_layer: &taled_core::TileLayer,
    start_x: u32,
    start_y: u32,
    sampled_gids: &BTreeSet<u32>,
) -> BTreeSet<(i32, i32)> {
    if sampled_gids.is_empty() {
        return BTreeSet::new();
    }

    let Some(target_gid) = tile_layer.tile_at(start_x, start_y) else {
        return BTreeSet::new();
    };
    if !sampled_gids.contains(&target_gid) {
        return BTreeSet::new();
    }

    let mut result = BTreeSet::new();
    let mut queue = VecDeque::from([(start_x, start_y)]);
    let mut visited = vec![false; tile_layer.tiles.len()];

    while let Some((cell_x, cell_y)) = queue.pop_front() {
        let Some(index) = tile_layer.index_of(cell_x, cell_y) else {
            continue;
        };
        if visited[index] {
            continue;
        }
        visited[index] = true;

        let gid = tile_layer.tiles[index];
        if !sampled_gids.contains(&gid) {
            continue;
        }

        result.insert((cell_x as i32, cell_y as i32));

        if cell_x > 0 {
            queue.push_back((cell_x - 1, cell_y));
        }
        if cell_x + 1 < tile_layer.width {
            queue.push_back((cell_x + 1, cell_y));
        }
        if cell_y > 0 {
            queue.push_back((cell_x, cell_y - 1));
        }
        if cell_y + 1 < tile_layer.height {
            queue.push_back((cell_x, cell_y + 1));
        }
    }

    result
}

fn select_same_tile_cells(
    tile_layer: &taled_core::TileLayer,
    sampled_gids: &BTreeSet<u32>,
) -> BTreeSet<(i32, i32)> {
    let mut result = BTreeSet::new();
    if sampled_gids.is_empty() {
        return result;
    }

    for y in 0..tile_layer.height {
        for x in 0..tile_layer.width {
            if tile_layer
                .tile_at(x, y)
                .is_some_and(|gid| sampled_gids.contains(&gid))
            {
                result.insert((x as i32, y as i32));
            }
        }
    }

    result
}

pub(crate) fn handle_tile_selection_tap(state: &mut AppState, x: u32, y: u32) -> bool {
    if state.tile_selection_transfer.is_some() {
        return false;
    }

    let Some(selection_cells) = state.tile_selection_cells.as_ref() else {
        return false;
    };

    if state.tile_selection_mode != TileSelectionMode::Replace {
        let already_selected = selection_cells.contains(&(x as i32, y as i32));
        match state.tile_selection_mode {
            TileSelectionMode::Add => {
                if already_selected {
                    state.status = "Add selection unchanged.".to_string();
                    return true;
                }
            }
            TileSelectionMode::Subtract => {
                if !already_selected {
                    state.status = "Subtract selection unchanged.".to_string();
                    return true;
                }
            }
            TileSelectionMode::Intersect => {}
            TileSelectionMode::Replace => {}
        }
        apply_tile_selection_mode_region(state, x as i32, y as i32, x as i32, y as i32);
        return true;
    }

    if selection_cells.contains(&(x as i32, y as i32)) {
        let now = Instant::now();
        if state
            .tile_selection_last_tap_at
            .is_some_and(|last| now.duration_since(last) <= TILE_SELECTION_DOUBLE_TAP_WINDOW)
        {
            dismiss_tile_selection(state);
            state.status = "Selection closed.".to_string();
        } else {
            state.tile_selection_last_tap_at = Some(now);
        }
        true
    } else {
        dismiss_tile_selection(state);
        state.status = "Selection cleared.".to_string();
        true
    }
}

pub(crate) fn dismiss_tile_selection(state: &mut AppState) {
    state.tile_selection_last_tap_at = None;
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    if let (Some(selection), Some(selection_cells)) = (
        state.tile_selection.take(),
        state.tile_selection_cells.take(),
    ) {
        state.tile_selection_closing = Some(selection);
        state.tile_selection_closing_cells = Some(selection_cells);
        state.tile_selection_closing_started_at = Some(Instant::now());
    } else {
        state.tile_selection_closing = None;
        state.tile_selection_closing_cells = None;
        state.tile_selection_closing_started_at = None;
    }
}

pub(crate) fn clear_tile_selection_immediately(state: &mut AppState) {
    state.tile_selection_last_tap_at = None;
    state.tile_selection = None;
    state.tile_selection_cells = None;
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.tile_selection_closing = None;
    state.tile_selection_closing_cells = None;
    state.tile_selection_closing_started_at = None;
}

pub(crate) fn copy_tile_selection(state: &mut AppState) {
    if state.tile_selection_transfer.is_some() {
        let placed = apply_tile_selection_transfer(state, false);
        if let Some(transfer) = state.tile_selection_transfer.as_ref()
            && placed
        {
            state.status = format!(
                "Copied moving region {}x{} at the current position.",
                transfer.width, transfer.height
            );
        }
        return;
    }

    let Some((transfer, clipboard)) = capture_tile_selection_transfer(state) else {
        return;
    };
    let width = transfer.width;
    let height = transfer.height;

    state.tile_clipboard = Some(clipboard);
    state.tile_selection_transfer = Some(transfer);
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.tile_selection_closing = None;
    state.tile_selection_closing_cells = None;
    state.tile_selection_closing_started_at = None;
    state.tile_selection_last_tap_at = None;
    state.selected_object = None;
    state.selected_cell = None;
    state.status = format!("Copied region {}x{}. Drag to place.", width, height);
}

pub(crate) fn cut_tile_selection(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        if matches!(transfer.mode, TileSelectionTransferMode::Cut) {
            state.status = "Selection is already in cut-move mode.".to_string();
            return;
        }

        let (min_x, min_y, _, _) = selection_bounds(transfer.source_selection);
        let Some(session) = state.session.as_mut() else {
            state.status = "No map loaded.".to_string();
            return;
        };

        session.begin_history_batch();
        let clear_result = session.edit(|document| {
            let tile_layer = selected_tile_layer_mut(document, transfer.source_layer)?;
            clear_region_tiles_masked(
                tile_layer,
                min_x,
                min_y,
                transfer.width,
                transfer.height,
                &transfer.source_mask,
            )
        });

        match clear_result {
            Ok(()) => {
                transfer.mode = TileSelectionTransferMode::Cut;
                state.tile_selection_closing = None;
                state.tile_selection_closing_cells = None;
                state.tile_selection_closing_started_at = None;
                state.tile_selection_last_tap_at = None;
                state.status = format!(
                    "Cut moving region {}x{}. Drag to move, tap Done to place.",
                    transfer.width, transfer.height
                );
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
    let (min_x, min_y, _, _) = selection_bounds(transfer.source_selection);
    let Some(session) = state.session.as_mut() else {
        state.status = "No map loaded.".to_string();
        return;
    };

    session.begin_history_batch();
    let clear_result = session.edit(|document| {
        let tile_layer = selected_tile_layer_mut(document, transfer.source_layer)?;
        clear_region_tiles_masked(
            tile_layer,
            min_x,
            min_y,
            transfer.width,
            transfer.height,
            &transfer.source_mask,
        )
    });

    match clear_result {
        Ok(()) => {
            state.tile_clipboard = Some(clipboard);
            state.tile_selection_transfer = Some(TileSelectionTransfer {
                mode: TileSelectionTransferMode::Cut,
                ..transfer
            });
            state.tile_selection_preview = None;
            state.tile_selection_preview_cells = None;
            state.tile_selection_closing = None;
            state.tile_selection_closing_cells = None;
            state.tile_selection_closing_started_at = None;
            state.tile_selection_last_tap_at = None;
            state.selected_object = None;
            state.selected_cell = None;
            state.status = format!(
                "Cut region {}x{}. Drag to move.",
                transfer.width, transfer.height
            );
        }
        Err(error) => {
            session.abort_history_batch();
            state.status = format!("Cut failed: {error}");
        }
    }
}

pub(crate) fn flip_tile_selection_horizontally(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        transfer.tiles = flip_tiles_horizontally(transfer.width, transfer.height, &transfer.tiles);
        transfer.mask = flip_mask_horizontally(transfer.width, transfer.height, &transfer.mask);
        sync_clipboard_from_transfer(state);
        resize_transfer_selection(state);
        state.status = "Flipped moving selection on the X axis.".to_string();
        return;
    }

    let Some((layer_index, selection, selection_cells)) = selected_tile_selection(state) else {
        state.status = "Select a tile region first.".to_string();
        return;
    };
    let (min_x, min_y, _, _) = selection_bounds(selection);
    let (width, height) = selection_dimensions(selection);
    let mask = selection_mask_from_cells(selection, &selection_cells);
    let next_mask = flip_mask_horizontally(width, height, &mask);
    let mask_for_edit = mask.clone();
    let next_mask_for_edit = next_mask.clone();

    apply_edit(state, move |document| {
        let tile_layer = selected_tile_layer_mut(document, layer_index)?;
        let snapshot = capture_region_clipped(tile_layer, min_x, min_y, width, height);
        let flipped_tiles = flip_tiles_horizontally(width, height, &snapshot);
        clear_region_tiles_masked(tile_layer, min_x, min_y, width, height, &mask_for_edit)?;
        write_region_tiles_clipped(
            tile_layer,
            min_x,
            min_y,
            width,
            height,
            &flipped_tiles,
            Some(&next_mask_for_edit),
        )
    });

    if state.status == "Edit applied." {
        let next_cells = selection_cells_from_mask(min_x, min_y, width, height, &next_mask);
        set_tile_selection_cells(state, next_cells);
        state.status = "Flipped selection on the X axis.".to_string();
    }
}

pub(crate) fn flip_tile_selection_vertically(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        transfer.tiles = flip_tiles_vertically(transfer.width, transfer.height, &transfer.tiles);
        transfer.mask = flip_mask_vertically(transfer.width, transfer.height, &transfer.mask);
        sync_clipboard_from_transfer(state);
        resize_transfer_selection(state);
        state.status = "Flipped moving selection on the Y axis.".to_string();
        return;
    }

    let Some((layer_index, selection, selection_cells)) = selected_tile_selection(state) else {
        state.status = "Select a tile region first.".to_string();
        return;
    };
    let (min_x, min_y, _, _) = selection_bounds(selection);
    let (width, height) = selection_dimensions(selection);
    let mask = selection_mask_from_cells(selection, &selection_cells);
    let next_mask = flip_mask_vertically(width, height, &mask);
    let mask_for_edit = mask.clone();
    let next_mask_for_edit = next_mask.clone();

    apply_edit(state, move |document| {
        let tile_layer = selected_tile_layer_mut(document, layer_index)?;
        let snapshot = capture_region_clipped(tile_layer, min_x, min_y, width, height);
        let flipped_tiles = flip_tiles_vertically(width, height, &snapshot);
        clear_region_tiles_masked(tile_layer, min_x, min_y, width, height, &mask_for_edit)?;
        write_region_tiles_clipped(
            tile_layer,
            min_x,
            min_y,
            width,
            height,
            &flipped_tiles,
            Some(&next_mask_for_edit),
        )
    });

    if state.status == "Edit applied." {
        let next_cells = selection_cells_from_mask(min_x, min_y, width, height, &next_mask);
        set_tile_selection_cells(state, next_cells);
        state.status = "Flipped selection on the Y axis.".to_string();
    }
}

pub(crate) fn rotate_tile_selection_clockwise(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        let old_width = transfer.width;
        let old_height = transfer.height;
        transfer.tiles = rotate_tiles_clockwise(old_width, old_height, &transfer.tiles);
        transfer.mask = rotate_mask_clockwise(old_width, old_height, &transfer.mask);
        transfer.width = old_height;
        transfer.height = old_width;
        sync_clipboard_from_transfer(state);
        resize_transfer_selection(state);
        state.status = "Rotated moving selection clockwise.".to_string();
        return;
    }

    let Some((layer_index, selection, selection_cells)) = selected_tile_selection(state) else {
        state.status = "Select a tile region first.".to_string();
        return;
    };
    let (min_x, min_y, _, _) = selection_bounds(selection);
    let (width, height) = selection_dimensions(selection);
    let new_width = height;
    let new_height = width;
    let mask = selection_mask_from_cells(selection, &selection_cells);
    let next_mask = rotate_mask_clockwise(width, height, &mask);
    let mask_for_edit = mask.clone();
    let next_mask_for_edit = next_mask.clone();

    apply_edit(state, move |document| {
        let tile_layer = selected_tile_layer_mut(document, layer_index)?;
        let snapshot = capture_region_clipped(tile_layer, min_x, min_y, width, height);
        let rotated_tiles = rotate_tiles_clockwise(width, height, &snapshot);
        clear_region_tiles_masked(tile_layer, min_x, min_y, width, height, &mask_for_edit)?;
        write_region_tiles_clipped(
            tile_layer,
            min_x,
            min_y,
            new_width,
            new_height,
            &rotated_tiles,
            Some(&next_mask_for_edit),
        )
    });

    if state.status == "Edit applied." {
        set_tile_selection_cells(
            state,
            selection_cells_from_mask(min_x, min_y, new_width, new_height, &next_mask),
        );
        state.status = "Rotated selection clockwise.".to_string();
    }
}

pub(crate) fn delete_tile_selection(state: &mut AppState) {
    if state.tile_selection_transfer.is_some() {
        delete_tile_selection_source_and_exit(state);
        return;
    }

    let Some((layer_index, selection, selection_cells)) = selected_tile_selection(state) else {
        state.status = "Select a tile region first.".to_string();
        return;
    };
    let (min_x, min_y, _, _) = selection_bounds(selection);
    let (width, height) = selection_dimensions(selection);
    let mask = selection_mask_from_cells(selection, &selection_cells);

    apply_edit(state, move |document| {
        let tile_layer = selected_tile_layer_mut(document, layer_index)?;
        clear_region_tiles_masked(tile_layer, min_x, min_y, width, height, &mask)
    });

    if state.status == "Edit applied." {
        dismiss_tile_selection(state);
        state.status = "Cleared selected region.".to_string();
    }
}

pub(crate) fn place_tile_selection_transfer(state: &mut AppState) {
    if apply_tile_selection_transfer(state, true)
        && let Some(selection) = state.tile_selection
    {
        let (min_x, min_y, _, _) = selection_bounds(selection);
        state.status = format!("Placed selection at ({min_x}, {min_y}).");
    }
}

pub(crate) fn cancel_tile_selection_transfer(state: &mut AppState) {
    let Some(transfer) = state.tile_selection_transfer.take() else {
        return;
    };

    if matches!(transfer.mode, TileSelectionTransferMode::Cut) {
        let (min_x, min_y, _, _) = selection_bounds(transfer.source_selection);
        let restore = {
            let Some(session) = state.session.as_mut() else {
                return;
            };
            session.edit(|document| {
                let tile_layer = selected_tile_layer_mut(document, transfer.source_layer)?;
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
            state.status = "Canceled move, but restoring the cut region failed.".to_string();
        }
    }

    state.tile_selection_closing = None;
    state.tile_selection_closing_cells = None;
    state.tile_selection_closing_started_at = None;
    state.tile_selection_last_tap_at = None;
}

pub(crate) fn apply_shape_fill_rect(
    state: &mut AppState,
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,
) {
    let layer_index = state.active_layer;
    let gid = state.selected_gid;
    let min_x = start_x.min(end_x);
    let max_x = start_x.max(end_x);
    let min_y = start_y.min(end_y);
    let max_y = start_y.max(end_y);

    apply_edit(state, move |document| {
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        if layer.locked() {
            return Err(EditorError::Invalid("layer is locked".to_string()));
        }
        let tile_layer = layer
            .as_tile_mut()
            .ok_or_else(|| EditorError::Invalid("active layer is not a tile layer".to_string()))?;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                tile_layer.set_tile(x, y, gid)?;
            }
        }
        Ok(())
    });
}

fn apply_fill(state: &mut AppState, x: u32, y: u32) {
    let layer_index = state.active_layer;
    let replacement_gid = state.selected_gid;

    apply_edit(state, move |document| {
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        if layer.locked() {
            return Err(EditorError::Invalid("layer is locked".to_string()));
        }
        let tile_layer = layer
            .as_tile_mut()
            .ok_or_else(|| EditorError::Invalid("active layer is not a tile layer".to_string()))?;

        let target_gid = tile_layer.tile_at(x, y).ok_or_else(|| {
            EditorError::Invalid(format!("tile coordinate out of bounds: {x},{y}"))
        })?;
        if target_gid == replacement_gid {
            return Ok(());
        }

        let mut queue = VecDeque::from([(x, y)]);
        let mut visited = vec![false; tile_layer.tiles.len()];

        while let Some((cell_x, cell_y)) = queue.pop_front() {
            let Some(index) = tile_layer.index_of(cell_x, cell_y) else {
                continue;
            };
            if visited[index] {
                continue;
            }
            visited[index] = true;
            if tile_layer.tiles[index] != target_gid {
                continue;
            }

            tile_layer.tiles[index] = replacement_gid;

            if cell_x > 0 {
                queue.push_back((cell_x - 1, cell_y));
            }
            if cell_x + 1 < tile_layer.width {
                queue.push_back((cell_x + 1, cell_y));
            }
            if cell_y > 0 {
                queue.push_back((cell_x, cell_y - 1));
            }
            if cell_y + 1 < tile_layer.height {
                queue.push_back((cell_x, cell_y + 1));
            }
        }

        Ok(())
    });
}

pub(crate) fn create_object(state: &mut AppState, shape: ObjectShape) {
    let cell = state.selected_cell.unwrap_or((0, 0));
    create_object_at(state, shape, cell.0, cell.1);
}

fn create_object_at(state: &mut AppState, shape: ObjectShape, x: u32, y: u32) {
    let layer_index = state.active_layer;
    let mut created = None;
    apply_edit(state, |document| {
        let id = document.map.next_object_id;
        document.map.next_object_id += 1;
        let tile_width = document.map.tile_width as f32;
        let tile_height = document.map.tile_height as f32;
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        if layer.locked() {
            return Err(EditorError::Invalid("layer is locked".to_string()));
        }
        let object_layer = layer.as_object_mut().ok_or_else(|| {
            EditorError::Invalid("active layer is not an object layer".to_string())
        })?;
        object_layer.objects.push(MapObject {
            id,
            name: format!("Object {id}"),
            visible: true,
            x: x as f32 * tile_width,
            y: y as f32 * tile_height,
            width: if matches!(shape, ObjectShape::Rectangle) {
                tile_width
            } else {
                0.0
            },
            height: if matches!(shape, ObjectShape::Rectangle) {
                tile_height
            } else {
                0.0
            },
            shape: shape.clone(),
            properties: Vec::new(),
        });
        created = Some(id);
        Ok(())
    });
    state.selected_object = created;
}

pub(crate) fn nudge_selected_object(state: &mut AppState, dx: f32, dy: f32) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object_layer = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .ok_or_else(|| {
                EditorError::Invalid("active layer is not an object layer".to_string())
            })?;
        let object = object_layer
            .object_mut(object_id)
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        object.x += dx;
        object.y += dy;
        Ok(())
    });
}

pub(crate) fn delete_selected_object(state: &mut AppState) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object_layer = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .ok_or_else(|| {
                EditorError::Invalid("active layer is not an object layer".to_string())
            })?;
        object_layer
            .remove_object(object_id)
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        Ok(())
    });
    state.selected_object = None;
}

pub(crate) fn rename_selected_object(state: &mut AppState, name: String) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        object.name = name;
        Ok(())
    });
}

pub(crate) fn update_selected_object_geometry(
    state: &mut AppState,
    field: &'static str,
    raw: String,
) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let Ok(value) = raw.parse::<f32>() else {
        state.status = format!("Cannot parse '{raw}' as number.");
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        match field {
            "x" => object.x = value,
            "y" => object.y = value,
            "width" => object.width = value.max(0.0),
            "height" => object.height = value.max(0.0),
            _ => {}
        }
        Ok(())
    });
}

pub(crate) fn add_layer_property(state: &mut AppState) {
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let layer = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
        layer.properties_mut().push(Property {
            name: "new_property".to_string(),
            value: PropertyValue::String(String::new()),
        });
        Ok(())
    });
}

pub(crate) fn remove_layer_property(state: &mut AppState, property_index: usize) {
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let properties = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?
            .properties_mut();
        if property_index < properties.len() {
            properties.remove(property_index);
        }
        Ok(())
    });
}

pub(crate) fn rename_layer_property(state: &mut AppState, property_index: usize, name: String) {
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let property = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?
            .properties_mut()
            .get_mut(property_index)
            .ok_or_else(|| {
                EditorError::Invalid(format!("unknown property index {property_index}"))
            })?;
        property.name = name;
        Ok(())
    });
}

pub(crate) fn update_layer_property_value(
    state: &mut AppState,
    property_index: usize,
    raw: String,
) {
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let property = document
            .map
            .layer_mut(layer_index)
            .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?
            .properties_mut()
            .get_mut(property_index)
            .ok_or_else(|| {
                EditorError::Invalid(format!("unknown property index {property_index}"))
            })?;
        property.value = property.value.parse_like(&raw)?;
        Ok(())
    });
}

pub(crate) fn add_object_property(state: &mut AppState) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        object.properties.push(Property {
            name: "new_property".to_string(),
            value: PropertyValue::String(String::new()),
        });
        Ok(())
    });
}

pub(crate) fn remove_object_property(state: &mut AppState, property_index: usize) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let object = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .ok_or_else(|| EditorError::Invalid(format!("unknown object id {object_id}")))?;
        if property_index < object.properties.len() {
            object.properties.remove(property_index);
        }
        Ok(())
    });
}

pub(crate) fn rename_object_property(state: &mut AppState, property_index: usize, name: String) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let property = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .and_then(|object| object.properties.get_mut(property_index))
            .ok_or_else(|| {
                EditorError::Invalid(format!("unknown property index {property_index}"))
            })?;
        property.name = name;
        Ok(())
    });
}

pub(crate) fn update_object_property_value(
    state: &mut AppState,
    property_index: usize,
    raw: String,
) {
    let Some(object_id) = state.selected_object else {
        state.status = "Select an object first.".to_string();
        return;
    };
    let layer_index = state.active_layer;
    apply_edit(state, move |document| {
        let property = document
            .map
            .layer_mut(layer_index)
            .and_then(Layer::as_object_mut)
            .and_then(|layer| layer.object_mut(object_id))
            .and_then(|object| object.properties.get_mut(property_index))
            .ok_or_else(|| {
                EditorError::Invalid(format!("unknown property index {property_index}"))
            })?;
        property.value = property.value.parse_like(&raw)?;
        Ok(())
    });
}

fn selected_tile_selection(
    state: &AppState,
) -> Option<(usize, TileSelectionRegion, BTreeSet<(i32, i32)>)> {
    let selection = state.tile_selection?;
    let selection_cells = state.tile_selection_cells.clone()?;
    state
        .session
        .as_ref()
        .and_then(|session| session.document().map.layer(state.active_layer))
        .and_then(Layer::as_tile)
        .map(|_| (state.active_layer, selection, selection_cells))
}

fn selection_dimensions(selection: TileSelectionRegion) -> (u32, u32) {
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    ((max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32)
}

fn tile_layer_contains_cell(tile_layer: &taled_core::TileLayer, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && (x as u32) < tile_layer.width && (y as u32) < tile_layer.height
}

fn selected_tile_layer_mut(
    document: &mut taled_core::EditorDocument,
    layer_index: usize,
) -> Result<&mut taled_core::TileLayer, EditorError> {
    let layer = document
        .map
        .layer_mut(layer_index)
        .ok_or_else(|| EditorError::Invalid(format!("unknown layer index {layer_index}")))?;
    if layer.locked() {
        return Err(EditorError::Invalid("layer is locked".to_string()));
    }
    layer
        .as_tile_mut()
        .ok_or_else(|| EditorError::Invalid("active layer is not a tile layer".to_string()))
}

fn capture_region_clipped(
    tile_layer: &taled_core::TileLayer,
    min_x: i32,
    min_y: i32,
    width: u32,
    height: u32,
) -> Vec<u32> {
    let mut tiles = Vec::with_capacity((width * height) as usize);
    for local_y in 0..height {
        for local_x in 0..width {
            let x = min_x + local_x as i32;
            let y = min_y + local_y as i32;
            let gid = if tile_layer_contains_cell(tile_layer, x, y) {
                tile_layer.tile_at(x as u32, y as u32).unwrap_or(0)
            } else {
                0
            };
            tiles.push(gid);
        }
    }
    tiles
}

fn flip_tiles_horizontally(width: u32, height: u32, tiles: &[u32]) -> Vec<u32> {
    let mut flipped = vec![0; (width * height) as usize];
    for local_y in 0..height {
        for local_x in 0..width {
            let source_index = (local_y * width + (width - 1 - local_x)) as usize;
            let dest_index = (local_y * width + local_x) as usize;
            flipped[dest_index] = tiles[source_index];
        }
    }
    flipped
}

fn flip_tiles_vertically(width: u32, height: u32, tiles: &[u32]) -> Vec<u32> {
    let mut flipped = vec![0; (width * height) as usize];
    for local_y in 0..height {
        for local_x in 0..width {
            let source_index = ((height - 1 - local_y) * width + local_x) as usize;
            let dest_index = (local_y * width + local_x) as usize;
            flipped[dest_index] = tiles[source_index];
        }
    }
    flipped
}

fn rotate_tiles_clockwise(width: u32, height: u32, tiles: &[u32]) -> Vec<u32> {
    let mut rotated = vec![0; (width * height) as usize];
    for source_y in 0..height {
        for source_x in 0..width {
            let source_index = (source_y * width + source_x) as usize;
            let dest_x = height - 1 - source_y;
            let dest_y = source_x;
            let dest_index = (dest_y * height + dest_x) as usize;
            rotated[dest_index] = tiles[source_index];
        }
    }
    rotated
}

fn flip_mask_horizontally(width: u32, height: u32, mask: &[bool]) -> Vec<bool> {
    let mut flipped = vec![false; (width * height) as usize];
    for local_y in 0..height {
        for local_x in 0..width {
            let source_index = (local_y * width + (width - 1 - local_x)) as usize;
            let dest_index = (local_y * width + local_x) as usize;
            flipped[dest_index] = mask.get(source_index).copied().unwrap_or(false);
        }
    }
    flipped
}

fn flip_mask_vertically(width: u32, height: u32, mask: &[bool]) -> Vec<bool> {
    let mut flipped = vec![false; (width * height) as usize];
    for local_y in 0..height {
        for local_x in 0..width {
            let source_index = ((height - 1 - local_y) * width + local_x) as usize;
            let dest_index = (local_y * width + local_x) as usize;
            flipped[dest_index] = mask.get(source_index).copied().unwrap_or(false);
        }
    }
    flipped
}

fn rotate_mask_clockwise(width: u32, height: u32, mask: &[bool]) -> Vec<bool> {
    let mut rotated = vec![false; (width * height) as usize];
    for source_y in 0..height {
        for source_x in 0..width {
            let source_index = (source_y * width + source_x) as usize;
            let dest_x = height - 1 - source_y;
            let dest_y = source_x;
            let dest_index = (dest_y * height + dest_x) as usize;
            rotated[dest_index] = mask.get(source_index).copied().unwrap_or(false);
        }
    }
    rotated
}

fn sync_clipboard_from_transfer(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_ref() {
        state.tile_clipboard = Some(TileClipboard {
            width: transfer.width,
            height: transfer.height,
            tiles: transfer.tiles.clone(),
            mask: transfer.mask.clone(),
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
    let (min_x, min_y, _, _) = selection_bounds(selection);
    let selection_cells = selection_cells_from_mask(
        min_x,
        min_y,
        transfer.width,
        transfer.height,
        &transfer.mask,
    );
    set_tile_selection_cells(state, selection_cells);
}

fn apply_tile_selection_transfer(state: &mut AppState, finalize: bool) -> bool {
    let Some(transfer) = state.tile_selection_transfer.clone() else {
        state.status = "Nothing to place.".to_string();
        return false;
    };
    let Some(selection) = state.tile_selection else {
        state.status = "Move the selection before placing it.".to_string();
        return false;
    };
    let target_layer = state.active_layer;
    let (min_x, min_y, _, _) = selection_bounds(selection);

    if transfer.source_layer != target_layer {
        cancel_tile_selection_transfer(state);
        state.status = "Selection move canceled because the active layer changed.".to_string();
        return false;
    }

    let apply_result = if matches!(transfer.mode, TileSelectionTransferMode::Cut) {
        let Some(session) = state.session.as_mut() else {
            state.status = "No map loaded.".to_string();
            return false;
        };
        let result = session.edit(|document| {
            let tile_layer = selected_tile_layer_mut(document, target_layer)?;
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
        match (result.is_ok(), finalize) {
            (true, true) => {
                session.finish_history_batch();
            }
            (false, true) => {
                session.abort_history_batch();
            }
            _ => {}
        }
        result
    } else {
        let tiles = transfer.tiles.clone();
        let width = transfer.width;
        let height = transfer.height;
        let mask = transfer.mask.clone();
        apply_edit_result(state, move |document| {
            let tile_layer = selected_tile_layer_mut(document, target_layer)?;
            write_region_tiles_clipped(tile_layer, min_x, min_y, width, height, &tiles, Some(&mask))
        })
    };

    match apply_result {
        Ok(()) => {
            let selection_cells = selection_cells_from_mask(
                min_x,
                min_y,
                transfer.width,
                transfer.height,
                &transfer.mask,
            );
            set_tile_selection_cells(state, selection_cells);
            if finalize {
                state.tile_selection_transfer = None;
            }
            true
        }
        Err(error) => {
            if finalize {
                state.tile_selection_transfer = None;
            }
            state.status = format!("Place failed: {error}");
            false
        }
    }
}

fn delete_tile_selection_source_and_exit(state: &mut AppState) {
    let Some(transfer) = state.tile_selection_transfer.clone() else {
        state.status = "Nothing to delete.".to_string();
        return;
    };
    let (min_x, min_y, _, _) = selection_bounds(transfer.source_selection);

    if matches!(transfer.mode, TileSelectionTransferMode::Cut) {
        if let Some(session) = state.session.as_mut() {
            session.abort_history_batch();
        }
        state.tile_selection_transfer = None;
        state.tile_selection = None;
        state.tile_selection_cells = None;
        state.tile_selection_preview = None;
        state.tile_selection_preview_cells = None;
        state.status = "Deleted the source selection and exited move mode.".to_string();
        return;
    }

    let deleted = apply_edit_result(state, move |document| {
        let tile_layer = selected_tile_layer_mut(document, transfer.source_layer)?;
        clear_region_tiles_masked(
            tile_layer,
            min_x,
            min_y,
            transfer.width,
            transfer.height,
            &transfer.source_mask,
        )
    });

    match deleted {
        Ok(()) => {
            state.tile_selection_transfer = None;
            state.tile_selection = None;
            state.tile_selection_cells = None;
            state.tile_selection_preview = None;
            state.tile_selection_preview_cells = None;
            state.status = "Deleted the source selection and exited move mode.".to_string();
        }
        Err(error) => {
            state.status = format!("Delete failed: {error}");
        }
    }
}

fn write_region_tiles_clipped(
    tile_layer: &mut taled_core::TileLayer,
    min_x: i32,
    min_y: i32,
    width: u32,
    height: u32,
    tiles: &[u32],
    mask: Option<&[bool]>,
) -> Result<(), EditorError> {
    for local_y in 0..height {
        for local_x in 0..width {
            let index = (local_y * width + local_x) as usize;
            if mask.is_some_and(|mask| !mask.get(index).copied().unwrap_or(false)) {
                continue;
            }
            let gid = tiles[index];
            let x = min_x + local_x as i32;
            let y = min_y + local_y as i32;
            if tile_layer_contains_cell(tile_layer, x, y) {
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
) -> Result<(), EditorError> {
    for local_y in 0..height {
        for local_x in 0..width {
            let index = (local_y * width + local_x) as usize;
            if !mask.get(index).copied().unwrap_or(false) {
                continue;
            }
            let x = min_x + local_x as i32;
            let y = min_y + local_y as i32;
            if tile_layer_contains_cell(tile_layer, x, y) {
                tile_layer.set_tile(x as u32, y as u32, 0)?;
            }
        }
    }
    Ok(())
}

fn capture_tile_selection_transfer(
    state: &mut AppState,
) -> Option<(TileSelectionTransfer, TileClipboard)> {
    let Some((layer_index, selection, selection_cells)) = selected_tile_selection(state) else {
        state.status = "Select a tile region first.".to_string();
        return None;
    };
    let Some(session) = state.session.as_ref() else {
        state.status = "No map loaded.".to_string();
        return None;
    };
    let Some(tile_layer) = session
        .document()
        .map
        .layer(layer_index)
        .and_then(Layer::as_tile)
    else {
        state.status = "Active layer is not a tile layer.".to_string();
        return None;
    };

    let (min_x, min_y, _, _) = selection_bounds(selection);
    let (width, height) = selection_dimensions(selection);
    let mask = selection_mask_from_cells(selection, &selection_cells);
    let mut tiles = capture_region_clipped(tile_layer, min_x, min_y, width, height);
    for (index, selected) in mask.iter().copied().enumerate() {
        if !selected {
            tiles[index] = 0;
        }
    }
    let clipboard = TileClipboard {
        width,
        height,
        tiles: tiles.clone(),
        mask: mask.clone(),
    };
    let transfer = TileSelectionTransfer {
        source_layer: layer_index,
        source_selection: selection,
        source_mask: mask.clone(),
        width,
        height,
        tiles,
        mask,
        mode: TileSelectionTransferMode::Copy,
    };

    Some((transfer, clipboard))
}

pub(crate) fn selected_object_view(
    session: &EditorSession,
    selected_object: Option<u32>,
    layer_index: usize,
) -> Option<(&MapObject, usize)> {
    let object_id = selected_object?;
    let layer = session.document().map.layer(layer_index)?.as_object()?;
    let object = layer.object(object_id)?;
    Some((object, layer_index))
}

pub(crate) fn apply_edit<F>(state: &mut AppState, edit: F)
where
    F: FnOnce(&mut taled_core::EditorDocument) -> Result<(), EditorError>,
{
    match apply_edit_result(state, edit) {
        Ok(()) => state.status = "Edit applied.".to_string(),
        Err(error) => state.status = format!("Edit failed: {error}"),
    }
}

fn apply_edit_result<F>(state: &mut AppState, edit: F) -> Result<(), EditorError>
where
    F: FnOnce(&mut taled_core::EditorDocument) -> Result<(), EditorError>,
{
    let Some(session) = state.session.as_mut() else {
        state.status = "No map loaded.".to_string();
        return Err(EditorError::Invalid("No map loaded.".to_string()));
    };

    session.edit(edit)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use taled_core::EditorSession;

    use super::{
        apply_cell_tool, apply_magic_wand_selection, apply_select_same_tile_selection,
        apply_shape_fill_rect, apply_tile_selection_mode_region, copy_tile_selection,
        cut_tile_selection, delete_tile_selection, flip_tile_selection_horizontally,
        flip_tile_selection_vertically, handle_tile_selection_tap, place_tile_selection_transfer,
        rotate_tile_selection_clockwise, select_tile_region,
    };
    use crate::app_state::{
        AppState, TileSelectionMode, TileSelectionRegion, TileSelectionTransferMode, Tool,
    };

    fn sample_map_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace apps dir")
            .parent()
            .expect("workspace root")
            .join("assets")
            .join("samples")
            .join("stage1-basic")
            .join("map.tmx")
    }

    fn test_state(tool: Tool, selected_gid: u32) -> AppState {
        AppState {
            session: Some(EditorSession::load(sample_map_path()).expect("sample map should load")),
            active_layer: 0,
            selected_gid,
            tool,
            ..AppState::default()
        }
    }

    #[test]
    fn fill_replaces_a_connected_region_in_one_edit() {
        let mut state = test_state(Tool::Fill, 9);

        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(0, 0, 1)?;
                    layer.set_tile(1, 0, 1)?;
                    layer.set_tile(0, 1, 1)?;
                    layer.set_tile(1, 1, 1)?;
                    layer.set_tile(2, 0, 2)?;
                    layer.set_tile(2, 1, 2)?;
                    Ok(())
                })
                .expect("seed region");
        }

        apply_cell_tool(&mut state, 0, 0);

        let session = state.session.as_ref().expect("session");
        let layer = session.document().map.layers[0]
            .as_tile()
            .expect("tile layer");
        assert_eq!(layer.tile_at(0, 0), Some(9));
        assert_eq!(layer.tile_at(1, 0), Some(9));
        assert_eq!(layer.tile_at(0, 1), Some(9));
        assert_eq!(layer.tile_at(1, 1), Some(9));
        assert_eq!(layer.tile_at(2, 0), Some(2));
        assert!(session.can_undo());
        assert!(!session.can_redo());
    }

    #[test]
    fn shape_fill_paints_the_requested_rectangle() {
        let mut state = test_state(Tool::ShapeFill, 7);

        apply_shape_fill_rect(&mut state, 1, 1, 3, 2);

        let session = state.session.as_ref().expect("session");
        let layer = session.document().map.layers[0]
            .as_tile()
            .expect("tile layer");
        for y in 1..=2 {
            for x in 1..=3 {
                assert_eq!(layer.tile_at(x, y), Some(7), "cell {x},{y}");
            }
        }
        assert_eq!(layer.tile_at(0, 0), Some(1));
        assert_eq!(layer.tile_at(5, 4), Some(2));
        assert!(session.can_undo());
        assert!(!session.can_redo());
    }

    #[test]
    fn magic_wand_selects_only_the_connected_matching_region() {
        let mut state = test_state(Tool::MagicWand, 1);

        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(0, 0, 97)?;
                    layer.set_tile(1, 0, 97)?;
                    layer.set_tile(0, 1, 97)?;
                    layer.set_tile(1, 1, 97)?;
                    layer.set_tile(4, 3, 97)?;
                    layer.set_tile(3, 4, 97)?;
                    layer.set_tile(4, 4, 97)?;
                    Ok(())
                })
                .expect("seed region");
        }

        let applied = apply_magic_wand_selection(&mut state, 0, 0, None);

        assert!(applied);
        let cells = state.tile_selection_cells.expect("selection cells");
        assert!(cells.contains(&(0, 0)));
        assert!(cells.contains(&(1, 1)));
        assert!(!cells.contains(&(4, 4)));
        assert_eq!(cells.len(), 4);
    }

    #[test]
    fn select_same_tile_selects_all_cells_matching_sampled_tiles() {
        let mut state = test_state(Tool::SelectSameTile, 1);

        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(0, 0, 96)?;
                    layer.set_tile(2, 1, 96)?;
                    layer.set_tile(4, 3, 96)?;
                    layer.set_tile(4, 4, 2)?;
                    Ok(())
                })
                .expect("seed same-tile region");
        }

        let applied = apply_select_same_tile_selection(&mut state, 0, 0, None);

        assert!(applied);
        let cells = state.tile_selection_cells.expect("selection cells");
        assert!(cells.contains(&(0, 0)));
        assert!(cells.contains(&(2, 1)));
        assert!(cells.contains(&(4, 3)));
        assert!(!cells.contains(&(4, 4)));
    }

    #[test]
    fn sampling_tools_ignore_empty_tiles() {
        let mut magic_state = test_state(Tool::MagicWand, 1);
        let mut same_state = test_state(Tool::SelectSameTile, 1);

        for state in [&mut magic_state, &mut same_state] {
            state
                .session
                .as_mut()
                .expect("session")
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(1, 1, 0)?;
                    Ok(())
                })
                .expect("clear sample tile");
        }

        assert!(!apply_magic_wand_selection(&mut magic_state, 1, 1, None));
        assert!(magic_state.tile_selection.is_none());
        assert!(magic_state.tile_selection_cells.is_none());

        assert!(!apply_select_same_tile_selection(&mut same_state, 1, 1, None));
        assert!(same_state.tile_selection.is_none());
        assert!(same_state.tile_selection_cells.is_none());
    }

    #[test]
    fn same_tile_add_mode_can_union_multiple_sampled_tile_values() {
        let mut state = test_state(Tool::SelectSameTile, 1);

        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(0, 0, 7)?;
                    layer.set_tile(2, 0, 7)?;
                    layer.set_tile(0, 2, 8)?;
                    layer.set_tile(2, 2, 8)?;
                    Ok(())
                })
                .expect("seed multiple tiles");
        }

        let sampled = BTreeSet::from([7, 8]);
        state.tile_selection_mode = TileSelectionMode::Add;
        select_tile_region(&mut state, 4, 4, 4, 4);

        let applied = apply_select_same_tile_selection(&mut state, 0, 0, Some(&sampled));

        assert!(applied);
        let cells = state.tile_selection_cells.expect("selection cells");
        assert!(cells.contains(&(4, 4)));
        assert!(cells.contains(&(0, 0)));
        assert!(cells.contains(&(2, 0)));
        assert!(cells.contains(&(0, 2)));
        assert!(cells.contains(&(2, 2)));
    }

    #[test]
    fn tile_region_selection_tracks_multicell_bounds() {
        let mut state = test_state(Tool::Select, 1);

        select_tile_region(&mut state, 2, 3, 5, 7);

        assert_eq!(
            state.tile_selection,
            Some(crate::app_state::TileSelectionRegion {
                start_cell: (2, 3),
                end_cell: (5, 7),
            })
        );
        assert_eq!(state.tile_selection_preview, None);
        assert_eq!(state.selected_cell, None);
        assert!(state.status.contains("4x5"));
    }

    #[test]
    fn add_selection_mode_unions_the_new_region() {
        let mut state = test_state(Tool::Select, 1);
        select_tile_region(&mut state, 1, 1, 2, 2);
        state.tile_selection_mode = TileSelectionMode::Add;

        apply_tile_selection_mode_region(&mut state, 3, 2, 4, 3);

        let selection = state.tile_selection.expect("selection");
        assert_eq!(selection.start_cell, (1, 1));
        assert_eq!(selection.end_cell, (4, 3));
        let cells = state.tile_selection_cells.expect("selection cells");
        assert!(cells.contains(&(1, 1)));
        assert!(cells.contains(&(4, 3)));
        assert_eq!(cells.len(), 8);
    }

    #[test]
    fn subtract_selection_mode_removes_overlapping_cells() {
        let mut state = test_state(Tool::Select, 1);
        select_tile_region(&mut state, 1, 1, 3, 3);
        state.tile_selection_mode = TileSelectionMode::Subtract;

        apply_tile_selection_mode_region(&mut state, 2, 2, 3, 3);

        let cells = state.tile_selection_cells.expect("selection cells");
        assert!(!cells.contains(&(2, 2)));
        assert!(!cells.contains(&(3, 3)));
        assert!(cells.contains(&(1, 1)));
        assert_eq!(cells.len(), 5);
    }

    #[test]
    fn intersect_selection_mode_keeps_only_the_overlap() {
        let mut state = test_state(Tool::Select, 1);
        select_tile_region(&mut state, 1, 1, 4, 3);
        state.tile_selection_mode = TileSelectionMode::Intersect;

        apply_tile_selection_mode_region(&mut state, 3, 2, 5, 4);

        let selection = state.tile_selection.expect("selection");
        assert_eq!(selection.start_cell, (3, 2));
        assert_eq!(selection.end_cell, (4, 3));
        let cells = state.tile_selection_cells.expect("selection cells");
        assert_eq!(cells.len(), 4);
        assert!(cells.contains(&(3, 2)));
        assert!(cells.contains(&(4, 3)));
        assert!(!cells.contains(&(2, 2)));
    }

    #[test]
    fn add_mode_tap_outside_selection_adds_a_single_cell() {
        let mut state = test_state(Tool::Select, 1);
        select_tile_region(&mut state, 1, 1, 2, 2);
        state.tile_selection_mode = TileSelectionMode::Add;

        assert!(handle_tile_selection_tap(&mut state, 4, 4));

        let cells = state.tile_selection_cells.expect("selection cells");
        assert!(cells.contains(&(1, 1)));
        assert!(cells.contains(&(4, 4)));
        assert_eq!(cells.len(), 5);
    }

    #[test]
    fn subtract_mode_tap_inside_selection_removes_a_single_cell() {
        let mut state = test_state(Tool::Select, 1);
        select_tile_region(&mut state, 1, 1, 2, 2);
        state.tile_selection_mode = TileSelectionMode::Subtract;

        assert!(handle_tile_selection_tap(&mut state, 2, 2));

        let cells = state.tile_selection_cells.expect("selection cells");
        assert!(!cells.contains(&(2, 2)));
        assert!(cells.contains(&(1, 1)));
        assert_eq!(cells.len(), 3);
    }

    #[test]
    fn tapping_outside_an_existing_selection_clears_without_creating_a_new_one() {
        let mut state = test_state(Tool::Select, 1);
        select_tile_region(&mut state, 2, 3, 5, 7);

        assert!(handle_tile_selection_tap(&mut state, 0, 0));
        assert_eq!(state.tile_selection, None);
        assert!(state.tile_selection_closing.is_some());
    }

    #[test]
    fn single_tap_inside_an_existing_selection_keeps_it_active() {
        let mut state = test_state(Tool::Select, 1);
        select_tile_region(&mut state, 2, 3, 5, 7);

        assert!(handle_tile_selection_tap(&mut state, 3, 4));
        assert_eq!(
            state.tile_selection,
            Some(TileSelectionRegion {
                start_cell: (2, 3),
                end_cell: (5, 7),
            })
        );
        assert!(state.tile_selection_last_tap_at.is_some());
    }

    #[test]
    fn double_tap_inside_an_existing_selection_closes_it() {
        let mut state = test_state(Tool::Select, 1);
        select_tile_region(&mut state, 2, 3, 5, 7);

        assert!(handle_tile_selection_tap(&mut state, 3, 4));
        assert!(handle_tile_selection_tap(&mut state, 3, 4));
        assert_eq!(state.tile_selection, None);
        assert!(state.tile_selection_closing.is_some());
    }

    #[test]
    fn copy_tile_selection_stores_region_tiles_in_clipboard() {
        let mut state = test_state(Tool::Select, 1);
        select_tile_region(&mut state, 1, 1, 2, 2);

        copy_tile_selection(&mut state);

        let clipboard = state.tile_clipboard.expect("clipboard");
        assert_eq!(clipboard.width, 2);
        assert_eq!(clipboard.height, 2);
        assert_eq!(clipboard.tiles.len(), 4);
        let transfer = state.tile_selection_transfer.expect("transfer");
        assert_eq!(transfer.mode, TileSelectionTransferMode::Copy);
    }

    #[test]
    fn copy_move_places_tiles_at_the_new_selection_region() {
        let mut state = test_state(Tool::Select, 1);
        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(1, 1, 51)?;
                    layer.set_tile(2, 1, 52)?;
                    Ok(())
                })
                .expect("seed row");
        }
        select_tile_region(&mut state, 1, 1, 2, 1);

        copy_tile_selection(&mut state);
        state.tile_selection = Some(TileSelectionRegion {
            start_cell: (4, 2),
            end_cell: (5, 2),
        });
        place_tile_selection_transfer(&mut state);

        let session = state.session.as_ref().expect("session");
        let layer = session.document().map.layers[0]
            .as_tile()
            .expect("tile layer");
        assert_eq!(layer.tile_at(1, 1), Some(51));
        assert_eq!(layer.tile_at(2, 1), Some(52));
        assert_eq!(layer.tile_at(4, 2), Some(51));
        assert_eq!(layer.tile_at(5, 2), Some(52));
    }

    #[test]
    fn copy_in_move_mode_stamps_current_position_and_stays_active() {
        let mut state = test_state(Tool::Select, 1);
        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(1, 1, 71)?;
                    layer.set_tile(2, 1, 72)?;
                    Ok(())
                })
                .expect("seed row");
        }
        select_tile_region(&mut state, 1, 1, 2, 1);
        copy_tile_selection(&mut state);

        state.tile_selection = Some(TileSelectionRegion {
            start_cell: (4, 2),
            end_cell: (5, 2),
        });
        copy_tile_selection(&mut state);

        let session = state.session.as_ref().expect("session");
        let layer = session.document().map.layers[0]
            .as_tile()
            .expect("tile layer");
        assert_eq!(layer.tile_at(4, 2), Some(71));
        assert_eq!(layer.tile_at(5, 2), Some(72));
        assert!(state.tile_selection_transfer.is_some());
    }

    #[test]
    fn cut_move_clears_source_then_places_and_undoes_as_one_step() {
        let mut state = test_state(Tool::Select, 1);
        let original_target_tiles = {
            let session = state.session.as_ref().expect("session");
            let layer = session.document().map.layers[0]
                .as_tile()
                .expect("tile layer");
            (
                layer.tile_at(4, 2).expect("target tile"),
                layer.tile_at(5, 2).expect("target tile"),
            )
        };
        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(1, 1, 61)?;
                    layer.set_tile(2, 1, 62)?;
                    Ok(())
                })
                .expect("seed row");
        }
        select_tile_region(&mut state, 1, 1, 2, 1);

        cut_tile_selection(&mut state);
        {
            let session = state.session.as_ref().expect("session");
            let layer = session.document().map.layers[0]
                .as_tile()
                .expect("tile layer");
            assert_eq!(layer.tile_at(1, 1), Some(0));
            assert_eq!(layer.tile_at(2, 1), Some(0));
        }

        state.tile_selection = Some(TileSelectionRegion {
            start_cell: (4, 2),
            end_cell: (5, 2),
        });
        place_tile_selection_transfer(&mut state);

        {
            let session = state.session.as_ref().expect("session");
            let layer = session.document().map.layers[0]
                .as_tile()
                .expect("tile layer");
            assert_eq!(layer.tile_at(1, 1), Some(0));
            assert_eq!(layer.tile_at(2, 1), Some(0));
            assert_eq!(layer.tile_at(4, 2), Some(61));
            assert_eq!(layer.tile_at(5, 2), Some(62));
        }

        let session = state.session.as_mut().expect("session");
        assert!(session.undo());
        let layer = session.document().map.layers[0]
            .as_tile()
            .expect("tile layer");
        assert_eq!(layer.tile_at(1, 1), Some(61));
        assert_eq!(layer.tile_at(2, 1), Some(62));
        assert_eq!(layer.tile_at(4, 2), Some(original_target_tiles.0));
        assert_eq!(layer.tile_at(5, 2), Some(original_target_tiles.1));
    }

    #[test]
    fn delete_in_move_mode_clears_the_source_region_and_exits() {
        let mut state = test_state(Tool::Select, 1);
        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(1, 1, 81)?;
                    layer.set_tile(2, 1, 82)?;
                    Ok(())
                })
                .expect("seed row");
        }
        select_tile_region(&mut state, 1, 1, 2, 1);
        copy_tile_selection(&mut state);
        state.tile_selection = Some(TileSelectionRegion {
            start_cell: (4, 2),
            end_cell: (5, 2),
        });

        delete_tile_selection(&mut state);

        let session = state.session.as_ref().expect("session");
        let layer = session.document().map.layers[0]
            .as_tile()
            .expect("tile layer");
        assert_eq!(layer.tile_at(1, 1), Some(0));
        assert_eq!(layer.tile_at(2, 1), Some(0));
        assert!(state.tile_selection_transfer.is_none());
        assert!(state.tile_selection.is_none());
    }

    #[test]
    fn flip_tile_selection_horizontally_swaps_cells_across_region() {
        let mut state = test_state(Tool::Select, 1);
        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(1, 1, 11)?;
                    layer.set_tile(2, 1, 12)?;
                    Ok(())
                })
                .expect("seed row");
        }
        select_tile_region(&mut state, 1, 1, 2, 1);

        flip_tile_selection_horizontally(&mut state);

        let layer = state
            .session
            .as_ref()
            .expect("session")
            .document()
            .map
            .layers[0]
            .as_tile()
            .expect("tile layer");
        assert_eq!(layer.tile_at(1, 1), Some(12));
        assert_eq!(layer.tile_at(2, 1), Some(11));
    }

    #[test]
    fn flip_tile_selection_vertically_swaps_cells_across_rows() {
        let mut state = test_state(Tool::Select, 1);
        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(1, 1, 31)?;
                    layer.set_tile(1, 2, 32)?;
                    Ok(())
                })
                .expect("seed column");
        }
        select_tile_region(&mut state, 1, 1, 1, 2);

        flip_tile_selection_vertically(&mut state);

        let layer = state
            .session
            .as_ref()
            .expect("session")
            .document()
            .map
            .layers[0]
            .as_tile()
            .expect("tile layer");
        assert_eq!(layer.tile_at(1, 1), Some(32));
        assert_eq!(layer.tile_at(1, 2), Some(31));
    }

    #[test]
    fn rotate_tile_selection_clockwise_updates_tiles_and_bounds() {
        let mut state = test_state(Tool::Select, 1);
        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(1, 1, 21)?;
                    layer.set_tile(2, 1, 22)?;
                    layer.set_tile(1, 2, 23)?;
                    layer.set_tile(2, 2, 24)?;
                    layer.set_tile(1, 3, 25)?;
                    layer.set_tile(2, 3, 26)?;
                    Ok(())
                })
                .expect("seed region");
        }
        select_tile_region(&mut state, 1, 1, 2, 3);

        rotate_tile_selection_clockwise(&mut state);

        let layer = state
            .session
            .as_ref()
            .expect("session")
            .document()
            .map
            .layers[0]
            .as_tile()
            .expect("tile layer");
        assert_eq!(layer.tile_at(3, 1), Some(21));
        assert_eq!(layer.tile_at(3, 2), Some(22));
        assert_eq!(layer.tile_at(2, 1), Some(23));
        assert_eq!(layer.tile_at(2, 2), Some(24));
        assert_eq!(layer.tile_at(1, 1), Some(25));
        assert_eq!(layer.tile_at(1, 2), Some(26));
        assert_eq!(
            state.tile_selection,
            Some(crate::app_state::TileSelectionRegion {
                start_cell: (1, 1),
                end_cell: (3, 2),
            })
        );
    }

    #[test]
    fn rotate_tile_selection_clockwise_can_extend_beyond_map_bounds() {
        let mut state = test_state(Tool::Select, 1);
        let map_width = state
            .session
            .as_ref()
            .expect("session")
            .document()
            .map
            .width;
        let last_x = map_width - 1;
        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(last_x, 0, 91)?;
                    layer.set_tile(last_x, 1, 92)?;
                    Ok(())
                })
                .expect("seed edge column");
        }
        select_tile_region(&mut state, last_x as i32, 0, last_x as i32, 1);

        rotate_tile_selection_clockwise(&mut state);

        let layer = state
            .session
            .as_ref()
            .expect("session")
            .document()
            .map
            .layers[0]
            .as_tile()
            .expect("tile layer");
        assert_eq!(layer.tile_at(last_x, 0), Some(92));
        assert_eq!(layer.tile_at(last_x, 1), Some(0));
        assert_eq!(
            state.tile_selection,
            Some(TileSelectionRegion {
                start_cell: (last_x as i32, 0),
                end_cell: (last_x as i32 + 1, 0),
            })
        );
    }

    #[test]
    fn delete_tile_selection_clears_region_tiles() {
        let mut state = test_state(Tool::Select, 1);
        select_tile_region(&mut state, 1, 1, 2, 2);

        delete_tile_selection(&mut state);

        let layer = state
            .session
            .as_ref()
            .expect("session")
            .document()
            .map
            .layers[0]
            .as_tile()
            .expect("tile layer");
        for y in 1..=2 {
            for x in 1..=2 {
                assert_eq!(layer.tile_at(x, y), Some(0));
            }
        }
    }

    #[test]
    fn placing_selection_clips_tiles_that_extend_past_the_map_edge() {
        let mut state = test_state(Tool::Select, 1);
        let map_width = state
            .session
            .as_ref()
            .expect("session")
            .document()
            .map
            .width;
        let last_x = map_width - 1;

        if let Some(session) = state.session.as_mut() {
            session
                .edit(|document| {
                    let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
                    layer.set_tile(1, 1, 51)?;
                    layer.set_tile(2, 1, 52)?;
                    layer.set_tile(last_x, 2, 7)?;
                    Ok(())
                })
                .expect("seed source tiles");
        }

        select_tile_region(&mut state, 1, 1, 2, 1);
        copy_tile_selection(&mut state);
        state.tile_selection = Some(TileSelectionRegion {
            start_cell: (last_x as i32, 2),
            end_cell: (last_x as i32 + 1, 2),
        });

        place_tile_selection_transfer(&mut state);

        let layer = state
            .session
            .as_ref()
            .expect("session")
            .document()
            .map
            .layers[0]
            .as_tile()
            .expect("tile layer");
        assert_eq!(layer.tile_at(last_x, 2), Some(51));
        assert_eq!(
            state.tile_selection,
            Some(TileSelectionRegion {
                start_cell: (last_x as i32, 2),
                end_cell: (last_x as i32 + 1, 2),
            })
        );
    }
}
