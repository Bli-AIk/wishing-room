use std::collections::VecDeque;

use taled_core::{
    EditorError, EditorSession, Layer, MapObject, ObjectShape, Property, PropertyValue,
};

use crate::app_state::{
    AppState, TileClipboard, TileSelectionRegion, TileSelectionTransfer,
    TileSelectionTransferMode, Tool,
};

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
                select_tile_region(state, x, y, x, y);
            }
        }
        Tool::AddRectangle => create_object_at(state, ObjectShape::Rectangle, x, y),
        Tool::AddPoint => create_object_at(state, ObjectShape::Point, x, y),
    }
}

pub(crate) fn select_tile_region(
    state: &mut AppState,
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,
) {
    state.tile_selection = Some(crate::app_state::TileSelectionRegion {
        start_cell: (start_x, start_y),
        end_cell: (end_x, end_y),
    });
    state.tile_selection_preview = None;
    state.selected_object = None;
    state.selected_cell = None;

    let width = start_x.abs_diff(end_x) + 1;
    let height = start_y.abs_diff(end_y) + 1;
    state.status = format!(
        "Selected region {}x{} from ({}, {}) to ({}, {}).",
        width, height, start_x, start_y, end_x, end_y
    );
}

pub(crate) fn copy_tile_selection(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_ref() {
        state.tile_clipboard = Some(TileClipboard {
            width: transfer.width,
            height: transfer.height,
            tiles: transfer.tiles.clone(),
        });
        state.status = format!(
            "Copied moving region {}x{}.",
            transfer.width, transfer.height
        );
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
    state.selected_object = None;
    state.selected_cell = None;
    state.status = format!(
        "Copied region {}x{}. Drag to place.",
        width, height
    );
}

pub(crate) fn cut_tile_selection(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        if matches!(transfer.mode, TileSelectionTransferMode::Cut) {
            state.status = "Selection is already in cut-move mode.".to_string();
            return;
        }

        let (min_x, min_y, max_x, max_y) = selection_bounds(transfer.source_selection);
        let Some(session) = state.session.as_mut() else {
            state.status = "No map loaded.".to_string();
            return;
        };

        session.begin_history_batch();
        let clear_result = session.edit(|document| {
            let tile_layer = selected_tile_layer_mut(document, transfer.source_layer)?;
            clear_region_tiles(tile_layer, min_x, min_y, max_x, max_y)
        });

        match clear_result {
            Ok(()) => {
                transfer.mode = TileSelectionTransferMode::Cut;
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
    let (min_x, min_y, max_x, max_y) = selection_bounds(transfer.source_selection);
    let Some(session) = state.session.as_mut() else {
        state.status = "No map loaded.".to_string();
        return;
    };

    session.begin_history_batch();
    let clear_result = session.edit(|document| {
        let tile_layer = selected_tile_layer_mut(document, transfer.source_layer)?;
        clear_region_tiles(tile_layer, min_x, min_y, max_x, max_y)
    });

    match clear_result {
        Ok(()) => {
            state.tile_clipboard = Some(clipboard);
            state.tile_selection_transfer = Some(TileSelectionTransfer {
                mode: TileSelectionTransferMode::Cut,
                ..transfer
            });
            state.tile_selection_preview = None;
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
        for local_y in 0..transfer.height {
            for local_x in 0..(transfer.width / 2) {
                let left = (local_y * transfer.width + local_x) as usize;
                let right = (local_y * transfer.width + (transfer.width - 1 - local_x)) as usize;
                transfer.tiles.swap(left, right);
            }
        }
        sync_clipboard_from_transfer(state);
        state.status = "Flipped moving selection on the X axis.".to_string();
        return;
    }

    let Some((layer_index, selection)) = selected_tile_selection(state) else {
        state.status = "Select a tile region first.".to_string();
        return;
    };
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;

    apply_edit(state, move |document| {
        let tile_layer = selected_tile_layer_mut(document, layer_index)?;
        let snapshot = capture_region(tile_layer, min_x, min_y, width, height)?;

        for local_y in 0..height {
            for local_x in 0..width {
                let source_x = width - 1 - local_x;
                let gid = snapshot[(local_y * width + source_x) as usize];
                tile_layer.set_tile(min_x + local_x, min_y + local_y, gid)?;
            }
        }

        Ok(())
    });

    if state.status == "Edit applied." {
        state.status = "Flipped selection on the X axis.".to_string();
    }
}

pub(crate) fn flip_tile_selection_vertically(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        for local_y in 0..(transfer.height / 2) {
            for local_x in 0..transfer.width {
                let top = (local_y * transfer.width + local_x) as usize;
                let bottom =
                    ((transfer.height - 1 - local_y) * transfer.width + local_x) as usize;
                transfer.tiles.swap(top, bottom);
            }
        }
        sync_clipboard_from_transfer(state);
        state.status = "Flipped moving selection on the Y axis.".to_string();
        return;
    }

    let Some((layer_index, selection)) = selected_tile_selection(state) else {
        state.status = "Select a tile region first.".to_string();
        return;
    };
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;

    apply_edit(state, move |document| {
        let tile_layer = selected_tile_layer_mut(document, layer_index)?;
        let snapshot = capture_region(tile_layer, min_x, min_y, width, height)?;

        for local_y in 0..height {
            for local_x in 0..width {
                let source_y = height - 1 - local_y;
                let gid = snapshot[(source_y * width + local_x) as usize];
                tile_layer.set_tile(min_x + local_x, min_y + local_y, gid)?;
            }
        }

        Ok(())
    });

    if state.status == "Edit applied." {
        state.status = "Flipped selection on the Y axis.".to_string();
    }
}

pub(crate) fn rotate_tile_selection_clockwise(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        let old_width = transfer.width;
        let old_height = transfer.height;
        let mut rotated = vec![0; (old_width * old_height) as usize];
        for source_y in 0..old_height {
            for source_x in 0..old_width {
                let gid = transfer.tiles[(source_y * old_width + source_x) as usize];
                let dest_x = old_height - 1 - source_y;
                let dest_y = source_x;
                rotated[(dest_y * old_height + dest_x) as usize] = gid;
            }
        }
        transfer.width = old_height;
        transfer.height = old_width;
        transfer.tiles = rotated;
        sync_clipboard_from_transfer(state);
        resize_transfer_selection(state);
        state.status = "Rotated moving selection clockwise.".to_string();
        return;
    }

    let Some((layer_index, selection)) = selected_tile_selection(state) else {
        state.status = "Select a tile region first.".to_string();
        return;
    };
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    let new_width = height;
    let new_height = width;

    apply_edit(state, move |document| {
        let tile_layer = selected_tile_layer_mut(document, layer_index)?;
        if min_x + new_width > tile_layer.width || min_y + new_height > tile_layer.height {
            return Err(EditorError::Invalid(
                "rotated selection would extend beyond the layer bounds".to_string(),
            ));
        }

        let snapshot = capture_region(tile_layer, min_x, min_y, width, height)?;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                tile_layer.set_tile(x, y, 0)?;
            }
        }

        for source_y in 0..height {
            for source_x in 0..width {
                let gid = snapshot[(source_y * width + source_x) as usize];
                let dest_x = min_x + (height - 1 - source_y);
                let dest_y = min_y + source_x;
                tile_layer.set_tile(dest_x, dest_y, gid)?;
            }
        }

        Ok(())
    });

    if state.status == "Edit applied." {
        state.tile_selection = Some(TileSelectionRegion {
            start_cell: (min_x, min_y),
            end_cell: (min_x + new_width - 1, min_y + new_height - 1),
        });
        state.status = "Rotated selection clockwise.".to_string();
    }
}

pub(crate) fn delete_tile_selection(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_mut() {
        transfer.tiles.fill(0);
        sync_clipboard_from_transfer(state);
        state.status = "Cleared moving selection contents.".to_string();
        return;
    }

    let Some((layer_index, selection)) = selected_tile_selection(state) else {
        state.status = "Select a tile region first.".to_string();
        return;
    };
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);

    apply_edit(state, move |document| {
        let tile_layer = selected_tile_layer_mut(document, layer_index)?;
        clear_region_tiles(tile_layer, min_x, min_y, max_x, max_y)
    });

    if state.status == "Edit applied." {
        state.status = "Cleared selected region.".to_string();
    }
}

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
    let (min_x, min_y, _, _) = selection_bounds(selection);

    if transfer.source_layer != target_layer {
        cancel_tile_selection_transfer(state);
        state.status = "Selection move canceled because the active layer changed.".to_string();
        return;
    }

    let apply_result = if matches!(transfer.mode, TileSelectionTransferMode::Cut) {
        let Some(session) = state.session.as_mut() else {
            state.status = "No map loaded.".to_string();
            return;
        };
        let result = session.edit(|document| {
            let tile_layer = selected_tile_layer_mut(document, target_layer)?;
            write_region_tiles(tile_layer, min_x, min_y, transfer.width, transfer.height, &transfer.tiles)
        });
        if result.is_ok() {
            session.finish_history_batch();
        } else {
            session.abort_history_batch();
        }
        result
    } else {
        let tiles = transfer.tiles.clone();
        let width = transfer.width;
        let height = transfer.height;
        apply_edit_result(state, move |document| {
            let tile_layer = selected_tile_layer_mut(document, target_layer)?;
            write_region_tiles(tile_layer, min_x, min_y, width, height, &tiles)
        })
    };

    match apply_result {
        Ok(()) => {
            let end_cell = (
                min_x + transfer.width.saturating_sub(1),
                min_y + transfer.height.saturating_sub(1),
            );
            state.tile_selection = Some(TileSelectionRegion {
                start_cell: (min_x, min_y),
                end_cell,
            });
            state.tile_selection_preview = None;
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
        let (min_x, min_y, _, _) = selection_bounds(transfer.source_selection);
        let restore = {
            let Some(session) = state.session.as_mut() else {
                return;
            };
            session.edit(|document| {
                let tile_layer = selected_tile_layer_mut(document, transfer.source_layer)?;
                write_region_tiles(
                    tile_layer,
                    min_x,
                    min_y,
                    transfer.width,
                    transfer.height,
                    &transfer.tiles,
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

fn selected_tile_selection(state: &AppState) -> Option<(usize, TileSelectionRegion)> {
    let selection = state.tile_selection?;
    state
        .session
        .as_ref()
        .and_then(|session| session.document().map.layer(state.active_layer))
        .and_then(Layer::as_tile)
        .map(|_| (state.active_layer, selection))
}

fn selection_bounds(selection: TileSelectionRegion) -> (u32, u32, u32, u32) {
    (
        selection.start_cell.0.min(selection.end_cell.0),
        selection.start_cell.1.min(selection.end_cell.1),
        selection.start_cell.0.max(selection.end_cell.0),
        selection.start_cell.1.max(selection.end_cell.1),
    )
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

fn capture_region(
    tile_layer: &taled_core::TileLayer,
    min_x: u32,
    min_y: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u32>, EditorError> {
    let mut tiles = Vec::with_capacity((width * height) as usize);
    for local_y in 0..height {
        for local_x in 0..width {
            let x = min_x + local_x;
            let y = min_y + local_y;
            let gid = tile_layer.tile_at(x, y).ok_or_else(|| {
                EditorError::Invalid(format!("tile coordinate out of bounds: {x},{y}"))
            })?;
            tiles.push(gid);
        }
    }
    Ok(tiles)
}

fn sync_clipboard_from_transfer(state: &mut AppState) {
    if let Some(transfer) = state.tile_selection_transfer.as_ref() {
        state.tile_clipboard = Some(TileClipboard {
            width: transfer.width,
            height: transfer.height,
            tiles: transfer.tiles.clone(),
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
    state.tile_selection = Some(clamp_transfer_selection_to_map(
        state,
        min_x,
        min_y,
        transfer.width,
        transfer.height,
    ));
}

fn clamp_transfer_selection_to_map(
    state: &AppState,
    origin_x: u32,
    origin_y: u32,
    width: u32,
    height: u32,
) -> TileSelectionRegion {
    let (origin_x, origin_y) = if let Some(session) = state.session.as_ref() {
        let map = &session.document().map;
        (
            origin_x.min(map.width.saturating_sub(width)),
            origin_y.min(map.height.saturating_sub(height)),
        )
    } else {
        (origin_x, origin_y)
    };

    TileSelectionRegion {
        start_cell: (origin_x, origin_y),
        end_cell: (
            origin_x + width.saturating_sub(1),
            origin_y + height.saturating_sub(1),
        ),
    }
}

fn write_region_tiles(
    tile_layer: &mut taled_core::TileLayer,
    min_x: u32,
    min_y: u32,
    width: u32,
    height: u32,
    tiles: &[u32],
) -> Result<(), EditorError> {
    for local_y in 0..height {
        for local_x in 0..width {
            let gid = tiles[(local_y * width + local_x) as usize];
            tile_layer.set_tile(min_x + local_x, min_y + local_y, gid)?;
        }
    }
    Ok(())
}

fn clear_region_tiles(
    tile_layer: &mut taled_core::TileLayer,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
) -> Result<(), EditorError> {
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            tile_layer.set_tile(x, y, 0)?;
        }
    }
    Ok(())
}

fn capture_tile_selection_transfer(
    state: &mut AppState,
) -> Option<(TileSelectionTransfer, TileClipboard)> {
    let Some((layer_index, selection)) = selected_tile_selection(state) else {
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

    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    let tiles = capture_region(tile_layer, min_x, min_y, width, height)
        .expect("selection bounds should stay inside tile layer");
    let clipboard = TileClipboard {
        width,
        height,
        tiles: tiles.clone(),
    };
    let transfer = TileSelectionTransfer {
        source_layer: layer_index,
        source_selection: selection,
        width,
        height,
        tiles,
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
    use std::path::PathBuf;

    use taled_core::EditorSession;

    use super::{
        apply_cell_tool, apply_shape_fill_rect, copy_tile_selection, cut_tile_selection,
        delete_tile_selection, flip_tile_selection_horizontally, flip_tile_selection_vertically,
        place_tile_selection_transfer, rotate_tile_selection_clockwise, select_tile_region,
    };
    use crate::app_state::{AppState, TileSelectionRegion, TileSelectionTransferMode, Tool};

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
}
