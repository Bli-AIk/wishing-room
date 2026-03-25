use std::{
    path::Path,
    time::{Duration, Instant},
};

use dioxus::prelude::*;
use futures_timer::Delay;
use taled_core::{EditorSession, Layer, ObjectShape};

use crate::{
    app_state::{AppState, MobileScreen, MobileTransition, PaletteTile, TileSelectionRegion, Tool},
    edit_ops::{
        confirm_tile_selection, copy_tile_selection, create_object, delete_selected_object,
        delete_tile_selection, flip_tile_selection_horizontally, nudge_selected_object,
        rename_selected_object, rotate_tile_selection_clockwise, selected_object_view,
        toggle_layer_lock, toggle_layer_visibility,
    },
    embedded_samples::{embedded_sample, embedded_sample_thumb, embedded_samples},
    session_ops::{
        adjust_zoom, adjust_zoom_around_view_center, animate_camera_to_center,
        animate_camera_to_fit_map, apply_redo, apply_undo, load_embedded_sample, load_sample,
        save_document,
    },
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReviewToolbarKind {
    Tile,
    Object,
}

const CONTROL_DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(320);
const JOYSTICK_LOOP_INTERVAL: Duration = Duration::from_millis(16);
const ZOOM_LOOP_INTERVAL: Duration = Duration::from_millis(28);
const JOYSTICK_TAP_MAX_DISTANCE: f64 = 0.18;
const JOYSTICK_STEP_PER_TICK: f64 = 7.0;
const ZOOM_STEP_PER_TICK: f64 = 3.0;

pub(crate) fn render_mobile_shell(snapshot: &AppState, state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "mobile-shell review-shell",
            match snapshot.mobile_screen {
                MobileScreen::Dashboard => rsx! { {render_dashboard(snapshot, state)} },
                MobileScreen::Editor => rsx! { {render_editor(snapshot, state)} },
                MobileScreen::Tilesets => rsx! { {render_tilesets(snapshot, state)} },
                MobileScreen::Layers => rsx! { {render_layers(snapshot, state)} },
                MobileScreen::Objects => rsx! { {render_objects(snapshot, state)} },
                MobileScreen::Properties => rsx! { {render_properties(snapshot, state)} },
                MobileScreen::Settings => rsx! { {render_settings(snapshot, state)} },
            }
        }
    }
}

fn review_page_class(snapshot: &AppState, base: &'static str) -> String {
    let transition_class = match snapshot.mobile_transition {
        MobileTransition::None => "",
        MobileTransition::HorizontalForward => " review-transition-horizontal-forward",
        MobileTransition::HorizontalBackward => " review-transition-horizontal-backward",
        MobileTransition::VerticalForward => " review-transition-vertical-forward",
        MobileTransition::VerticalBackward => " review-transition-vertical-backward",
    };

    format!("{base}{transition_class}")
}

fn review_page_key(snapshot: &AppState, label: &'static str) -> String {
    format!("{label}-{}", snapshot.mobile_transition_nonce)
}

fn navigate_mobile_screen(state: &mut AppState, next: MobileScreen) {
    let current = state.mobile_screen;
    if current == next {
        return;
    }

    state.mobile_transition = match (current, next) {
        (MobileScreen::Dashboard, MobileScreen::Editor) => MobileTransition::HorizontalForward,
        (MobileScreen::Editor, MobileScreen::Dashboard) => MobileTransition::HorizontalBackward,
        (MobileScreen::Editor, MobileScreen::Tilesets)
        | (MobileScreen::Editor, MobileScreen::Layers)
        | (MobileScreen::Editor, MobileScreen::Objects)
        | (MobileScreen::Editor, MobileScreen::Properties) => MobileTransition::VerticalForward,
        (MobileScreen::Tilesets, MobileScreen::Editor)
        | (MobileScreen::Layers, MobileScreen::Editor)
        | (MobileScreen::Objects, MobileScreen::Editor)
        | (MobileScreen::Properties, MobileScreen::Editor) => MobileTransition::VerticalBackward,
        _ => MobileTransition::None,
    };
    state.mobile_transition_nonce = state.mobile_transition_nonce.wrapping_add(1);
    state.mobile_screen = next;
}

fn render_dashboard(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let page_key = review_page_key(snapshot, "dashboard");
    let page_class = review_page_class(snapshot, "review-page");
    rsx! {
        div { key: "{page_key}", class: "{page_class}",
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
                                    navigate_mobile_screen(&mut state, MobileScreen::Editor);
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

    let layers: Vec<(usize, String, bool, bool, bool)> = session
        .document()
        .map
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            (
                index,
                layer.name().to_string(),
                layer.as_tile().is_some(),
                layer.visible(),
                layer.locked(),
            )
        })
        .collect();
    let active_layer_summary = session
        .document()
        .map
        .layer(snapshot.active_layer)
        .map(|layer| (layer.name().to_string(), layer_kind_label(layer)))
        .unwrap_or_else(|| ("No layer".to_string(), "Unavailable"));
    let toolbar_kind = active_toolbar_kind(session, snapshot.active_layer);
    let can_undo = session.can_undo();
    let can_redo = session.can_redo();
    let palette: Vec<PaletteTile> = collect_palette(session.document())
        .into_iter()
        .take(24)
        .collect();
    let grid_style = editor_grid_style(snapshot, session);
    let page_key = review_page_key(snapshot, "editor");
    let page_class = review_page_class(snapshot, "review-page review-editor-page");
    let selection_action_bar = tile_selection_action_bar(snapshot, session, state);

    rsx! {
        div { key: "{page_key}", class: "{page_class}",
            {review_top_bar_inactive_right(
                document_title(snapshot),
                ("Back", MobileScreen::Dashboard),
                "Settings",
                state,
            )}
            div { class: "review-tile-strip review-tile-strip-top",
                for tile in palette.clone() {
                    button {
                        key: "review-top-tile-{tile.gid}",
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
            div { class: "review-editor-canvas", style: grid_style,
                div { class: "review-map-surface review-map-live",
                    {crate::ui_canvas::render_canvas(snapshot, state)}
                }
                {selection_action_bar}
                div { class: "review-history-float",
                    button {
                        class: if can_undo {
                            "review-history-button"
                        } else {
                            "review-history-button disabled"
                        },
                        disabled: !can_undo,
                        onclick: move |_| apply_undo(&mut state.write()),
                        aria_label: "Undo",
                        {review_history_icon(true)}
                    }
                    button {
                        class: if can_redo {
                            "review-history-button"
                        } else {
                            "review-history-button disabled"
                        },
                        disabled: !can_redo,
                        onclick: move |_| apply_redo(&mut state.write()),
                        aria_label: "Redo",
                        {review_history_icon(false)}
                    }
                }
                ReviewPanJoystick { state }
                ReviewZoomControl { zoom_percent: snapshot.zoom_percent, state }
                div {
                    class: if snapshot.layers_panel_expanded {
                        "review-layer-float expanded"
                    } else {
                        "review-layer-float"
                    },
                    button {
                        class: "review-layer-float-title",
                        onclick: move |_| {
                            let mut state = state.write();
                            state.layers_panel_expanded = !state.layers_panel_expanded;
                        },
                        span { class: "review-layer-float-title-stack",
                            span { "Layers" }
                            span { class: "review-layer-float-current", "{active_layer_summary.0}" }
                        }
                        span { class: "review-layer-float-title-icon", {review_layer_chevron_icon(snapshot.layers_panel_expanded)} }
                    }
                    div {
                        class: if snapshot.layers_panel_expanded {
                            "review-layer-float-body expanded"
                        } else {
                            "review-layer-float-body"
                        },
                        div { class: "review-layer-float-list",
                            for (index, name, is_tile_layer, visible, locked) in layers {
                                div {
                                    key: "review-float-layer-{index}",
                                    class: if snapshot.active_layer == index {
                                        "review-layer-float-item active"
                                    } else {
                                        "review-layer-float-item"
                                    },
                                    span { class: if visible { "review-eye on" } else { "review-eye off" }, {review_eye_icon(visible)} }
                                    button {
                                        class: "review-layer-float-switch",
                                        onclick: move |_| {
                                            let mut state = state.write();
                                            set_review_active_layer_kind(
                                                &mut state,
                                                index,
                                                if is_tile_layer {
                                                    ReviewToolbarKind::Tile
                                                } else {
                                                    ReviewToolbarKind::Object
                                                },
                                            );
                                        },
                                        span { class: "review-layer-float-kind", {review_layer_kind_icon(is_tile_layer)} }
                                        span { class: "review-layer-float-name", "{name}" }
                                    }
                                    span {
                                        class: if locked {
                                            "review-lock on"
                                        } else {
                                            "review-lock off"
                                        },
                                        {review_lock_icon(locked)}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "review-editor-toolbar",
                {review_tool_row(snapshot, state, toolbar_kind)}
            }
            {review_nav(snapshot, state, false)}
        }
    }
}

fn tile_selection_action_bar(
    snapshot: &AppState,
    session: &EditorSession,
    state: Signal<AppState>,
) -> Element {
    let Some(selection) = snapshot.tile_selection else {
        return rsx! { Fragment {} };
    };
    if snapshot.tile_selection_preview.is_some() || snapshot.tool != Tool::Select {
        return rsx! { Fragment {} };
    }
    if session
        .document()
        .map
        .layer(snapshot.active_layer)
        .and_then(Layer::as_tile)
        .is_none()
    {
        return rsx! { Fragment {} };
    }

    let action_bar_style = tile_selection_action_bar_style(snapshot, session, selection);

    rsx! {
        div { class: "review-selection-actions", style: "{action_bar_style}",
            {review_selection_action_button(
                state,
                ReviewSelectionAction::Copy,
                "Copy",
                copy_tile_selection,
            )}
            {review_selection_action_button(
                state,
                ReviewSelectionAction::Flip,
                "Flip",
                flip_tile_selection_horizontally,
            )}
            {review_selection_action_button(
                state,
                ReviewSelectionAction::Rotate,
                "Rotate",
                rotate_tile_selection_clockwise,
            )}
            {review_selection_action_button(
                state,
                ReviewSelectionAction::Delete,
                "Delete",
                delete_tile_selection,
            )}
            {review_selection_action_button(
                state,
                ReviewSelectionAction::Confirm,
                "Confirm",
                confirm_tile_selection,
            )}
        }
    }
}

fn tile_selection_action_bar_style(
    snapshot: &AppState,
    session: &EditorSession,
    selection: TileSelectionRegion,
) -> String {
    let map = &session.document().map;
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    let zoom = f64::from(snapshot.zoom_percent) / 100.0;
    let left = f64::from(snapshot.pan_x) + f64::from(min_x * map.tile_width) * zoom;
    let right = f64::from(snapshot.pan_x) + f64::from((max_x + 1) * map.tile_width) * zoom;
    let top = f64::from(snapshot.pan_y) + f64::from(min_y * map.tile_height) * zoom;
    let bottom = f64::from(snapshot.pan_y) + f64::from((max_y + 1) * map.tile_height) * zoom;
    let host_width = snapshot
        .canvas_host_size
        .map(|(width, _)| width)
        .unwrap_or(384.0);
    let center_x = ((left + right) * 0.5).clamp(92.0, host_width - 92.0);

    if top >= 86.0 {
        format!(
            "left:{center_x:.1}px;top:{top:.1}px;transform:translate(-50%, calc(-100% - 10px));"
        )
    } else {
        format!("left:{center_x:.1}px;top:{bottom:.1}px;transform:translate(-50%, 10px);")
    }
}

fn selection_bounds(selection: TileSelectionRegion) -> (u32, u32, u32, u32) {
    (
        selection.start_cell.0.min(selection.end_cell.0),
        selection.start_cell.1.min(selection.end_cell.1),
        selection.start_cell.0.max(selection.end_cell.0),
        selection.start_cell.1.max(selection.end_cell.1),
    )
}

#[derive(Clone, Copy)]
enum ReviewSelectionAction {
    Copy,
    Flip,
    Rotate,
    Delete,
    Confirm,
}

fn review_selection_action_button(
    mut state: Signal<AppState>,
    action: ReviewSelectionAction,
    label: &'static str,
    apply: fn(&mut AppState),
) -> Element {
    rsx! {
        button {
            class: "review-selection-action",
            onclick: move |_| apply(&mut state.write()),
            div { class: "review-selection-action-icon", {review_selection_action_icon(action)} }
            span { "{label}" }
        }
    }
}

fn review_selection_action_icon(action: ReviewSelectionAction) -> Element {
    match action {
        ReviewSelectionAction::Copy => rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "8", y: "8", width: "11", height: "11", rx: "2.4" }
                path { d: "M5 15V7a2 2 0 0 1 2-2h8" }
            }
        },
        ReviewSelectionAction::Flip => rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M8 7 4 12l4 5" }
                path { d: "M16 7l4 5-4 5" }
                path { d: "M12 5v14" }
            }
        },
        ReviewSelectionAction::Rotate => rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M8 6H4v4" }
                path { d: "M4 10a8 8 0 1 1 2.3 5.7" }
            }
        },
        ReviewSelectionAction::Delete => rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M4 7h16" }
                path { d: "M9 7V5h6v2" }
                path { d: "M7 7l1 12h8l1-12" }
                path { d: "M10 11v5" }
                path { d: "M14 11v5" }
            }
        },
        ReviewSelectionAction::Confirm => rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "m6 12 4 4 8-8" }
            }
        },
    }
}

#[component]
fn ReviewPanJoystick(state: Signal<AppState>) -> Element {
    let mut active_pointer = use_signal(|| None::<i32>);
    let mut joystick_vector = use_signal(|| (0.0_f64, 0.0_f64));
    let mut knob_offset = use_signal(|| (0.0_f64, 0.0_f64));
    let mut last_tap_at = use_signal(|| None::<Instant>);
    let mut press_started_at = use_signal(|| None::<Instant>);
    let mut tap_candidate = use_signal(|| false);
    let mut loop_token = use_signal(|| 0_u64);

    rsx! {
        div {
            class: "review-pan-joystick",
            onpointerdown: move |event| {
                if event.pointer_type() != "touch" {
                    return;
                }
                event.prevent_default();
                let (vector_x, vector_y, offset_x, offset_y) = pan_joystick_motion(&event);
                active_pointer.set(Some(event.pointer_id()));
                joystick_vector.set((vector_x, vector_y));
                knob_offset.set((offset_x, offset_y));
                press_started_at.set(Some(Instant::now()));
                tap_candidate.set(true);
                let token = {
                    let next = *loop_token.read() + 1;
                    loop_token.set(next);
                    next
                };
                spawn_pan_joystick_loop(state, active_pointer, joystick_vector, loop_token, token);
            },
            onpointermove: move |event| {
                if event.pointer_type() != "touch" {
                    return;
                }
                let Some(pointer_id) = *active_pointer.read() else {
                    return;
                };
                if pointer_id != event.pointer_id() {
                    return;
                }
                event.prevent_default();
                let (vector_x, vector_y, offset_x, offset_y) = pan_joystick_motion(&event);
                joystick_vector.set((vector_x, vector_y));
                knob_offset.set((offset_x, offset_y));
                if vector_x.abs().max(vector_y.abs()) > JOYSTICK_TAP_MAX_DISTANCE {
                    tap_candidate.set(false);
                }
            },
            onpointerup: move |event| {
                if event.pointer_type() != "touch" {
                    return;
                }
                let Some(pointer_id) = *active_pointer.read() else {
                    return;
                };
                if pointer_id != event.pointer_id() {
                    return;
                }
                event.prevent_default();
                active_pointer.set(None);
                joystick_vector.set((0.0, 0.0));
                knob_offset.set((0.0, 0.0));
                let started_at = press_started_at.write().take();
                let was_tap_candidate = *tap_candidate.read();
                tap_candidate.set(false);
                if is_control_tap(started_at, was_tap_candidate) {
                    handle_pan_joystick_tap(state, &mut last_tap_at);
                }
            },
            onpointercancel: move |event| {
                if event.pointer_type() != "touch" {
                    return;
                }
                active_pointer.set(None);
                joystick_vector.set((0.0, 0.0));
                knob_offset.set((0.0, 0.0));
                press_started_at.set(None);
                tap_candidate.set(false);
            },
            div { class: "review-pan-joystick-ring" }
            div { class: "review-pan-joystick-center-mark", {review_dpad_center_icon()} }
            div {
                class: "review-pan-joystick-knob",
                style: joystick_knob_style(*knob_offset.read()),
            }
        }
    }
}

#[component]
fn ReviewZoomControl(zoom_percent: i32, state: Signal<AppState>) -> Element {
    let mut active_pointer = use_signal(|| None::<i32>);
    let mut zoom_vector = use_signal(|| 0.0_f64);
    let mut knob_offset_x = use_signal(|| 0.0_f64);
    let mut last_tap_at = use_signal(|| None::<Instant>);
    let mut press_started_at = use_signal(|| None::<Instant>);
    let mut tap_candidate = use_signal(|| false);
    let mut loop_token = use_signal(|| 0_u64);

    rsx! {
        div {
            class: "review-zoom-control",
            onpointerdown: move |event| {
                if event.pointer_type() != "touch" {
                    return;
                }
                event.prevent_default();
                let (value, offset_x) = zoom_control_motion(&event);
                active_pointer.set(Some(event.pointer_id()));
                zoom_vector.set(value);
                knob_offset_x.set(offset_x);
                press_started_at.set(Some(Instant::now()));
                tap_candidate.set(true);
                let token = {
                    let next = *loop_token.read() + 1;
                    loop_token.set(next);
                    next
                };
                spawn_zoom_control_loop(state, active_pointer, zoom_vector, loop_token, token);
            },
            onpointermove: move |event| {
                if event.pointer_type() != "touch" {
                    return;
                }
                let Some(pointer_id) = *active_pointer.read() else {
                    return;
                };
                if pointer_id != event.pointer_id() {
                    return;
                }
                event.prevent_default();
                let (value, offset_x) = zoom_control_motion(&event);
                zoom_vector.set(value);
                knob_offset_x.set(offset_x);
                if value.abs() > JOYSTICK_TAP_MAX_DISTANCE {
                    tap_candidate.set(false);
                }
            },
            onpointerup: move |event| {
                if event.pointer_type() != "touch" {
                    return;
                }
                let Some(pointer_id) = *active_pointer.read() else {
                    return;
                };
                if pointer_id != event.pointer_id() {
                    return;
                }
                event.prevent_default();
                active_pointer.set(None);
                zoom_vector.set(0.0);
                knob_offset_x.set(0.0);
                let started_at = press_started_at.write().take();
                let was_tap_candidate = *tap_candidate.read();
                tap_candidate.set(false);
                if is_control_tap(started_at, was_tap_candidate) {
                    handle_zoom_control_tap(state, &mut last_tap_at);
                }
            },
            onpointercancel: move |event| {
                if event.pointer_type() != "touch" {
                    return;
                }
                active_pointer.set(None);
                zoom_vector.set(0.0);
                knob_offset_x.set(0.0);
                press_started_at.set(None);
                tap_candidate.set(false);
            },
            div { class: "review-zoom-control-glyph minus", "-" }
            div { class: "review-zoom-control-glyph plus", "+" }
            div { class: "review-zoom-control-track" }
            div {
                class: "review-zoom-control-knob",
                style: zoom_knob_style(*knob_offset_x.read()),
                span { class: "review-zoom-control-label", "{zoom_percent}%" }
            }
        }
    }
}

fn spawn_pan_joystick_loop(
    mut state: Signal<AppState>,
    active_pointer: Signal<Option<i32>>,
    joystick_vector: Signal<(f64, f64)>,
    loop_token: Signal<u64>,
    token: u64,
) {
    spawn(async move {
        loop {
            Delay::new(JOYSTICK_LOOP_INTERVAL).await;
            if active_pointer.read().is_none() || *loop_token.read() != token {
                break;
            }
            let (vector_x, vector_y) = *joystick_vector.read();
            let step_x = (vector_x * JOYSTICK_STEP_PER_TICK).round() as i32;
            let step_y = (vector_y * JOYSTICK_STEP_PER_TICK).round() as i32;
            if step_x == 0 && step_y == 0 {
                continue;
            }
            let mut app = state.write();
            app.pan_x -= step_x;
            app.pan_y -= step_y;
        }
    });
}

fn spawn_zoom_control_loop(
    mut state: Signal<AppState>,
    active_pointer: Signal<Option<i32>>,
    zoom_vector: Signal<f64>,
    loop_token: Signal<u64>,
    token: u64,
) {
    spawn(async move {
        loop {
            Delay::new(ZOOM_LOOP_INTERVAL).await;
            if active_pointer.read().is_none() || *loop_token.read() != token {
                break;
            }
            let value = *zoom_vector.read();
            let delta = (value * ZOOM_STEP_PER_TICK).round() as i32;
            if delta == 0 {
                continue;
            }
            adjust_zoom_around_view_center(&mut state.write(), delta);
        }
    });
}

fn handle_pan_joystick_tap(mut state: Signal<AppState>, last_tap_at: &mut Signal<Option<Instant>>) {
    let now = Instant::now();
    let is_double_tap = last_tap_at
        .read()
        .is_some_and(|last_tap| now.duration_since(last_tap) <= CONTROL_DOUBLE_TAP_WINDOW);
    if is_double_tap {
        animate_camera_to_center(&mut state.write());
        last_tap_at.set(None);
    } else {
        last_tap_at.set(Some(now));
    }
}

fn handle_zoom_control_tap(mut state: Signal<AppState>, last_tap_at: &mut Signal<Option<Instant>>) {
    let now = Instant::now();
    let is_double_tap = last_tap_at
        .read()
        .is_some_and(|last_tap| now.duration_since(last_tap) <= CONTROL_DOUBLE_TAP_WINDOW);
    if is_double_tap {
        animate_camera_to_fit_map(&mut state.write());
        last_tap_at.set(None);
    } else {
        last_tap_at.set(Some(now));
    }
}

fn is_control_tap(started_at: Option<Instant>, was_tap_candidate: bool) -> bool {
    was_tap_candidate
        && started_at.is_some_and(|started_at| started_at.elapsed() <= Duration::from_millis(260))
}

fn pan_joystick_motion(event: &Event<PointerData>) -> (f64, f64, f64, f64) {
    const CONTROL_SIZE: f64 = 92.0;
    const INPUT_RADIUS: f64 = 28.0;
    const KNOB_TRAVEL: f64 = 22.0;

    let point = event.element_coordinates();
    let center = CONTROL_SIZE * 0.5;
    let raw_x = point.x - center;
    let raw_y = point.y - center;
    let distance = (raw_x * raw_x + raw_y * raw_y).sqrt();
    let clamped = if distance > INPUT_RADIUS {
        INPUT_RADIUS / distance
    } else {
        1.0
    };
    let vector_x = (raw_x * clamped) / INPUT_RADIUS;
    let vector_y = (raw_y * clamped) / INPUT_RADIUS;
    let offset_x = vector_x * KNOB_TRAVEL;
    let offset_y = vector_y * KNOB_TRAVEL;
    (vector_x, vector_y, offset_x, offset_y)
}

fn zoom_control_motion(event: &Event<PointerData>) -> (f64, f64) {
    const CONTROL_WIDTH: f64 = 118.0;
    const INPUT_HALF_WIDTH: f64 = 34.0;
    const KNOB_TRAVEL: f64 = 26.0;

    let point = event.element_coordinates();
    let center_x = CONTROL_WIDTH * 0.5;
    let raw_x = point.x - center_x;
    let vector_x = (raw_x / INPUT_HALF_WIDTH).clamp(-1.0, 1.0);
    let offset_x = vector_x * KNOB_TRAVEL;
    (vector_x, offset_x)
}

fn joystick_knob_style((offset_x, offset_y): (f64, f64)) -> String {
    format!("transform: translate({offset_x:.1}px, {offset_y:.1}px);")
}

fn zoom_knob_style(offset_x: f64) -> String {
    format!("transform: translateX({offset_x:.1}px);")
}

fn render_tilesets(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return render_missing_screen(
            "Tile Property Editor".to_string(),
            "Load an embedded TMX sample before opening tilesets.",
            state,
        );
    };

    let palette = collect_palette(session.document());
    let selected_gid = snapshot.selected_gid;
    let selected_reference = session.document().map.tile_reference_for_gid(selected_gid);
    let sheet_style = tileset_sheet_style(session.document(), selected_gid);
    let selected_label = selected_reference
        .as_ref()
        .map(|reference| format!("Selected Tile: ID {}", reference.local_id))
        .unwrap_or_else(|| "Selected Tile: None".to_string());
    let tile_name = selected_reference
        .as_ref()
        .map(|reference| format!("{} {}", reference.tileset.tileset.name, reference.local_id))
        .unwrap_or_else(|| "None".to_string());
    let tile_type = selected_reference
        .as_ref()
        .map(|reference| format!("{} Tileset", reference.tileset.tileset.name))
        .unwrap_or_else(|| "Unknown".to_string());
    let property_count = 0usize;
    let page_key = review_page_key(snapshot, "tilesets");
    let page_class = review_page_class(snapshot, "review-page");

    rsx! {
        div { key: "{page_key}", class: "{page_class}",
            {review_top_bar(
                "Tile Property Editor".to_string(),
                Some(("Back", MobileScreen::Editor)),
                Some(("Done", MobileScreen::Editor)),
                state,
            )}
            div { class: "review-body review-section-stack",
                div { class: "review-section-title", "Sprite Sheet View" }
                div { class: "review-tileset-sheet review-tileset-sheet-live", style: sheet_style,
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
                div { class: "review-selected-tile-summary", "{selected_label}" }
                div { class: "review-property-field-card",
                    div { class: "review-property-field-row",
                        span { class: "review-property-field-label", "Name:" }
                        div { class: "review-property-field-value", "{tile_name}" }
                    }
                    div { class: "review-property-field-row",
                        span { class: "review-property-field-label", "Type:" }
                        div { class: "review-property-field-value", "{tile_type}" }
                    }
                }
                div { class: "review-section-title with-gap", "Custom Properties" }
                div { class: "review-property-group-card",
                    div { class: "review-setting-row review-property-empty-row",
                        span { class: "muted", "No editable tile properties are available in the current Stage 1 data model." }
                    }
                    button {
                        class: "review-link review-property-add-link",
                        onclick: move |_| {
                            state.write().status =
                                format!("Tile properties are not implemented yet. Current selected gid: {selected_gid}.");
                        },
                        "+ Add Property"
                    }
                }
                div { class: "review-property-footer-note",
                    "{property_count} custom properties available. Collision editing stays out of scope for this pass."
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

    let layer_rows: Vec<(usize, String, bool, bool, bool, &'static str)> = session
        .document()
        .map
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            (
                index,
                layer.name().to_string(),
                layer.visible(),
                layer.locked(),
                layer.as_object().is_some(),
                layer_thumb_variant(index, layer),
            )
        })
        .collect();
    let page_key = review_page_key(snapshot, "layers");
    let page_class = review_page_class(snapshot, "review-page");

    rsx! {
        div { key: "{page_key}", class: "{page_class}",
            {review_top_bar(
                "Layer Manager".to_string(),
                Some(("Back", MobileScreen::Editor)),
                Some(("Done", MobileScreen::Editor)),
                state,
            )}
            div { class: "review-body review-list",
                for (index, layer_name, visible, locked, is_object_layer, thumb_variant) in layer_rows {
                    div {
                        key: "{index}",
                        class: if snapshot.active_layer == index {
                            "review-layer-row active"
                        } else {
                            "review-layer-row"
                        },
                        span { class: "review-menu-glyph", "≡" }
                        div { class: "review-layer-thumb {thumb_variant}" }
                        button {
                            class: "review-layer-name-button",
                            onclick: move |_| {
                                let mut state = state.write();
                                set_review_active_layer_kind(
                                    &mut state,
                                    index,
                                    if is_object_layer {
                                        ReviewToolbarKind::Object
                                    } else {
                                        ReviewToolbarKind::Tile
                                    },
                                );
                            },
                            span { class: "review-layer-title-stack",
                                span { class: "review-layer-name", "{layer_name}" }
                                span { class: "muted", if is_object_layer { "Object Layer" } else { "Tile Layer" } }
                            }
                        }
                        button {
                            class: if visible {
                                "review-eye on review-layer-toggle"
                            } else {
                                "review-eye off review-layer-toggle"
                            },
                            onclick: move |_| toggle_layer_visibility(&mut state.write(), index),
                            "o"
                        }
                        button {
                            class: if locked {
                                "review-lock on review-layer-toggle"
                            } else {
                                "review-lock off review-layer-toggle"
                            },
                            onclick: move |_| toggle_layer_lock(&mut state.write(), index),
                            "u"
                        }
                        div { class: "review-opacity",
                            span { if is_object_layer { "Object Layer" } else { "Tile Layer" } }
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
    let page_key = review_page_key(snapshot, "objects");
    let page_class = review_page_class(snapshot, "review-page");

    rsx! {
        div { key: "{page_key}", class: "{page_class}",
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
                                    set_review_active_layer_kind(
                                        &mut state,
                                        entry.layer_index,
                                        ReviewToolbarKind::Object,
                                    );
                                    state.selected_object = Some(entry.object_id);
                                    state.tile_selection = None;
                                    state.tile_selection_preview = None;
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
                        onclick: move |_| navigate_mobile_screen(&mut state.write(), MobileScreen::Editor),
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

fn render_properties(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let page_key = review_page_key(snapshot, "properties");
    let page_class = review_page_class(snapshot, "review-page");
    rsx! {
        div { key: "{page_key}", class: "{page_class}",
            {review_top_bar(
                "Properties".to_string(),
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
                                navigate_mobile_screen(&mut state, MobileScreen::Editor);
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
                                apply_undo(&mut state.write());
                            },
                            "Run"
                        }
                    }
                    div { class: "review-setting-row",
                        span { "Redo" }
                        button {
                            class: "review-link-button",
                            onclick: move |_| {
                                apply_redo(&mut state.write());
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

fn render_settings(snapshot: &AppState, state: Signal<AppState>) -> Element {
    let page_key = review_page_key(snapshot, "settings");
    let page_class = review_page_class(snapshot, "review-page");
    rsx! {
        div { key: "{page_key}", class: "{page_class}",
            {review_title_only_bar("App Settings".to_string())}
            div { class: "review-body review-section-stack",
                div { class: "review-caption", "Grid Settings" }
                div { class: "review-settings-card",
                    div { class: "review-setting-row",
                        span { "Grid Color" }
                        div { class: "review-color-chip",
                            span { class: "swatch", style: "background:#cccccc;" }
                            span { "#CCCCCC" }
                        }
                    }
                    div { class: "review-setting-row",
                        span { "Snapping" }
                        div { class: "review-toggle on", div { class: "knob" } }
                    }
                }
                div { class: "review-caption", "Theme" }
                div { class: "review-settings-card single",
                    div { class: "review-segmented",
                        button { class: "active", "Dark" }
                        button { "Light" }
                        button { "System" }
                    }
                }
                div { class: "review-caption", "Export Settings" }
                div { class: "review-settings-card",
                    div { class: "review-setting-row", span { "JSON" } div { class: "review-toggle on", div { class: "knob" } } }
                    div { class: "review-setting-row", span { "XML" } div { class: "review-toggle on", div { class: "knob" } } }
                    div { class: "review-setting-row", span { "PNG" } div { class: "review-toggle on", div { class: "knob" } } }
                }
            }
            {review_nav(snapshot, state, true)}
        }
    }
}

fn render_missing_screen(
    title: String,
    message: &'static str,
    mut state: Signal<AppState>,
) -> Element {
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
                    onclick: move |_| navigate_mobile_screen(&mut state.write(), MobileScreen::Dashboard),
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
                    onclick: move |_| navigate_mobile_screen(&mut state.write(), screen),
                    "{label}"
                }
            } else {
                div { class: "review-header-spacer" }
            }
            h1 { "{title}" }
            if let Some((label, screen)) = right {
                button {
                    class: "review-header-action right",
                    onclick: move |_| navigate_mobile_screen(&mut state.write(), screen),
                    "{label}"
                }
            } else {
                div { class: "review-header-spacer" }
            }
        }
    }
}

fn review_top_bar_inactive_right(
    title: String,
    left: (&'static str, MobileScreen),
    inactive_right: &'static str,
    mut state: Signal<AppState>,
) -> Element {
    rsx! {
        div { class: "review-header",
            button {
                class: "review-header-action left",
                onclick: move |_| navigate_mobile_screen(&mut state.write(), left.1),
                "{left.0}"
            }
            h1 { "{title}" }
            button {
                class: "review-header-action right",
                onclick: move |_| {
                    state.write().status =
                        "Editor-specific settings are not implemented yet.".to_string();
                },
                "{inactive_right}"
            }
        }
    }
}

fn review_title_only_bar(title: String) -> Element {
    rsx! {
        div { class: "review-header",
            div { class: "review-header-spacer" }
            h1 { "{title}" }
            div { class: "review-header-spacer" }
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
                {review_nav_button(snapshot, state, MobileScreen::Properties, "Properties")}
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
            onclick: move |_| navigate_mobile_screen(&mut state.write(), screen),
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
        "Properties" => rsx! {
            svg {
                class: "review-nav-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M5 6h14" }
                path { d: "M5 12h14" }
                path { d: "M5 18h14" }
                circle { cx: "9", cy: "6", r: "2" }
                circle { cx: "15", cy: "12", r: "2" }
                circle { cx: "11", cy: "18", r: "2" }
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

fn review_dpad_center_icon() -> Element {
    rsx! {
        svg {
            class: "review-dpad-center-svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "4" }
            path { d: "M12 3v3" }
            path { d: "M12 18v3" }
            path { d: "M3 12h3" }
            path { d: "M18 12h3" }
        }
    }
}

fn review_layer_chevron_icon(expanded: bool) -> Element {
    let rotation = if expanded { "180" } else { "0" };
    rsx! {
        svg {
            class: "review-inline-icon-svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.1",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            style: "transform: rotate({rotation}deg); transition: transform 180ms ease;",
            path { d: "m6 9 6 6 6-6" }
        }
    }
}

fn review_eye_icon(visible: bool) -> Element {
    if visible {
        rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M2 12s3.5-6 10-6 10 6 10 6-3.5 6-10 6S2 12 2 12z" }
                circle { cx: "12", cy: "12", r: "2.7" }
            }
        }
    } else {
        rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M3 3 21 21" }
                path { d: "M10.6 6.2A10.8 10.8 0 0 1 12 6c6.5 0 10 6 10 6a19 19 0 0 1-3 3.7" }
                path { d: "M6.6 6.7C3.9 8.4 2 12 2 12a18.7 18.7 0 0 0 5.7 5.2" }
                path { d: "M9.9 9.9A3 3 0 0 0 12 15a3 3 0 0 0 2.1-.9" }
            }
        }
    }
}

fn review_layer_kind_icon(is_tile_layer: bool) -> Element {
    if is_tile_layer {
        rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "4", y: "4", width: "16", height: "16", rx: "2" }
                path { d: "M12 4v16" }
                path { d: "M4 12h16" }
            }
        }
    } else {
        rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "5", y: "5", width: "14", height: "14", rx: "2.5" }
                circle { cx: "12", cy: "12", r: "2.5" }
            }
        }
    }
}

fn review_lock_icon(locked: bool) -> Element {
    if locked {
        rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "5", y: "11", width: "14", height: "9", rx: "2" }
                path { d: "M8 11V8.6A4 4 0 0 1 12 5a4 4 0 0 1 4 3.6V11" }
            }
        }
    } else {
        rsx! {
            svg {
                class: "review-inline-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "5", y: "11", width: "14", height: "9", rx: "2" }
                path { d: "M8 11V8.6A4 4 0 0 1 12 5a4 4 0 0 1 4 3.6" }
            }
        }
    }
}

fn review_history_icon(is_undo: bool) -> Element {
    let d = if is_undo {
        "M10 8 6 12l4 4"
    } else {
        "m14 8 4 4-4 4"
    };
    let tail = if is_undo {
        "M7 12h7a4 4 0 1 1 0 8"
    } else {
        "M17 12h-7a4 4 0 1 0 0 8"
    };

    rsx! {
        svg {
            class: "review-inline-icon-svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.9",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "{d}" }
            path { d: "{tail}" }
        }
    }
}

fn review_tool_row(
    snapshot: &AppState,
    state: Signal<AppState>,
    kind: ReviewToolbarKind,
) -> Element {
    let toolbar_key = match kind {
        ReviewToolbarKind::Tile => "tile-toolbar",
        ReviewToolbarKind::Object => "object-toolbar",
    };
    let toolbar_class = match kind {
        ReviewToolbarKind::Tile => "review-tool-row review-tool-row-live",
        ReviewToolbarKind::Object => "review-tool-row review-tool-row-live review-tool-row-object",
    };

    rsx! {
        div { key: "{toolbar_key}", class: "review-tool-row-shell review-tool-row-swap",
            {review_tool_button(snapshot, state, Tool::Hand, ReviewToolGlyph::Hand, "Hand", Some("review-tool review-tool-pinned"))}
            div { class: "review-tool-divider" }
            div { class: "{toolbar_class}",
                match kind {
                    ReviewToolbarKind::Tile => rsx! {
                        {review_tool_button(snapshot, state, Tool::Paint, ReviewToolGlyph::StampBrush, "Stamp", None)}
                        {review_placeholder_tool_button(
                            state,
                            ReviewToolGlyph::TerrainBrush,
                            "Terrain",
                            "Terrain Brush is not implemented yet."
                        )}
                        {review_tool_button(snapshot, state, Tool::Fill, ReviewToolGlyph::Fill, "Fill", None)}
                        {review_tool_button(
                            snapshot,
                            state,
                            Tool::ShapeFill,
                            ReviewToolGlyph::ShapeFill,
                            "Shape Fill",
                            None,
                        )}
                        {review_tool_button(snapshot, state, Tool::Erase, ReviewToolGlyph::Erase, "Eraser", None)}
                        {review_tool_button(
                            snapshot,
                            state,
                            Tool::Select,
                            ReviewToolGlyph::RectangularSelect,
                            "Rect Select",
                            None,
                        )}
                        {review_placeholder_tool_button(
                            state,
                            ReviewToolGlyph::MagicWand,
                            "Magic Wand",
                            "Magic Wand is not implemented yet."
                        )}
                        {review_placeholder_tool_button(
                            state,
                            ReviewToolGlyph::SelectSameTile,
                            "Same Tile",
                            "Select Same Tile is not implemented yet."
                        )}
                    },
                    ReviewToolbarKind::Object => rsx! {
                        {review_tool_button(
                            snapshot,
                            state,
                            Tool::Select,
                            ReviewToolGlyph::SelectObject,
                            "Select Object",
                            None,
                        )}
                        {review_placeholder_tool_button(
                            state,
                            ReviewToolGlyph::InsertTile,
                            "Insert Tile",
                            "Insert Tile Object is not implemented yet."
                        )}
                        {review_placeholder_tool_button(
                            state,
                            ReviewToolGlyph::EditPolygons,
                            "Edit Polygon",
                            "Edit Polygon is not implemented yet."
                        )}
                        {review_tool_button(
                            snapshot,
                            state,
                            Tool::AddRectangle,
                            ReviewToolGlyph::InsertRectangle,
                            "Insert Rect",
                            None,
                        )}
                        {review_tool_button(
                            snapshot,
                            state,
                            Tool::AddPoint,
                            ReviewToolGlyph::InsertPoint,
                            "Insert Point",
                            None,
                        )}
                        {review_placeholder_tool_button(
                            state,
                            ReviewToolGlyph::InsertEllipse,
                            "Insert Ellipse",
                            "Insert Ellipse is not implemented yet."
                        )}
                        {review_placeholder_tool_button(
                            state,
                            ReviewToolGlyph::InsertCapsule,
                            "Insert Capsule",
                            "Insert Capsule is not implemented yet."
                        )}
                        {review_placeholder_tool_button(
                            state,
                            ReviewToolGlyph::InsertPolygon,
                            "Insert Polygon",
                            "Insert Polygon is not implemented yet."
                        )}
                        {review_placeholder_tool_button(
                            state,
                            ReviewToolGlyph::InsertTemplate,
                            "Insert Template",
                            "Insert Template is not implemented yet."
                        )}
                        {review_placeholder_tool_button(
                            state,
                            ReviewToolGlyph::InsertText,
                            "Insert Text",
                            "Insert Text is not implemented yet."
                        )}
                    },
                }
            }
        }
    }
}

fn review_tool_button(
    snapshot: &AppState,
    mut state: Signal<AppState>,
    tool: Tool,
    glyph: ReviewToolGlyph,
    label: &'static str,
    class_override: Option<&'static str>,
) -> Element {
    let class_name = if let Some(class_override) = class_override {
        if snapshot.tool == tool {
            format!("{class_override} active")
        } else {
            class_override.to_string()
        }
    } else if snapshot.tool == tool {
        "review-tool active".to_string()
    } else {
        "review-tool".to_string()
    };

    rsx! {
        button {
            class: "{class_name}",
            onclick: move |_| {
                let mut state = state.write();
                state.tool = tool;
                state.shape_fill_preview = None;
                state.tile_selection_preview = None;
            },
            div { class: "review-tool-icon", {review_tool_icon(&glyph)} }
            span { "{label}" }
        }
    }
}

fn review_placeholder_tool_button(
    mut state: Signal<AppState>,
    glyph: ReviewToolGlyph,
    label: &'static str,
    status: &'static str,
) -> Element {
    rsx! {
        button {
            class: "review-tool placeholder",
            onclick: move |_| state.write().status = status.to_string(),
            div { class: "review-tool-icon", {review_tool_icon(&glyph)} }
            span { "{label}" }
        }
    }
}

#[derive(Clone, Copy)]
enum ReviewToolGlyph {
    Hand,
    StampBrush,
    TerrainBrush,
    Fill,
    ShapeFill,
    Erase,
    RectangularSelect,
    MagicWand,
    SelectSameTile,
    SelectObject,
    InsertTile,
    EditPolygons,
    InsertRectangle,
    InsertPoint,
    InsertEllipse,
    InsertCapsule,
    InsertPolygon,
    InsertTemplate,
    InsertText,
}

fn review_tool_icon(tool: &ReviewToolGlyph) -> Element {
    match tool {
        ReviewToolGlyph::Hand => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M7 11V6.5a1.5 1.5 0 0 1 3 0V11" }
                path { d: "M10 11V5.5a1.5 1.5 0 0 1 3 0V11" }
                path { d: "M13 11V6.5a1.5 1.5 0 0 1 3 0V12" }
                path { d: "M16 12V8.5a1.5 1.5 0 0 1 3 0V13" }
                path { d: "M7 11 5.7 9.8A1.6 1.6 0 0 0 3 11v.5c0 1.8.6 3.5 1.8 4.8l1.9 2.2c.8.9 2 1.5 3.2 1.5H14c1.7 0 3.2-.8 4.2-2.1l1.2-1.8c.4-.6.6-1.3.6-2V13" }
            }
        },
        ReviewToolGlyph::StampBrush => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M9 6.4c0-1.3 1.2-2.4 3-2.4s3 1.1 3 2.4c0 1-.7 1.8-1.7 2.2v1.2H10.7V8.6C9.7 8.2 9 7.4 9 6.4z" }
                path { d: "M8.4 11.2h7.2" }
                path { d: "M7.2 13.2h9.6" }
                path { d: "M8.6 15.2h6.8" }
                path { d: "M6 18.3h12" }
                path { d: "M7.6 18.3v1.7" }
                path { d: "M12 18.3v1.7" }
                path { d: "M16.4 18.3v1.7" }
            }
        },
        ReviewToolGlyph::TerrainBrush => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "4.5", y: "12.5", width: "4", height: "4", rx: "0.6" }
                rect { x: "9.8", y: "12.5", width: "4", height: "4", rx: "0.6" }
                rect { x: "4.5", y: "17.3", width: "4", height: "4", rx: "0.6" }
                path { d: "M13.5 5.2 18.8 10.5" }
                path { d: "M11.6 7.1 16.9 12.4" }
                path { d: "M10.2 18.2c1.2-.1 2.2-.6 3-1.4l5.2-5.2-3.8-3.8-5.2 5.2c-.8.8-1.3 1.9-1.4 3z" }
            }
        },
        ReviewToolGlyph::Fill => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M7.5 10.5 12.2 5.8l5.4 5.4-4.7 4.7a2.1 2.1 0 0 1-3 0z" }
                path { d: "M12.2 5.8 17.6 11.2" }
                path { d: "M5.4 17.6h8.6" }
                path { d: "M16.7 18.7c0 1-.8 1.8-1.8 1.8s-1.8-.8-1.8-1.8c0-.9.8-1.8 1.8-3.2 1 1.4 1.8 2.3 1.8 3.2z" }
            }
        },
        ReviewToolGlyph::ShapeFill => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "m8 10 5-5 5 5-4.2 4.2H12" }
                path { d: "M13 5 18 10" }
                rect { x: "5", y: "13.2", width: "14", height: "5.8", rx: "1.4", stroke_dasharray: "2.5 2" }
            }
        },
        ReviewToolGlyph::Erase => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "m7 14 6-6 5 5-4 4H10z" }
                path { d: "M12 9l5 5" }
                path { d: "M4 19.5h15" }
            }
        },
        ReviewToolGlyph::RectangularSelect => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "5", y: "6", width: "14", height: "12", rx: "2", stroke_dasharray: "3 2.2" }
            }
        },
        ReviewToolGlyph::MagicWand => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "m6 19 8.8-8.8" }
                path { d: "M14.4 5.2v3.3" }
                path { d: "M12.8 6.8h3.3" }
                path { d: "M18.6 9.2v2.4" }
                path { d: "M17.4 10.4h2.4" }
                path { d: "M8.4 4v2.6" }
                path { d: "M7.1 5.3h2.6" }
            }
        },
        ReviewToolGlyph::SelectSameTile => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "4.5", y: "5", width: "5", height: "5", rx: "0.8" }
                rect { x: "14.5", y: "5", width: "5", height: "5", rx: "0.8" }
                rect { x: "4.5", y: "14", width: "5", height: "5", rx: "0.8" }
                rect { x: "14.5", y: "14", width: "5", height: "5", rx: "0.8" }
                path { d: "M9.5 7.5h5" }
                path { d: "m12.5 4.8 2.7 2.7-2.7 2.7" }
            }
        },
        ReviewToolGlyph::SelectObject => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M6 4.5 16 12 11.5 12.8 13.8 18.5 11.3 19.5 9 13.8 6 16z" }
            }
        },
        ReviewToolGlyph::InsertTile => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "5", y: "6", width: "8", height: "8", rx: "1" }
                path { d: "M16.5 8.2v7.6" }
                path { d: "M12.7 12h7.6" }
            }
        },
        ReviewToolGlyph::EditPolygons => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M6 7.5 10 5l5 3 3 5-3.8 5.2-6 .8L4.8 14z" }
                circle { cx: "6", cy: "7.5", r: "1" }
                circle { cx: "15", cy: "8", r: "1" }
                circle { cx: "18", cy: "13", r: "1" }
                circle { cx: "8.2", cy: "19", r: "1" }
            }
        },
        ReviewToolGlyph::InsertRectangle => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "5", y: "7", width: "14", height: "10", rx: "1.8" }
                path { d: "M12 5v4" }
                path { d: "M10 7h4" }
            }
        },
        ReviewToolGlyph::InsertPoint => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M12 5v14" }
                path { d: "M5 12h14" }
                circle { cx: "12", cy: "12", r: "2.5" }
            }
        },
        ReviewToolGlyph::InsertEllipse => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                ellipse { cx: "12", cy: "12", rx: "7", ry: "4.8" }
                path { d: "M12 5v3.4" }
            }
        },
        ReviewToolGlyph::InsertCapsule => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "6", y: "5", width: "12", height: "14", rx: "6" }
            }
        },
        ReviewToolGlyph::InsertPolygon => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M6 8 10 5l6 2.5 2 5-4.2 5.5-6.6-.7L4.8 12z" }
            }
        },
        ReviewToolGlyph::InsertTemplate => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect { x: "6", y: "4.5", width: "12", height: "15", rx: "1.6" }
                path { d: "M9 9h6" }
                path { d: "M9 12h6" }
                path { d: "M9 15h4" }
            }
        },
        ReviewToolGlyph::InsertText => rsx! {
            svg {
                class: "review-tool-icon-svg",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.9",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M6 6h12" }
                path { d: "M12 6v12" }
                path { d: "M8.5 18h7" }
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

fn editor_grid_style(snapshot: &AppState, session: &EditorSession) -> String {
    const REVIEW_GRID_LINE_WIDTH: f32 = 0.5;

    let map = &session.document().map;
    let zoom = snapshot.zoom_percent as f32 / 100.0;
    let grid_width = (map.tile_width as f32 * zoom).max(1.0);
    let grid_height = (map.tile_height as f32 * zoom).max(1.0);
    let line_phase_offset = -(REVIEW_GRID_LINE_WIDTH * 0.5);
    let offset_x = snapshot.pan_x as f32 + line_phase_offset;
    let offset_y = snapshot.pan_y as f32 + 10.0 + line_phase_offset;

    format!(
        "--grid-size-x:{grid_width}px;--grid-size-y:{grid_height}px;--grid-line-width:{REVIEW_GRID_LINE_WIDTH}px;--grid-offset-x:{offset_x}px;--grid-offset-y:{offset_y}px;"
    )
}

fn tileset_sheet_style(document: &taled_core::EditorDocument, selected_gid: u32) -> String {
    let columns = document
        .map
        .tile_reference_for_gid(selected_gid)
        .map(|reference| reference.tileset.tileset.columns.max(1))
        .or_else(|| {
            document
                .map
                .tilesets
                .first()
                .map(|tileset| tileset.tileset.columns.max(1))
        })
        .unwrap_or(1);

    format!("grid-template-columns:repeat({columns}, minmax(0, 1fr));")
}

fn layer_kind_label(layer: &Layer) -> &'static str {
    if layer.as_tile().is_some() {
        "Tile Layer"
    } else {
        "Object Layer"
    }
}

fn active_toolbar_kind(session: &EditorSession, layer_index: usize) -> ReviewToolbarKind {
    match session.document().map.layer(layer_index) {
        Some(layer) if layer.as_object().is_some() => ReviewToolbarKind::Object,
        _ => ReviewToolbarKind::Tile,
    }
}

fn toolbar_supports_tool(kind: ReviewToolbarKind, tool: Tool) -> bool {
    match kind {
        ReviewToolbarKind::Tile => matches!(
            tool,
            Tool::Hand | Tool::Paint | Tool::Fill | Tool::ShapeFill | Tool::Erase | Tool::Select
        ),
        ReviewToolbarKind::Object => {
            matches!(tool, Tool::Select | Tool::AddRectangle | Tool::AddPoint)
        }
    }
}

fn set_review_active_layer_kind(state: &mut AppState, layer_index: usize, kind: ReviewToolbarKind) {
    state.active_layer = layer_index;
    state.selected_object = None;
    state.shape_fill_preview = None;
    state.tile_selection = None;
    state.tile_selection_preview = None;
    if !toolbar_supports_tool(kind, state.tool) {
        state.tool = match kind {
            ReviewToolbarKind::Tile => Tool::Paint,
            ReviewToolbarKind::Object => Tool::Select,
        };
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
    let Some(session) = state.session.as_ref() else {
        state.status = "Load a map first.".to_string();
        return;
    };
    let object_layer_index = session
        .document()
        .map
        .layers
        .iter()
        .enumerate()
        .find(|(_, layer)| layer.as_object().is_some())
        .map(|(index, _)| index);

    let Some(object_layer_index) = object_layer_index else {
        state.status = "No object layer is available yet.".to_string();
        return;
    };

    set_review_active_layer_kind(state, object_layer_index, ReviewToolbarKind::Object);
    create_object(state, shape);
}
