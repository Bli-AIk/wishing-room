use crate::app_state::AppState;

/// Apply snap settings (grid / integer) to a world coordinate pair.
pub(crate) fn snap_position(state: &AppState, x: f32, y: f32) -> (f32, f32) {
    let (mut sx, mut sy) = (x, y);
    if state.snap_to_grid {
        if let Some(session) = state.session.as_ref() {
            let tw = session.document().map.tile_width as f32;
            let th = session.document().map.tile_height as f32;
            if tw > 0.0 {
                sx = (sx / tw).round() * tw;
            }
            if th > 0.0 {
                sy = (sy / th).round() * th;
            }
        }
    } else if state.snap_to_int {
        sx = sx.round();
        sy = sy.round();
    }
    (sx, sy)
}

/// Insert the currently selected tile as a tile-object at the given world position.
pub(crate) fn insert_tile_object(state: &mut AppState, wx: f32, wy: f32) {
    let (wx, wy) = snap_position(state, wx, wy);
    let gid = state.selected_gid;
    if gid == 0 {
        state.status = "No tile selected.".to_string();
        return;
    }
    let layer_idx = state.active_layer;

    // Determine tile dimensions from the tileset.
    let (obj_w, obj_h) = {
        let Some(session) = state.session.as_ref() else {
            return;
        };
        let map = &session.document().map;
        let Some(tile_ref) = map.tile_reference_for_gid(gid) else {
            state.status = "Invalid tile GID.".to_string();
            return;
        };
        let ts = &tile_ref.tileset.tileset;
        if let Some(img) = ts.tile_images.get(&tile_ref.local_id) {
            (img.width as f32, img.height as f32)
        } else {
            (ts.tile_width as f32, ts.tile_height as f32)
        }
    };

    let Some(session) = state.session.as_mut() else {
        return;
    };
    let result = session.edit(move |doc| {
        let obj_id = doc.map.next_object_id;
        doc.map.next_object_id += 1;
        let layer = doc
            .map
            .layer_mut(layer_idx)
            .ok_or_else(|| taled_core::EditorError::Invalid("no layer".into()))?;
        let ol = layer
            .as_object_mut()
            .ok_or_else(|| taled_core::EditorError::Invalid("not object layer".into()))?;
        ol.objects.push(taled_core::MapObject {
            id: obj_id,
            name: String::new(),
            visible: true,
            x: wx,
            y: wy,
            width: obj_w,
            height: obj_h,
            shape: taled_core::ObjectShape::Rectangle,
            gid: Some(gid),
            properties: Vec::new(),
        });
        Ok(())
    });
    match result {
        Ok(()) => {
            state
                .undo_action_order
                .push(crate::app_state::UndoActionKind::DocumentEdit);
            state.redo_action_order.clear();
            state.canvas_dirty = true;
            state.tiles_dirty = true;
        }
        Err(e) => state.status = format!("Insert failed: {e}"),
    }
}
