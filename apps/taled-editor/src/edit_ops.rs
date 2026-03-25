use std::collections::VecDeque;

use taled_core::{
    EditorError, EditorSession, Layer, MapObject, ObjectShape, Property, PropertyValue,
};

use crate::app_state::{AppState, Tool};

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
        let layer = document.map.layer_mut(layer_index).ok_or_else(|| {
            EditorError::Invalid(format!("unknown layer index {layer_index}"))
        })?;
        if layer.locked() {
            return Err(EditorError::Invalid("layer is locked".to_string()));
        }
        let tile_layer = layer.as_tile_mut().ok_or_else(|| {
            EditorError::Invalid("active layer is not a tile layer".to_string())
        })?;
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
        let layer = document.map.layer_mut(layer_index).ok_or_else(|| {
            EditorError::Invalid(format!("unknown layer index {layer_index}"))
        })?;
        if layer.locked() {
            return Err(EditorError::Invalid("layer is locked".to_string()));
        }
        let tile_layer = layer.as_tile_mut().ok_or_else(|| {
            EditorError::Invalid("active layer is not a tile layer".to_string())
        })?;

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
    let Some(session) = state.session.as_mut() else {
        state.status = "No map loaded.".to_string();
        return;
    };

    match session.edit(edit) {
        Ok(()) => state.status = "Edit applied.".to_string(),
        Err(error) => state.status = format!("Edit failed: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use taled_core::EditorSession;

    use super::{apply_cell_tool, apply_shape_fill_rect, select_tile_region};
    use crate::app_state::{AppState, Tool};

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
}
