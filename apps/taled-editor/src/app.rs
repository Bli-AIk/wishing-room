use dioxus::prelude::*;
use taled_core::{EditorSession, Layer};

use crate::{
    app_state::{AppState, Tool},
    edit_ops::{cancel_tile_selection_transfer, toggle_layer_lock, toggle_layer_visibility},
    mobile_review::render_mobile_shell,
    mobile_review_styles::MOBILE_REVIEW_STYLES,
    session_ops::{adjust_zoom, load_sample, open_document, save_as_document, save_document},
    styles::STYLES,
    theme::{THEME_STYLE_OVERRIDES, runtime_theme_css},
    ui_canvas::render_canvas,
    ui_inspector::{render_inspector, render_palette},
};

#[cfg(target_arch = "wasm32")]
use crate::web_diag;

#[component]
pub(crate) fn App() -> Element {
    let state = use_signal(AppState::default);
    let snapshot = state.read().clone();

    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    use_effect(move || {
        crate::platform::mark_app_rendered();
    });

    let theme_css = runtime_theme_css(snapshot.theme_choice, &snapshot.custom_theme);

    rsx! {
        style { "{STYLES}{MOBILE_REVIEW_STYLES}{theme_css}{THEME_STYLE_OVERRIDES}" }
        div { class: "app-shell",
            {render_topbar(&snapshot, state)}
            div { class: "workspace",
                {render_desktop_left_panel(&snapshot, state)}
                {render_canvas(&snapshot, state)}
                div { class: "panel right desktop-panel",
                    {render_palette(&snapshot, state)}
                    {render_inspector(&snapshot, state)}
                }
            }
            {render_mobile_shell(&snapshot, state)}
            {render_web_log_panel(&snapshot, state)}
        }
    }
}

fn render_topbar(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "topbar",
            input {
                class: "desktop-file-control",
                value: snapshot.path_input.clone(),
                oninput: move |event| state.write().path_input = event.value(),
            }
            button {
                class: "desktop-file-control",
                onclick: move |_| open_document(&mut state.write()),
                "Open TMX"
            }
            button {
                class: "desktop-file-control",
                onclick: move |_| load_sample(&mut state.write()),
                "Load Sample"
            }
            button {
                class: "desktop-file-control",
                onclick: move |_| save_document(&mut state.write()),
                "Save"
            }
            input {
                class: "desktop-file-control",
                value: snapshot.save_as_input.clone(),
                oninput: move |event| state.write().save_as_input = event.value(),
            }
            button {
                class: "desktop-file-control",
                onclick: move |_| save_as_document(&mut state.write()),
                "Save As"
            }
            {render_web_log_controls(snapshot, state)}
            button {
                class: "desktop-file-control",
                onclick: move |_| {
                    let mut state = state.write();
                    if state.session.as_mut().is_some_and(EditorSession::undo) {
                        state.status = "Undo applied.".to_string();
                    } else {
                        state.status = "Nothing to undo.".to_string();
                    }
                },
                "Undo"
            }
            button {
                class: "desktop-file-control",
                onclick: move |_| {
                    let mut state = state.write();
                    if state.session.as_mut().is_some_and(EditorSession::redo) {
                        state.status = "Redo applied.".to_string();
                    } else {
                        state.status = "Nothing to redo.".to_string();
                    }
                },
                "Redo"
            }
            div { class: "status", "{snapshot.status}" }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn render_web_log_controls(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let show_web_logs = snapshot.show_web_logs;
    rsx! {
        button {
            onclick: move |_| state.write().show_web_logs = !show_web_logs,
            if show_web_logs { "Hide Logs" } else { "Logs" }
        }
        button {
            onclick: move |_| {
                let mut state = state.write();
                match web_diag::download_logs() {
                    Ok(()) => state.status = "Downloaded taled-web.log.".to_string(),
                    Err(error) => state.status = format!("Log download failed: {error}"),
                }
            },
            "Download Log"
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_web_log_controls(_snapshot: &AppState, _state: Signal<AppState>) -> Element {
    rsx! {}
}

#[cfg(target_arch = "wasm32")]
fn render_web_log_panel(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    if !snapshot.show_web_logs {
        return rsx! {};
    }

    let lines = web_diag::entries().join("\n");
    rsx! {
        div { class: "web-log-panel",
            div { class: "inline-row",
                strong { "Web Log" }
                button {
                    onclick: move |_| {
                        let mut state = state.write();
                        match web_diag::download_logs() {
                            Ok(()) => state.status = "Downloaded taled-web.log.".to_string(),
                            Err(error) => state.status = format!("Log download failed: {error}"),
                        }
                    },
                    "Download"
                }
                button {
                    onclick: move |_| state.write().show_web_logs = false,
                    "Close"
                }
            }
            pre { "{lines}" }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_web_log_panel(_snapshot: &AppState, _state: Signal<AppState>) -> Element {
    rsx! {}
}

fn render_desktop_left_panel(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "panel desktop-panel",
            h2 { "Tools" }
            div { class: "tool-grid",
                {tool_button(snapshot, state, Tool::Hand, "Hand")}
                {tool_button(snapshot, state, Tool::Paint, "Paint")}
                {tool_button(snapshot, state, Tool::Fill, "Fill")}
                {tool_button(snapshot, state, Tool::ShapeFill, "Shape Fill")}
                {tool_button(snapshot, state, Tool::Erase, "Erase")}
                {tool_button(snapshot, state, Tool::Select, "Select")}
                {tool_button(snapshot, state, Tool::AddRectangle, "Rect")}
                {tool_button(snapshot, state, Tool::AddPoint, "Point")}
            }

            h2 { "View" }
            div { class: "zoom-grid",
                button { onclick: move |_| adjust_zoom(&mut state.write(), -25), "- Zoom" }
                button { onclick: move |_| adjust_zoom(&mut state.write(), 25), "+ Zoom" }
                button { onclick: move |_| state.write().pan_y -= 32, "Pan Up" }
                button { onclick: move |_| state.write().pan_y += 32, "Pan Down" }
                button { onclick: move |_| state.write().pan_x -= 32, "Pan Left" }
                button { onclick: move |_| state.write().pan_x += 32, "Pan Right" }
            }

            {render_layers_section(snapshot, state)}
        }
    }
}

fn render_layers_section(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    rsx! {
        h2 { "Layers" }
        if let Some(session) = snapshot.session.as_ref() {
            div { class: "layer-list",
                for (index, layer) in session.document().map.layers.iter().enumerate() {
                    div {
                        key: "layer-{index}",
                        class: if index == snapshot.active_layer { "layer-row active" } else { "layer-row" },
                        button {
                            class: "name",
                            onclick: move |_| {
                                let mut state = state.write();
                                cancel_tile_selection_transfer(&mut state);
                                state.active_layer = index;
                                state.selected_object = None;
                                state.tile_selection = None;
                                state.tile_selection_cells = None;
                                state.tile_selection_preview = None;
                                state.tile_selection_preview_cells = None;
                                state.tile_selection_closing = None;
                                state.tile_selection_closing_cells = None;
                                state.tile_selection_closing_started_at = None;
                                state.tile_selection_last_tap_at = None;
                            },
                            span { class: "layer-name-stack",
                                span { "{layer.name()}" }
                                span { class: "layer-kind", "{layer_kind_label(layer)}" }
                            }
                        }
                        button {
                            onclick: move |_| toggle_layer_visibility(&mut state.write(), index),
                            if layer.visible() { "Visible" } else { "Hidden" }
                        }
                        button {
                            onclick: move |_| toggle_layer_lock(&mut state.write(), index),
                            if layer.locked() { "Locked" } else { "Unlocked" }
                        }
                    }
                }
            }
        }
    }
}

fn tool_button(
    snapshot: &AppState,
    mut state: Signal<AppState>,
    tool: Tool,
    label: &'static str,
) -> Element {
    let class = if snapshot.tool == tool { "active" } else { "" };
    rsx! {
        button {
            class: class,
            onclick: move |_| {
                let mut state = state.write();
                cancel_tile_selection_transfer(&mut state);
                state.tool = tool;
                state.shape_fill_preview = None;
                state.tile_selection_closing = None;
                state.tile_selection_closing_cells = None;
                state.tile_selection_closing_started_at = None;
                state.tile_selection_last_tap_at = None;
            },
            "{label}"
        }
    }
}

fn layer_kind_label(layer: &Layer) -> &'static str {
    if layer.as_tile().is_some() {
        "Tile Layer"
    } else {
        "Object Layer"
    }
}
