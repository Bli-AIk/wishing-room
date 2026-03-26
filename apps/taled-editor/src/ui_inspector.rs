use dioxus::prelude::*;
use taled_core::{EditorDocument, ObjectShape, Property};

use crate::{
    app_state::{AppState, PaletteTile},
    edit_ops::{
        add_layer_property, add_object_property, create_object, delete_selected_object,
        nudge_selected_object, remove_layer_property, remove_object_property, rename_layer,
        rename_layer_property, rename_object_property, rename_selected_object,
        selected_object_view, update_layer_property_value, update_object_property_value,
        update_selected_object_geometry,
    },
    ui_visuals::{object_icon_style, palette_tile_style},
};

pub(crate) fn render_palette(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return rsx! { div { class: "tileset-list" } };
    };

    let palette = collect_palette(session.document());
    rsx! {
        div {
            h2 { "Tileset" }
            div { class: "tileset-list",
                for (tileset_index, tileset) in session.document().map.tilesets.iter().enumerate() {
                    div {
                        key: "tileset-{tileset_index}",
                        class: "tileset-card",
                        h3 { "{tileset.tileset.name}" }
                        div { class: "palette-grid",
                            for tile in palette.clone().into_iter().filter(|tile| tile.tileset_index == tileset_index) {
                                button {
                                    key: "palette-{tile.gid}",
                                    class: if snapshot.selected_gid == tile.gid { "palette-tile active" } else { "palette-tile" },
                                    style: palette_tile_style(session.document(), &snapshot.image_cache, &tile),
                                    onclick: move |_| {
                                        let mut state = state.write();
                                        state.selected_gid = tile.gid;
                                        state.status = format!("Selected gid {}.", tile.gid);
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn render_inspector(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return rsx! { div { class: "field-stack", h2 { "Inspector" } } };
    };

    let active_layer_index = snapshot.active_layer;
    let selected_object_id = snapshot.selected_object;
    let layer = session.document().map.layer(active_layer_index);
    let dirty = if session.dirty() { "Yes" } else { "No" };

    rsx! {
        div { class: "field-stack",
            h2 { "Inspector" }
            if let Some(layer) = layer {
                div { class: "inline-row", "Dirty: {dirty}" }
                label {
                    "Layer Name"
                    input {
                        value: layer.name().to_string(),
                        onchange: move |event| rename_layer(&mut state.write(), active_layer_index, event.value()),
                    }
                }
                if let Some(tile_layer) = layer.as_tile() {
                    div { class: "inline-row", "Tile Layer · {tile_layer.width} x {tile_layer.height}" }
                }
                if let Some(object_layer) = layer.as_object() {
                    div { class: "inline-row",
                        button { onclick: move |_| create_object(&mut state.write(), ObjectShape::Rectangle), "Add Rectangle" }
                        button { onclick: move |_| create_object(&mut state.write(), ObjectShape::Point), "Add Point" }
                    }
                    div { class: "object-list",
                        for object in &object_layer.objects {
                            div {
                                key: "list-object-{object.id}",
                                class: "object-row",
                                button {
                                    class: if selected_object_id == Some(object.id) { "active" } else { "" },
                                    onclick: {
                                        let object_id = object.id;
                                        move |_| {
                                            let mut state = state.write();
                                            state.selected_object = Some(object_id);
                                            state.tile_selection = None;
                                            state.tile_selection_cells = None;
                                            state.tile_selection_preview = None;
                                            state.tile_selection_closing = None;
                                            state.tile_selection_closing_cells = None;
                                            state.tile_selection_closing_started_at = None;
                                            state.tile_selection_last_tap_at = None;
                                        }
                                    },
                                    span {
                                        class: "object-shape-icon",
                                        style: object_icon_style(&object.shape),
                                    }
                                    "#{object.id} {object.name}"
                                }
                            }
                        }
                    }
                }

                h3 { "Layer Properties" }
                {render_layer_properties(layer.properties(), state)}
            }

            if let Some((object, _layer_index)) = selected_object_view(session, selected_object_id, active_layer_index) {
                h3 { "Object" }
                label {
                    "Name"
                    input {
                        value: object.name.clone(),
                        onchange: move |event| rename_selected_object(&mut state.write(), event.value()),
                    }
                }
                div { class: "inline-row",
                    button { onclick: move |_| nudge_selected_object(&mut state.write(), -16.0, 0.0), "Left" }
                    button { onclick: move |_| nudge_selected_object(&mut state.write(), 16.0, 0.0), "Right" }
                    button { onclick: move |_| nudge_selected_object(&mut state.write(), 0.0, -16.0), "Up" }
                    button { onclick: move |_| nudge_selected_object(&mut state.write(), 0.0, 16.0), "Down" }
                    button { onclick: move |_| delete_selected_object(&mut state.write()), "Delete" }
                }
                label {
                    "X"
                    input {
                        value: object.x.to_string(),
                        onchange: move |event| update_selected_object_geometry(&mut state.write(), "x", event.value()),
                    }
                }
                label {
                    "Y"
                    input {
                        value: object.y.to_string(),
                        onchange: move |event| update_selected_object_geometry(&mut state.write(), "y", event.value()),
                    }
                }
                if matches!(object.shape, ObjectShape::Rectangle) {
                    label {
                        "Width"
                        input {
                            value: object.width.to_string(),
                            onchange: move |event| update_selected_object_geometry(&mut state.write(), "width", event.value()),
                        }
                    }
                    label {
                        "Height"
                        input {
                            value: object.height.to_string(),
                            onchange: move |event| update_selected_object_geometry(&mut state.write(), "height", event.value()),
                        }
                    }
                }
                h3 { "Object Properties" }
                {render_object_properties(&object.properties, state)}
            }
        }
    }
}

fn render_layer_properties(properties: &[Property], mut state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "property-list",
            for (index, property) in properties.iter().enumerate() {
                div {
                    key: "property-{index}",
                    class: "property-row",
                    label {
                        "Name"
                        input {
                            value: property.name.clone(),
                            onchange: move |event| rename_layer_property(&mut state.write(), index, event.value()),
                        }
                    }
                    label {
                        "Value ({property.value.type_name()})"
                        input {
                            value: property.value.as_editor_string(),
                            onchange: move |event| update_layer_property_value(&mut state.write(), index, event.value()),
                        }
                    }
                    button { onclick: move |_| remove_layer_property(&mut state.write(), index), "Remove" }
                }
            }
            button { onclick: move |_| add_layer_property(&mut state.write()), "Add String Property" }
        }
    }
}

fn render_object_properties(properties: &[Property], mut state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "property-list",
            for (index, property) in properties.iter().enumerate() {
                div {
                    key: "object-property-{index}",
                    class: "property-row",
                    label {
                        "Name"
                        input {
                            value: property.name.clone(),
                            onchange: move |event| rename_object_property(&mut state.write(), index, event.value()),
                        }
                    }
                    label {
                        "Value ({property.value.type_name()})"
                        input {
                            value: property.value.as_editor_string(),
                            onchange: move |event| update_object_property_value(&mut state.write(), index, event.value()),
                        }
                    }
                    button { onclick: move |_| remove_object_property(&mut state.write(), index), "Remove" }
                }
            }
            button { onclick: move |_| add_object_property(&mut state.write()), "Add String Property" }
        }
    }
}

pub(crate) fn collect_palette(document: &EditorDocument) -> Vec<PaletteTile> {
    let mut palette = Vec::new();
    for (tileset_index, tileset) in document.map.tilesets.iter().enumerate() {
        for local_id in 0..tileset.tileset.tile_count {
            palette.push(PaletteTile {
                gid: tileset.first_gid + local_id,
                tileset_index,
                local_id,
            });
        }
    }
    palette
}
