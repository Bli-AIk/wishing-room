use std::path::Path;

use dioxus::prelude::*;
use wishing_core::{EditorSession, Layer, ObjectShape};

use crate::{
    app_state::{AppState, MobileScreen, PaletteTile, Tool},
    embedded_samples::{embedded_sample, embedded_sample_thumb, embedded_samples},
    edit_ops::{
        create_object, delete_selected_object, nudge_selected_object, rename_selected_object,
        selected_object_view, toggle_layer_lock, toggle_layer_visibility,
    },
    session_ops::{adjust_zoom, load_embedded_sample, load_sample, save_document},
    ui_canvas::render_canvas,
    ui_inspector::collect_palette,
    ui_visuals::{object_icon_style, palette_tile_style},
};

#[cfg(target_os = "android")]
use crate::platform::log_path;

#[derive(Clone)]
struct MobileObjectSummary {
    layer_index: usize,
    object_id: u32,
    name: String,
    shape: ObjectShape,
}

pub(crate) fn render_mobile_shell(snapshot: &AppState, state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "mobile-shell review-shell",
            match snapshot.mobile_screen {
                MobileScreen::Dashboard => rsx! { {render_dashboard(snapshot, state)} },
                MobileScreen::Editor => rsx! { {render_editor(snapshot, state)} },
                MobileScreen::Tilesets => rsx! { {render_tilesets(snapshot, state)} },
                MobileScreen::Layers => rsx! { {render_layers(snapshot, state)} },
                MobileScreen::Objects => rsx! { {render_objects(snapshot, state)} },
                MobileScreen::Settings => rsx! { {render_settings(snapshot, state)} },
            }
        }
    }
}

fn render_dashboard(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "review-page",
            {review_top_bar("Project Dashboard".to_string(), None, None, state)}
            div { class: "review-body",
                button {
                    class: "review-create-project",
                    onclick: move |_| {
                        state.write().status =
                            "Create New Project is not implemented yet. Dashboard placeholder only.".to_string();
                    },
                    {review_plus_icon("review-plus-icon")}
                    span { "Create New Project" }
                }
                div { class: "review-project-list-panel",
                    for sample in embedded_samples().iter() {
                        button {
                            key: "{sample.path}",
                            class: "review-project-row",
                            onclick: {
                                let sample_path = sample.path;
                                move |_| {
                                    let mut state = state.write();
                                    load_embedded_sample(&mut state, sample_path);
                                    state.mobile_screen = MobileScreen::Editor;
                                }
                            },
                            img {
                                class: "review-project-thumb",
                                src: embedded_sample_thumb(sample.path),
                                alt: "{sample.title} thumbnail",
                            }
                            div { class: "review-project-copy",
                                div { class: "review-project-title", "{sample.title}" }
                                div { class: "review-project-meta", "{sample.subtitle}" }
                                div { class: "review-project-meta", "{sample.meta}" }
                            }
                        }
                    }
                }
            }
            {review_nav(snapshot, state, true)}
        }
    }
}

fn render_editor(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return render_missing_screen(
            "Tile Map Editor".to_string(),
            "Pick an embedded TMX sample from Projects before editing.",
            state,
        );
    };

    let layers: Vec<(usize, String, &'static str, bool)> = session
        .document()
        .map
        .layers
        .iter()
        .enumerate()
        .take(3)
        .map(|(index, layer)| {
            (
                index,
                layer.name().to_string(),
                layer_kind_label(layer),
                layer.visible(),
            )
        })
        .collect();
    let palette: Vec<PaletteTile> = collect_palette(session.document()).into_iter().take(24).collect();

    rsx! {
        div { class: "review-page review-editor-page",
            {review_top_bar(
                document_title(snapshot),
                Some(("Projects", MobileScreen::Dashboard)),
                Some(("Layers", MobileScreen::Layers)),
                state,
            )}
            div { class: "review-editor-canvas",
                div { class: "review-map-surface review-map-live",
                    {render_canvas(snapshot, state)}
                }
                div { class: "review-dpad",
                    button { class: "up", onclick: move |_| state.write().pan_y -= 32, "^" }
                    button { class: "left", onclick: move |_| state.write().pan_x -= 32, "<" }
                    button {
                        class: "center",
                        onclick: move |_| state.write().mobile_screen = MobileScreen::Tilesets,
                        "{snapshot.zoom_percent}%"
                    }
                    button { class: "right", onclick: move |_| state.write().pan_x += 32, ">" }
                    button { class: "down", onclick: move |_| state.write().pan_y += 32, "v" }
                }
                div { class: "review-layer-float",
                    div { class: "review-layer-float-title", "Layers" }
                    for (index, name, kind, visible) in layers {
                        div {
                            key: "review-float-layer-{index}",
                            class: if snapshot.active_layer == index {
                                "review-layer-float-item active"
                            } else {
                                "review-layer-float-item"
                            },
                            button {
                                onclick: move |_| {
                                    let mut state = state.write();
                                    state.active_layer = index;
                                    state.selected_object = None;
                                },
                                span { "{name}" }
                                span { class: "muted", "{kind}" }
                            }
                            span { class: if visible { "review-eye on" } else { "review-eye off" }, "o" }
                            span { class: "review-menu-glyph", "≡" }
                        }
                    }
                }
            }
            div { class: "review-editor-toolbar",
                div { class: "review-tool-row review-tool-row-live",
                    {review_tool_button(snapshot, state, Tool::Select, "Select")}
                    {review_tool_button(snapshot, state, Tool::Paint, "Brush")}
                    {review_tool_button(snapshot, state, Tool::Erase, "Eraser")}
                    {review_tool_button(snapshot, state, Tool::AddRectangle, "Rect")}
                    {review_tool_button(snapshot, state, Tool::AddPoint, "Point")}
                }
                div { class: "review-tile-strip review-tile-strip-live",
                    for tile in palette {
                        button {
                            key: "review-tile-{tile.gid}",
                            class: if snapshot.selected_gid == tile.gid {
                                "review-tile-chip selected live"
                            } else {
                                "review-tile-chip live"
                            },
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
            {review_nav(snapshot, state, false)}
        }
    }
}

fn render_tilesets(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return render_missing_screen(
            "Tileset Property Editor".to_string(),
            "Load an embedded TMX sample before opening tilesets.",
            state,
        );
    };

    let palette = collect_palette(session.document());
    let selected_gid = snapshot.selected_gid;
    let selected_summary = session
        .document()
        .map
        .tile_reference_for_gid(selected_gid)
        .map(|reference| {
            format!(
                "Tile ID {} from {}",
                selected_gid, reference.tileset.tileset.name
            )
        })
        .unwrap_or_else(|| "Choose a tile from the sheet below.".to_string());

    rsx! {
        div { class: "review-page",
            {review_top_bar(
                "Tileset Property Editor".to_string(),
                Some(("Back", MobileScreen::Editor)),
                Some(("Done", MobileScreen::Editor)),
                state,
            )}
            div { class: "review-body review-section-stack",
                div { class: "review-section-title", "Sprite Sheet View" }
                div { class: "review-tileset-sheet review-tileset-sheet-live",
                    for tile in palette {
                        button {
                            key: "tile-{tile.gid}",
                            class: if selected_gid == tile.gid {
                                "review-sheet-cell active live"
                            } else {
                                "review-sheet-cell live"
                            },
                            style: palette_tile_style(session.document(), &snapshot.image_cache, &tile),
                            onclick: move |_| {
                                let mut state = state.write();
                                state.selected_gid = tile.gid;
                                state.status = format!("Selected gid {}.", tile.gid);
                            },
                        }
                    }
                }
                div { class: "review-info-card review-selected-tile-card",
                    div {
                        class: "review-selected-tile-art",
                        style: selected_tile_style(snapshot, session, selected_gid),
                    }
                    div { class: "review-project-copy",
                        div { class: "review-info-title", "{selected_summary}" }
                        div { class: "review-info-meta", "Tile property editors and collision authoring stay on this page once implemented." }
                    }
                }
                div { class: "review-section-title with-gap", "Custom Properties" }
                div { class: "review-settings-card",
                    div { class: "review-setting-row",
                        span { "Properties" }
                        span { class: "muted", "Placeholder" }
                    }
                    div { class: "review-setting-row",
                        span { "Collision Editor" }
                        span { class: "muted", "Placeholder" }
                    }
                }
            }
            {review_nav(snapshot, state, false)}
        }
    }
}

fn render_layers(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return render_missing_screen(
            "Layer Manager".to_string(),
            "Load an embedded TMX sample before managing layers.",
            state,
        );
    };

    rsx! {
        div { class: "review-page",
            {review_top_bar(
                "Layer Manager".to_string(),
                Some(("Back", MobileScreen::Editor)),
                Some(("Done", MobileScreen::Editor)),
                state,
            )}
            div { class: "review-body review-list",
                for (index, layer) in session.document().map.layers.iter().enumerate() {
                    div {
                        key: "{index}",
                        class: if snapshot.active_layer == index {
                            "review-layer-row active"
                        } else {
                            "review-layer-row"
                        },
                        span { class: "review-menu-glyph", "≡" }
                        div { class: "review-layer-thumb {layer_thumb_variant(index, layer)}" }
                        button {
                            class: "review-layer-name-button",
                            onclick: move |_| {
                                let mut state = state.write();
                                state.active_layer = index;
                                state.selected_object = None;
                            },
                            span { class: "review-layer-title-stack",
                                span { class: "review-layer-name", "{layer.name()}" }
                                span { class: "muted", "{layer_kind_label(layer)}" }
                            }
                        }
                        button {
                            class: if layer.visible() {
                                "review-eye on review-layer-toggle"
                            } else {
                                "review-eye off review-layer-toggle"
                            },
                            onclick: move |_| toggle_layer_visibility(&mut state.write(), index),
                            "o"
                        }
                        button {
                            class: if layer.locked() {
                                "review-lock on review-layer-toggle"
                            } else {
                                "review-lock off review-layer-toggle"
                            },
                            onclick: move |_| toggle_layer_lock(&mut state.write(), index),
                            "u"
                        }
                        div { class: "review-opacity",
                            span { "{layer_kind_label(layer)}" }
                            span { "100%" }
                            div { class: "review-slider-track",
                                div { class: "review-slider-fill", style: "width:100%;" }
                                div { class: "review-slider-knob", style: "left:calc(100% - 10px);" }
                            }
                        }
                    }
                }
                button { class: "review-secondary-button", "Add Layer" }
            }
            {review_nav(snapshot, state, false)}
        }
    }
}

fn render_objects(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return render_missing_screen(
            "Object Library".to_string(),
            "Load an embedded TMX sample before browsing objects.",
            state,
        );
    };

    let objects = collect_objects(session);

    rsx! {
        div { class: "review-page",
            {review_top_bar(
                "Object Library".to_string(),
                Some(("Back", MobileScreen::Editor)),
                Some(("Done", MobileScreen::Editor)),
                state,
            )}
            div { class: "review-body review-section-stack",
                div { class: "review-search-bar",
                    span { class: "review-search-icon", "o" }
                    span { class: "muted", "Search objects..." }
                }
                div { class: "review-object-grid" ,
                    for entry in objects.iter() {
                        button {
                            key: "{entry.object_id}",
                            class: if snapshot.selected_object == Some(entry.object_id) {
                                "review-object-card active"
                            } else {
                                "review-object-card"
                            },
                            onclick: {
                                let entry = entry.clone();
                                move |_| {
                                    let mut state = state.write();
                                    state.active_layer = entry.layer_index;
                                    state.selected_object = Some(entry.object_id);
                                }
                            },
                            span {
                                class: "review-object-art live",
                                style: object_icon_style(&entry.shape),
                            }
                            div { class: "review-object-card-label", "{entry.name}" }
                        }
                    }
                }
                div { class: "review-actions-grid" ,
                    button {
                        class: "review-secondary-button compact",
                        onclick: move |_| create_object_on_first_object_layer(&mut state.write(), ObjectShape::Rectangle),
                        "Add Rect"
                    }
                    button {
                        class: "review-secondary-button compact",
                        onclick: move |_| create_object_on_first_object_layer(&mut state.write(), ObjectShape::Point),
                        "Add Point"
                    }
                    button {
                        class: "review-secondary-button compact",
                        onclick: move |_| delete_selected_object(&mut state.write()),
                        "Delete"
                    }
                    button {
                        class: "review-secondary-button compact",
                        onclick: move |_| state.write().mobile_screen = MobileScreen::Editor,
                        "Canvas"
                    }
                }
                if let Some((object, _layer_index)) =
                    selected_object_view(session, snapshot.selected_object, snapshot.active_layer)
                {
                    div { class: "review-info-card",
                        div { class: "review-project-copy",
                            div { class: "review-info-title", "Selected Object: {object.name} (ID {object.id})" }
                            div { class: "review-info-meta", "Global Coordinates: (X: {object.x:.0}, Y: {object.y:.0})" }
                        }
                    }
                    div { class: "review-actions-grid",
                        button { class: "review-secondary-button compact", onclick: move |_| nudge_selected_object(&mut state.write(), -16.0, 0.0), "Left" }
                        button { class: "review-secondary-button compact", onclick: move |_| nudge_selected_object(&mut state.write(), 16.0, 0.0), "Right" }
                        button { class: "review-secondary-button compact", onclick: move |_| nudge_selected_object(&mut state.write(), 0.0, -16.0), "Up" }
                        button { class: "review-secondary-button compact", onclick: move |_| nudge_selected_object(&mut state.write(), 0.0, 16.0), "Down" }
                    }
                    label { class: "review-field",
                        span { "Name" }
                        input {
                            value: object.name.clone(),
                            onchange: move |event| rename_selected_object(&mut state.write(), event.value()),
                        }
                    }
                }
                div { class: "review-info-card",
                    div { class: "review-info-title", "Attached Scripts & Events" }
                    div { class: "review-script-row", "UI placeholder for script bindings, triggers, and event metadata." }
                }
            }
            {review_nav(snapshot, state, false)}
        }
    }
}

fn render_settings(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "review-page",
            {review_top_bar(
                "App Settings".to_string(),
                Some(("Back", MobileScreen::Editor)),
                Some(("Done", MobileScreen::Editor)),
                state,
            )}
            div { class: "review-body review-section-stack",
                div { class: "review-caption", "Session Actions" }
                div { class: "review-settings-card" ,
                    div { class: "review-setting-row",
                        span { "Embedded Sample" }
                        button {
                            class: "review-link-button",
                            onclick: move |_| {
                                let mut state = state.write();
                                load_sample(&mut state);
                                state.mobile_screen = MobileScreen::Editor;
                            },
                            "Reload Default"
                        }
                    }
                    div { class: "review-setting-row",
                        span { "Save" }
                        button {
                            class: "review-link-button",
                            onclick: move |_| save_document(&mut state.write()),
                            "Run"
                        }
                    }
                    div { class: "review-setting-row",
                        span { "Undo" }
                        button {
                            class: "review-link-button",
                            onclick: move |_| {
                                let mut state = state.write();
                                if state.session.as_mut().is_some_and(EditorSession::undo) {
                                    state.status = "Undo applied.".to_string();
                                } else {
                                    state.status = "Nothing to undo.".to_string();
                                }
                            },
                            "Run"
                        }
                    }
                    div { class: "review-setting-row",
                        span { "Redo" }
                        button {
                            class: "review-link-button",
                            onclick: move |_| {
                                let mut state = state.write();
                                if state.session.as_mut().is_some_and(EditorSession::redo) {
                                    state.status = "Redo applied.".to_string();
                                } else {
                                    state.status = "Nothing to redo.".to_string();
                                }
                            },
                            "Run"
                        }
                    }
                }
                div { class: "review-caption", "View" }
                div { class: "review-settings-card" ,
                    {review_slider_row("Zoom", &format!("{}%", snapshot.zoom_percent))}
                    div { class: "review-setting-row",
                        span { "Zoom -" }
                        button {
                            class: "review-link-button",
                            onclick: move |_| adjust_zoom(&mut state.write(), -25),
                            "Apply"
                        }
                    }
                    div { class: "review-setting-row",
                        span { "Zoom +" }
                        button {
                            class: "review-link-button",
                            onclick: move |_| adjust_zoom(&mut state.write(), 25),
                            "Apply"
                        }
                    }
                    div { class: "review-setting-row",
                        span { "Theme" }
                        span { class: "muted", "Dark" }
                    }
                }
                div { class: "review-caption", "Diagnostics" }
                div { class: "review-info-card review-note-card",
                    div { class: "review-info-title", "Status" }
                    div { class: "review-info-meta", "{snapshot.status}" }
                }
                {render_log_path_card()}
                div { class: "review-caption", "Export Settings" }
                div { class: "review-settings-card",
                    div { class: "review-setting-row", span { "JSON" } div { class: "review-toggle on", div { class: "knob" } } }
                    div { class: "review-setting-row", span { "XML" } div { class: "review-toggle on", div { class: "knob" } } }
                    div { class: "review-setting-row", span { "PNG" } div { class: "review-toggle on", div { class: "knob" } } }
                }
            }
            {review_nav(snapshot, state, false)}
        }
    }
}

fn render_missing_screen(title: String, message: &'static str, mut state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "review-page",
            {review_top_bar(title, Some(("Back", MobileScreen::Dashboard)), None, state)}
            div { class: "review-body review-section-stack",
                div { class: "review-info-card review-note-card",
                    div { class: "review-info-title", "No map loaded" }
                    div { class: "review-info-meta", "{message}" }
                }
                button {
                    class: "review-secondary-button",
                    onclick: move |_| state.write().mobile_screen = MobileScreen::Dashboard,
                    "Open Projects"
                }
            }
            {review_nav(&state.read().clone(), state, false)}
        }
    }
}

fn review_top_bar(
    title: String,
    left: Option<(&'static str, MobileScreen)>,
    right: Option<(&'static str, MobileScreen)>,
    mut state: Signal<AppState>,
) -> Element {
    rsx! {
        div { class: "review-header",
            if let Some((label, screen)) = left {
                button {
                    class: "review-header-action left",
                    onclick: move |_| state.write().mobile_screen = screen.clone(),
                    "{label}"
                }
            } else {
                div { class: "review-header-spacer" }
            }
            h1 { "{title}" }
            if let Some((label, screen)) = right {
                button {
                    class: "review-header-action right",
                    onclick: move |_| state.write().mobile_screen = screen.clone(),
                    "{label}"
                }
            } else {
                div { class: "review-header-spacer" }
            }
        }
    }
}

fn review_nav(snapshot: &AppState, state: Signal<AppState>, dashboard_variant: bool) -> Element {
    rsx! {
        div { class: if dashboard_variant { "review-bottom-nav dashboard" } else { "review-bottom-nav editor" },
            if dashboard_variant {
                {review_nav_button(snapshot, state, MobileScreen::Dashboard, "Projects")}
                {review_static_nav_item("Assets")}
                {review_nav_button(snapshot, state, MobileScreen::Settings, "Settings")}
            } else {
                {review_nav_button(snapshot, state, MobileScreen::Tilesets, "Tilesets")}
                {review_nav_button(snapshot, state, MobileScreen::Layers, "Layers")}
                {review_nav_button(snapshot, state, MobileScreen::Objects, "Objects")}
                {review_nav_button(snapshot, state, MobileScreen::Settings, "Settings")}
            }
        }
    }
}

fn review_nav_button(
    snapshot: &AppState,
    mut state: Signal<AppState>,
    screen: MobileScreen,
    label: &'static str,
) -> Element {
    rsx! {
        button {
            class: if snapshot.mobile_screen == screen { "review-nav-item active" } else { "review-nav-item" },
            onclick: move |_| state.write().mobile_screen = screen.clone(),
            div { class: "review-nav-icon", {review_nav_icon(label)} }
            span { "{label}" }
        }
    }
}

fn review_static_nav_item(label: &'static str) -> Element {
    rsx! {
        div { class: "review-nav-item review-nav-static",
            div { class: "review-nav-icon", {review_nav_icon(label)} }
            span { "{label}" }
        }
    }
}

fn review_nav_icon(label: &'static str) -> Element {
    match label {
        "Projects" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M8 6h13" }
                path { d: "M8 12h13" }
                path { d: "M8 18h13" }
                path { d: "M3 6h.01" }
                path { d: "M3 12h.01" }
                path { d: "M3 18h.01" }
            }
        },
        "Assets" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" }
            }
        },
        "Tilesets" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
                rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
                rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
                rect { x: "14", y: "14", width: "7", height: "7", rx: "1" }
            }
        },
        "Layers" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M12 3 3 8l9 5 9-5-9-5z" }
                path { d: "m3 12 9 5 9-5" }
                path { d: "m3 16 9 5 9-5" }
            }
        },
        "Objects" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M12 2v20" }
                path { d: "M2 12h20" }
                circle { cx: "12", cy: "12", r: "5" }
            }
        },
        "Settings" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                circle { cx: "12", cy: "12", r: "3" }
                path { d: "M19.4 15a1.7 1.7 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06A1.7 1.7 0 0 0 15 19.4a1.7 1.7 0 0 0-1 .6 1.7 1.7 0 0 0-.4 1V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-.4-1 1.7 1.7 0 0 0-1-.6 1.7 1.7 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-.6-1 1.7 1.7 0 0 0-1-.4H3a2 2 0 1 1 0-4h.09a1.7 1.7 0 0 0 1-.4 1.7 1.7 0 0 0 .6-1 1.7 1.7 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.7 1.7 0 0 0 9 4.6c.38-.08.72-.28 1-.6a1.7 1.7 0 0 0 .4-1V3a2 2 0 1 1 4 0v.09c0 .38.14.74.4 1 .28.32.62.52 1 .6a1.7 1.7 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.7 1.7 0 0 0 19.4 9c.08.38.28.72.6 1 .26.26.62.4 1 .4H21a2 2 0 1 1 0 4h-.09c-.38 0-.74.14-1 .4-.32.28-.52.62-.6 1z" }
            }
        },
        _ => rsx! { span {} },
    }
}

fn review_plus_icon(class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12 5v14" }
            path { d: "M5 12h14" }
        }
    }
}

fn review_tool_button(
    snapshot: &AppState,
    mut state: Signal<AppState>,
    tool: Tool,
    label: &'static str,
) -> Element {
    rsx! {
        button {
            class: if snapshot.tool == tool { "review-tool active" } else { "review-tool" },
            onclick: move |_| state.write().tool = tool.clone(),
            div { class: "review-tool-icon", {review_tool_icon(&tool)} }
            span { "{label}" }
        }
    }
}

fn review_tool_icon(tool: &Tool) -> Element {
    match tool {
        Tool::Select => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M5 4v14l4-4h6" }
                path { d: "M13.5 13.5 18 20" }
            }
        },
        Tool::Paint => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M14 4c3 2 5 4.5 5 7a4 4 0 0 1-4 4h-1l-4.5 4.5a1.8 1.8 0 0 1-2.5 0l-2-2a1.8 1.8 0 0 1 0-2.5L9.5 10V9a4 4 0 0 1 4-5z" }
            }
        },
        Tool::Erase => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "m7 14 6-8 7 7-8 6H7z" }
                path { d: "M4 20h9" }
            }
        },
        Tool::AddRectangle => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "5", y: "7", width: "14", height: "10", rx: "2" }
            }
        },
        Tool::AddPoint => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M12 5v14" }
                path { d: "M5 12h14" }
                circle { cx: "12", cy: "12", r: "2.5" }
            }
        },
    }
}

fn review_slider_row(label: &'static str, value: &str) -> Element {
    rsx! {
        div { class: "review-setting-row slider",
            span { "{label}" }
            div { class: "review-slider-track wide",
                div { class: "review-slider-fill", style: "width:48%;" }
                div { class: "review-slider-knob", style: "left:calc(48% - 10px);" }
            }
            span { class: "muted", "{value}" }
        }
    }
}

#[cfg(target_os = "android")]
fn render_log_path_card() -> Element {
    let path = log_path().unwrap_or_default();
    rsx! {
        div { class: "review-info-card review-note-card",
            div { class: "review-info-title", "Log Path" }
            div { class: "review-info-meta", "{path}" }
        }
    }
}

#[cfg(not(target_os = "android"))]
fn render_log_path_card() -> Element {
    rsx! {}
}

fn document_title(snapshot: &AppState) -> String {
    if let Some(sample) = embedded_sample(&snapshot.path_input) {
        return sample.title.to_string();
    }
    Path::new(&snapshot.path_input)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Embedded Demo")
        .to_string()
}

fn layer_kind_label(layer: &Layer) -> &'static str {
    if layer.as_tile().is_some() {
        "Tile Layer"
    } else {
        "Object Layer"
    }
}

fn layer_thumb_variant(index: usize, layer: &Layer) -> &'static str {
    let lower = layer.name().to_ascii_lowercase();
    if lower.contains("ui") || lower.contains("object") {
        "ui"
    } else if lower.contains("decor") || lower.contains("fringe") || lower.contains("over") {
        "decor"
    } else if lower.contains("foreground") {
        "foreground"
    } else if lower.contains("obstacle") || lower.contains("collision") {
        "obstacles"
    } else if lower.contains("background") {
        "background"
    } else if index == 0 {
        "ground"
    } else {
        "foreground"
    }
}

fn selected_tile_style(snapshot: &AppState, session: &EditorSession, selected_gid: u32) -> String {
    let Some(reference) = session.document().map.tile_reference_for_gid(selected_gid) else {
        return String::new();
    };

    palette_tile_style(
        session.document(),
        &snapshot.image_cache,
        &PaletteTile {
            gid: selected_gid,
            tileset_index: reference.tileset_index,
            local_id: reference.local_id,
        },
    )
}

fn collect_objects(session: &EditorSession) -> Vec<MobileObjectSummary> {
    let mut objects = Vec::new();
    for (layer_index, layer) in session.document().map.layers.iter().enumerate() {
        if let Some(object_layer) = layer.as_object() {
            for object in &object_layer.objects {
                objects.push(MobileObjectSummary {
                    layer_index,
                    object_id: object.id,
                    name: object.name.clone(),
                    shape: object.shape.clone(),
                });
            }
        }
    }
    objects
}

fn create_object_on_first_object_layer(state: &mut AppState, shape: ObjectShape) {
    let object_layer_index = match state.session.as_ref() {
        Some(session) => session
            .document()
            .map
            .layers
            .iter()
            .enumerate()
            .find(|(_, layer)| layer.as_object().is_some())
            .map(|(index, _)| index),
        None => {
            state.status = "Load a map first.".to_string();
            return;
        }
    };

    let Some(object_layer_index) = object_layer_index else {
        state.status = "No object layer is available yet.".to_string();
        return;
    };

    state.active_layer = object_layer_index;
    create_object(state, shape);
}
