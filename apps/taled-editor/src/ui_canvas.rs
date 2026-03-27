use std::{collections::BTreeMap, time::Duration};

use base64::Engine;
use dioxus::prelude::*;
use taled_core::{EditorDocument, ObjectShape};

use crate::{
    app_state::{
        AppState, TileSelectionRegion, Tool, is_tile_selection_tool, selection_bounds,
        selection_cells_are_rectangular, selection_cells_from_region, selection_region_from_cells,
        shape_fill_cells,
    },
    edit_ops::{apply_cell_tool, clear_tile_selection_immediately},
    platform::log,
    touch_ops::{
        cell_from_surface, handle_touch_pointer_cancel, handle_touch_pointer_down,
        handle_touch_pointer_move, handle_touch_pointer_up, should_ignore_synthetic_click,
    },
    ui_visuals::object_overlay_style,
};

const TILE_SELECTION_FADE_DURATION: Duration = Duration::from_millis(170);

pub(crate) fn render_canvas(snapshot: &AppState, mut state: Signal<AppState>) -> Element {
    let Some(session) = snapshot.session.as_ref() else {
        return rsx! {
            div { class: "canvas-host",
                div { class: "empty-state", "No map loaded yet." }
            }
        };
    };

    let document = session.document();
    let map = &document.map;
    let zoom = snapshot.zoom_percent as f32 / 100.0;
    let canvas_style = format!(
        "width:{}px;height:{}px;transform:translate3d({}px, {}px, 0) scale({zoom});",
        map.total_pixel_width(),
        map.total_pixel_height(),
        snapshot.pan_x,
        snapshot.pan_y
    );
    let canvas_class = if snapshot.camera_transition_active {
        "canvas camera-transition"
    } else {
        "canvas"
    };
    let shape_fill_preview = if snapshot.tool == Tool::ShapeFill {
        snapshot
            .shape_fill_preview
            .map(|preview| build_shape_fill_preview(document, snapshot, preview))
    } else {
        None
    };
    let tile_selection_overlay = active_tile_selection_overlay(document, snapshot);
    let has_tile_selection_overlay = tile_selection_overlay.is_some();
    let tile_selection_transfer_preview =
        active_tile_selection_transfer_preview(document, snapshot);
    let has_flat_tile_layers = snapshot.flat_tile_layers_data_url.is_some();
    let live_active_tile_styles = if has_flat_tile_layers {
        Vec::new()
    } else {
        collect_visible_tile_styles(document, snapshot)
    };

    rsx! {
        div {
            class: "canvas-host",
            onmounted: move |event| {
                let mut state = state;
                async move {
                    if let Ok(rect) = event.get_client_rect().await {
                        log(format!(
                            "touch:host-rect origin=({:.1},{:.1}) size=({:.1},{:.1})",
                            rect.origin.x,
                            rect.origin.y,
                            rect.size.width,
                            rect.size.height,
                        ));
                        let mut state = state.write();
                        state.canvas_stage_client_origin = Some((rect.origin.x, rect.origin.y));
                        state.canvas_host_size = Some((rect.size.width, rect.size.height));
                        center_canvas_if_needed(&mut state, rect.size.width, rect.size.height);
                    }
                    if let Ok(scroll) = event.get_scroll_offset().await {
                        log(format!(
                            "touch:host-scroll offset=({:.1},{:.1})",
                            scroll.x, scroll.y,
                        ));
                        state.write().canvas_host_scroll_offset = (scroll.x, scroll.y);
                    }
                }
            },
            onscroll: move |event| {
                let mut state = state.write();
                let scroll_left = event.scroll_left();
                let scroll_top = event.scroll_top();
                log(format!(
                    "touch:host-scroll offset=({scroll_left:.1},{scroll_top:.1}) size=({},{}) client=({},{})",
                    event.scroll_width(),
                    event.scroll_height(),
                    event.client_width(),
                    event.client_height(),
                ));
                state.canvas_host_scroll_offset = (scroll_left, scroll_top);
            },
            div {
                    class: "canvas-stage",
                onmounted: move |event| {
                    let mut state = state;
                    async move {
                        if let Ok(rect) = event.get_client_rect().await {
                            log(format!(
                                "touch:stage-rect origin=({:.1},{:.1}) size=({:.1},{:.1})",
                                rect.origin.x,
                                rect.origin.y,
                                rect.size.width,
                                rect.size.height,
                            ));
                            center_canvas_if_needed(&mut state.write(), rect.size.width, rect.size.height);
                        }
                    }
                },
                onclick: move |event| {
                    let mut state = state.write();
                    if should_ignore_synthetic_click(&mut state) {
                        return;
                    }
                    handle_canvas_click(
                        &mut state,
                        event.data().element_coordinates().x,
                        event.data().element_coordinates().y,
                    );
                },
                onpointerdown: move |event| handle_touch_pointer_down(&mut state.write(), event),
                onpointermove: move |event| handle_touch_pointer_move(&mut state.write(), event),
                onpointerup: move |event| handle_touch_pointer_up(&mut state.write(), event),
                onpointercancel: move |event| handle_touch_pointer_cancel(&mut state.write(), event),
                div {
                    class: canvas_class,
                    style: canvas_style,
                    ontransitionend: move |_| state.write().camera_transition_active = false,
                    if let (Some(data_url), Some(style)) = (
                        snapshot.flat_tile_layers_data_url.as_ref(),
                        flat_tile_layers_style(snapshot, map),
                    ) {
                        img {
                            class: "canvas-flat-layer",
                            src: "{data_url}",
                            alt: "",
                            style: "{style}",
                        }
                    }
                    for (x, y, style) in live_active_tile_styles.iter() {
                        div {
                            key: "tile-{x}-{y}",
                            class: "tile-sprite",
                            style: "{style}",
                        }
                    }

                    for (layer_index, layer) in map.layers.iter().enumerate() {
                        if let Some(object_layer) = layer.as_object() {
                            if object_layer.visible {
                                for object in &object_layer.objects {
                                    div {
                                        key: "object-{layer_index}-{object.id}",
                                        class: object_class(snapshot.selected_object, object.id, &object.shape),
                                        style: object_overlay_style(
                                            object,
                                            snapshot.tool == Tool::Select,
                                            snapshot.selected_object == Some(object.id),
                                            zoom,
                                        ),
                                        onclick: {
                                            let object_id = object.id;
                                            move |_| {
                                                let mut state = state.write();
                                                if should_ignore_synthetic_click(&mut state) {
                                                    return;
                                                }
                                                if state.tool != Tool::Select {
                                                    state.status = "Switch to Select before choosing objects.".to_string();
                                                    return;
                                                }
                                                state.active_layer = layer_index;
                                                state.selected_object = Some(object_id);
                                                state.tile_selection = None;
                                                state.tile_selection_cells = None;
                                                state.tile_selection_preview = None;
                                                state.tile_selection_preview_cells = None;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    {shape_fill_preview.as_ref().map(|preview| rsx! {
                        for tile in &preview.tiles {
                            div {
                                key: "shape-fill-preview-{tile.x}-{tile.y}",
                                class: if tile.fallback {
                                    "shape-fill-preview-tile fallback"
                                } else {
                                    "tile-preview shape-fill-preview-tile"
                                },
                                style: "{tile.style}",
                            }
                        }
                        div {
                            class: "shape-fill-preview-frame",
                            style: "{preview.frame_style}",
                        }
                    })}

                    {tile_selection_overlay.as_ref().map(|overlay| rsx! {
                        if overlay.irregular {
                            Fragment {
                                div {
                                    class: if overlay.closing {
                                        "tile-selection-region-cells closing"
                                    } else if overlay.preview {
                                        "tile-selection-region-cells preview"
                                    } else {
                                        "tile-selection-region-cells"
                                    },
                                    for (index, cell_style) in overlay.cell_styles.iter().enumerate() {
                                        div {
                                            key: "tile-selection-cell-{index}",
                                            class: "tile-selection-cell-fragment",
                                            style: "{cell_style}",
                                        }
                                    }
                                }
                                div {
                                    class: if overlay.closing {
                                        "tile-selection-irregular-bounds closing"
                                    } else if overlay.preview {
                                        "tile-selection-irregular-bounds preview"
                                    } else {
                                        "tile-selection-irregular-bounds"
                                    },
                                    style: "{overlay.region_style}",
                                    if overlay.show_irregular_handles {
                                        for handle in &overlay.handles {
                                            div {
                                                key: "tile-selection-irregular-handle-{handle.position}",
                                                class: "tile-selection-handle ghost {handle.position}",
                                                style: "{handle.style}",
                                                div { class: "tile-selection-handle-dot ghost" }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            div {
                                class: if overlay.closing {
                                    "tile-selection-region closing"
                                } else if overlay.preview {
                                    "tile-selection-region preview"
                                } else {
                                    "tile-selection-region"
                                },
                                style: "{overlay.region_style}",
                                div { class: "tile-selection-frame" }
                                if overlay.show_handles {
                                    for handle in &overlay.handles {
                                        div {
                                            key: "tile-selection-handle-{handle.position}",
                                            class: "tile-selection-handle {handle.position}",
                                            style: "{handle.style}",
                                            div { class: "tile-selection-handle-dot" }
                                        }
                                    }
                                }
                            }
                        }
                    })}

                    {tile_selection_transfer_preview.as_ref().map(|preview| rsx! {
                        for tile in &preview.tiles {
                            div {
                                key: "tile-selection-transfer-preview-{tile.x}-{tile.y}",
                                class: if tile.fallback {
                                    "shape-fill-preview-tile tile-selection-transfer-preview-tile fallback"
                                } else {
                                    "tile-preview shape-fill-preview-tile tile-selection-transfer-preview-tile"
                                },
                                style: "{tile.style}",
                            }
                        }
                    })}

                    if let Some((selected_x, selected_y)) =
                        (!has_tile_selection_overlay).then_some(snapshot.selected_cell).flatten()
                    {
                        div {
                            key: "selected-cell-{selected_x}-{selected_y}",
                            class: "cell-hitbox selected",
                            style: cell_style(map.tile_width, map.tile_height, selected_x, selected_y),
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct VisibleCellBounds {
    min_x: u32,
    max_x: u32,
    min_y: u32,
    max_y: u32,
}

fn visible_cell_bounds(snapshot: &AppState, map: &taled_core::Map) -> VisibleCellBounds {
    let (host_width, host_height) = snapshot.canvas_host_size.unwrap_or((384.0, 500.0));
    let zoom = (f64::from(snapshot.zoom_percent) / 100.0).max(0.01);
    let tile_width = f64::from(map.tile_width.max(1));
    let tile_height = f64::from(map.tile_height.max(1));
    let margin_x = tile_width;
    let margin_y = tile_height;

    let world_left = (-f64::from(snapshot.pan_x) - margin_x * zoom) / zoom;
    let world_top = (-f64::from(snapshot.pan_y) - margin_y * zoom) / zoom;
    let world_right = (host_width - f64::from(snapshot.pan_x) + margin_x * zoom) / zoom;
    let world_bottom = (host_height - f64::from(snapshot.pan_y) + margin_y * zoom) / zoom;

    let min_x = (world_left / tile_width).floor().max(0.0) as u32;
    let min_y = (world_top / tile_height).floor().max(0.0) as u32;
    let max_x = (world_right / tile_width).ceil().max(0.0) as u32;
    let max_y = (world_bottom / tile_height).ceil().max(0.0) as u32;

    VisibleCellBounds {
        min_x: min_x.min(map.width),
        max_x: max_x.max(min_x + 1).min(map.width),
        min_y: min_y.min(map.height),
        max_y: max_y.max(min_y + 1).min(map.height),
    }
}

fn expanded_visible_cell_bounds(snapshot: &AppState, map: &taled_core::Map) -> VisibleCellBounds {
    const CACHE_MARGIN_TILES: u32 = 4;

    let visible = visible_cell_bounds(snapshot, map);
    VisibleCellBounds {
        min_x: visible.min_x.saturating_sub(CACHE_MARGIN_TILES),
        max_x: visible.max_x.saturating_add(CACHE_MARGIN_TILES).min(map.width),
        min_y: visible.min_y.saturating_sub(CACHE_MARGIN_TILES),
        max_y: visible.max_y.saturating_add(CACHE_MARGIN_TILES).min(map.height),
    }
}

fn full_map_cell_bounds(map: &taled_core::Map) -> VisibleCellBounds {
    VisibleCellBounds {
        min_x: 0,
        max_x: map.width,
        min_y: 0,
        max_y: map.height,
    }
}

fn visible_painted_tile_count(map: &taled_core::Map) -> usize {
    map.layers
        .iter()
        .filter_map(|layer| layer.as_tile())
        .filter(|layer| layer.visible)
        .map(|layer| {
            (0..layer.height)
                .flat_map(|y| (0..layer.width).map(move |x| (x, y)))
                .filter(|(x, y)| layer.tile_at(*x, *y).is_some_and(|gid| gid != 0))
                .count()
        })
        .sum()
}

fn prefers_full_flat_tile_cache(map: &taled_core::Map) -> bool {
    const MAX_FULL_CACHE_AXIS_PX: u32 = 4_096;
    const MAX_FULL_CACHE_PAINTED_TILES: usize = 12_000;

    map.total_pixel_width() <= MAX_FULL_CACHE_AXIS_PX
        && map.total_pixel_height() <= MAX_FULL_CACHE_AXIS_PX
        && visible_painted_tile_count(map) <= MAX_FULL_CACHE_PAINTED_TILES
}

fn flat_tile_cache_bounds(snapshot: &AppState, map: &taled_core::Map) -> VisibleCellBounds {
    if prefers_full_flat_tile_cache(map) {
        full_map_cell_bounds(map)
    } else {
        expanded_visible_cell_bounds(snapshot, map)
    }
}

#[cfg(target_arch = "wasm32")]
fn perf_now_ms() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
fn perf_now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() * 1_000.0)
        .unwrap_or_default()
}

fn flat_tile_layers_style(snapshot: &AppState, map: &taled_core::Map) -> Option<String> {
    let (min_x, max_x, min_y, max_y) = snapshot.flat_tile_layers_cell_bounds?;
    Some(format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        min_x * map.tile_width,
        min_y * map.tile_height,
        (max_x.saturating_sub(min_x)).max(1) * map.tile_width,
        (max_y.saturating_sub(min_y)).max(1) * map.tile_height,
    ))
}

pub(crate) fn rebuild_flat_tile_layer_cache(state: &mut AppState) {
    let Some(session) = state.session.as_ref() else {
        state.flat_tile_layers_data_url = None;
        state.flat_tile_layers_cell_bounds = None;
        return;
    };

    let started_at_ms = perf_now_ms();
    let document = session.document();
    let map = &document.map;
    let cache_bounds = flat_tile_cache_bounds(state, map);
    let strategy = if cache_bounds.min_x == 0
        && cache_bounds.min_y == 0
        && cache_bounds.max_x == map.width
        && cache_bounds.max_y == map.height
    {
        "full-map"
    } else {
        "slice"
    };

    let Some(svg) = build_flat_tile_layer_svg(map, &state.image_cache, cache_bounds) else {
        state.flat_tile_layers_data_url = None;
        state.flat_tile_layers_cell_bounds = None;
        return;
    };
    let svg_bytes = svg.len();
    let encoded = base64::engine::general_purpose::STANDARD.encode(svg);

    state.flat_tile_layers_data_url = Some(format!("data:image/svg+xml;base64,{encoded}"));
    state.flat_tile_layers_cell_bounds = Some((
        cache_bounds.min_x,
        cache_bounds.max_x,
        cache_bounds.min_y,
        cache_bounds.max_y,
    ));
    log(format!(
        "perf: flat-cache rebuilt strategy={strategy} format=svg bounds=({}, {})..({}, {}) cache_bytes={} duration_ms={:.1}",
        cache_bounds.min_x,
        cache_bounds.min_y,
        cache_bounds.max_x,
        cache_bounds.max_y,
        svg_bytes,
        perf_now_ms() - started_at_ms,
    ));
}

fn build_flat_tile_layer_svg(
    map: &taled_core::Map,
    image_cache: &BTreeMap<usize, String>,
    cache_bounds: VisibleCellBounds,
) -> Option<String> {
    let slice_width = (cache_bounds.max_x.saturating_sub(cache_bounds.min_x)).max(1) * map.tile_width;
    let slice_height =
        (cache_bounds.max_y.saturating_sub(cache_bounds.min_y)).max(1) * map.tile_height;
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" shape-rendering=\"crispEdges\">",
        slice_width,
        slice_height,
        slice_width,
        slice_height
    );
    let mut defs = BTreeMap::new();
    let mut body = String::new();
    let mut wrote_any = false;

    for layer in &map.layers {
        let Some(tile_layer) = layer.as_tile() else {
            continue;
        };
        if !tile_layer.visible {
            continue;
        }
        if let Some(layer_svg) =
            flat_tile_layer_slice_svg(map, image_cache, tile_layer, cache_bounds, &mut defs)
        {
            body.push_str(&layer_svg);
            wrote_any = true;
        }
    }

    if !wrote_any {
        return None;
    }

    if !defs.is_empty() {
        svg.push_str("<defs>");
        for symbol in defs.values() {
            svg.push_str(symbol);
        }
        svg.push_str("</defs>");
    }
    svg.push_str(&body);
    svg.push_str("</svg>");
    Some(svg)
}

fn flat_tile_symbol_svg(
    map: &taled_core::Map,
    image_cache: &BTreeMap<usize, String>,
    gid: u32,
) -> Option<String> {
    let tile = map.tile_reference_for_gid(gid)?;
    let image = image_cache.get(&tile.tileset_index)?;
    let columns = tile.tileset.tileset.columns.max(1);
    let tile_width = tile.tileset.tileset.tile_width;
    let tile_height = tile.tileset.tileset.tile_height;
    let source_x = (tile.local_id % columns) * tile_width;
    let source_y = (tile.local_id / columns) * tile_height;

    Some(format!(
        "<symbol id=\"tile-{gid}\" viewBox=\"{} {} {} {}\" preserveAspectRatio=\"none\"><image href=\"{}\" width=\"{}\" height=\"{}\"/></symbol>",
        source_x,
        source_y,
        tile_width,
        tile_height,
        image,
        tile.tileset.tileset.image.width,
        tile.tileset.tileset.image.height,
    ))
}

fn flat_tile_layer_tile_svg(
    map: &taled_core::Map,
    image_cache: &BTreeMap<usize, String>,
    tile_layer: &taled_core::TileLayer,
    x: u32,
    y: u32,
    cache_bounds: VisibleCellBounds,
    defs: &mut BTreeMap<u32, String>,
) -> Option<String> {
    let gid = tile_layer.tile_at(x, y).filter(|gid| *gid != 0)?;
    if let std::collections::btree_map::Entry::Vacant(entry) = defs.entry(gid) {
        entry.insert(flat_tile_symbol_svg(map, image_cache, gid)?);
    }

    Some(format!(
        "<use href=\"#tile-{gid}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
        (x - cache_bounds.min_x) * map.tile_width,
        (y - cache_bounds.min_y) * map.tile_height,
        map.tile_width,
        map.tile_height,
    ))
}

fn collect_visible_tile_styles(
    document: &EditorDocument,
    snapshot: &AppState,
) -> Vec<(u32, u32, String)> {
    let visible_bounds = visible_cell_bounds(snapshot, &document.map);
    document
        .map
        .layer(snapshot.active_layer)
        .and_then(|layer| layer.as_tile())
        .filter(|layer| layer.visible)
        .map(|tile_layer| {
            let max_y = visible_bounds.max_y.min(tile_layer.height);
            let max_x = visible_bounds.max_x.min(tile_layer.width);
            (visible_bounds.min_y..max_y)
                .flat_map(|y| (visible_bounds.min_x..max_x).map(move |x| (x, y)))
                .filter_map(|(x, y)| {
                    let gid = tile_layer.tile_at(x, y).filter(|gid| *gid != 0)?;
                    let style = sprite_style(document, &snapshot.image_cache, gid, x, y)?;
                    Some((x, y, style))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn flat_tile_layer_slice_svg(
    map: &taled_core::Map,
    image_cache: &BTreeMap<usize, String>,
    tile_layer: &taled_core::TileLayer,
    cache_bounds: VisibleCellBounds,
    defs: &mut BTreeMap<u32, String>,
) -> Option<String> {
    let mut layer_svg = String::new();

    for y in cache_bounds.min_y..cache_bounds.max_y.min(tile_layer.height) {
        for x in cache_bounds.min_x..cache_bounds.max_x.min(tile_layer.width) {
            if let Some(tile_svg) =
                flat_tile_layer_tile_svg(map, image_cache, tile_layer, x, y, cache_bounds, defs)
            {
                layer_svg.push_str(&tile_svg);
            }
        }
    }

    (!layer_svg.is_empty()).then_some(layer_svg)
}

pub(crate) fn refresh_flat_tile_layer_cache_if_needed(state: &mut AppState) {
    let Some(session) = state.session.as_ref() else {
        state.flat_tile_layers_data_url = None;
        state.flat_tile_layers_cell_bounds = None;
        return;
    };

    let visible = visible_cell_bounds(state, &session.document().map);
    let Some((cache_min_x, cache_max_x, cache_min_y, cache_max_y)) =
        state.flat_tile_layers_cell_bounds
    else {
        rebuild_flat_tile_layer_cache(state);
        return;
    };

    let map = &session.document().map;
    if cache_min_x == 0 && cache_min_y == 0 && cache_max_x == map.width && cache_max_y == map.height {
        return;
    }

    let fits_horizontally = visible.min_x >= cache_min_x && visible.max_x <= cache_max_x;
    let fits_vertically = visible.min_y >= cache_min_y && visible.max_y <= cache_max_y;
    if fits_horizontally && fits_vertically {
        return;
    }

    rebuild_flat_tile_layer_cache(state);
}

fn center_canvas_if_needed(state: &mut AppState, host_width: f64, host_height: f64) {
    if !state.pending_canvas_center || host_width <= 0.0 || host_height <= 0.0 {
        return;
    }

    let Some(session) = state.session.as_ref() else {
        return;
    };

    let map = &session.document().map;
    let zoom = f64::from(state.zoom_percent) / 100.0;
    let map_width = f64::from(map.total_pixel_width()) * zoom;
    let map_height = f64::from(map.total_pixel_height()) * zoom;

    state.pan_x = ((host_width - map_width) * 0.5).round() as i32;
    state.pan_y = ((host_height - map_height) * 0.5).round() as i32;
    state.pending_canvas_center = false;
    rebuild_flat_tile_layer_cache(state);
    log(format!(
        "touch:center-map host=({host_width:.1},{host_height:.1}) map=({map_width:.1},{map_height:.1}) pan=({}, {}) zoom={}",
        state.pan_x, state.pan_y, state.zoom_percent,
    ));
}

fn sprite_style(
    document: &EditorDocument,
    image_cache: &BTreeMap<usize, String>,
    gid: u32,
    x: u32,
    y: u32,
) -> Option<String> {
    let tile = document.map.tile_reference_for_gid(gid)?;
    let image = image_cache.get(&tile.tileset_index)?;
    let columns = tile.tileset.tileset.columns.max(1);
    let tile_width = tile.tileset.tileset.tile_width;
    let tile_height = tile.tileset.tileset.tile_height;
    let source_x = (tile.local_id % columns) * tile_width;
    let source_y = (tile.local_id / columns) * tile_height;

    Some(format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;background-image:url('{image}');background-position:-{}px -{}px;background-size:{}px {}px;",
        x * document.map.tile_width,
        y * document.map.tile_height,
        document.map.tile_width,
        document.map.tile_height,
        source_x,
        source_y,
        tile.tileset.tileset.image.width,
        tile.tileset.tileset.image.height,
    ))
}

fn cell_style(tile_width: u32, tile_height: u32, x: u32, y: u32) -> String {
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        x * tile_width,
        y * tile_height,
        tile_width,
        tile_height
    )
}

fn signed_cell_style(tile_width: u32, tile_height: u32, x: i32, y: i32) -> String {
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        x * tile_width as i32,
        y * tile_height as i32,
        tile_width,
        tile_height
    )
}

fn preview_tile_style(
    document: &EditorDocument,
    image_cache: &BTreeMap<usize, String>,
    gid: u32,
    x: u32,
    y: u32,
) -> Option<String> {
    let mut style = sprite_style(document, image_cache, gid, x, y)?;
    style.push_str("opacity:0.46;filter:saturate(0.92);");
    Some(style)
}

fn preview_tile_style_signed(
    document: &EditorDocument,
    image_cache: &BTreeMap<usize, String>,
    gid: u32,
    x: i32,
    y: i32,
) -> Option<String> {
    let tile = document.map.tile_reference_for_gid(gid)?;
    let image = image_cache.get(&tile.tileset_index)?;
    let columns = tile.tileset.tileset.columns.max(1);
    let tile_width = tile.tileset.tileset.tile_width;
    let tile_height = tile.tileset.tileset.tile_height;
    let source_x = (tile.local_id % columns) * tile_width;
    let source_y = (tile.local_id / columns) * tile_height;

    Some(format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;background-image:url('{image}');background-position:-{}px -{}px;background-size:{}px {}px;opacity:0.46;filter:saturate(0.92);",
        x * document.map.tile_width as i32,
        y * document.map.tile_height as i32,
        document.map.tile_width,
        document.map.tile_height,
        source_x,
        source_y,
        tile.tileset.tileset.image.width,
        tile.tileset.tileset.image.height,
    ))
}

fn build_shape_fill_preview(
    document: &EditorDocument,
    snapshot: &AppState,
    preview: crate::app_state::ShapeFillPreview,
) -> ShapeFillPreviewVisual {
    let (min_x, min_y, max_x, max_y) = preview_bounds(preview);
    let mut tiles = Vec::new();
    let preview_cells = shape_fill_cells(
        snapshot.shape_fill_mode,
        preview.start_cell.0,
        preview.start_cell.1,
        preview.end_cell.0,
        preview.end_cell.1,
    );

    for (x, y) in preview_cells {
        let style = preview_tile_style(document, &snapshot.image_cache, snapshot.selected_gid, x, y)
            .unwrap_or_else(|| cell_style(document.map.tile_width, document.map.tile_height, x, y));
        tiles.push(ShapeFillPreviewTile {
            x: x as i32,
            y: y as i32,
            style,
            fallback: document
                .map
                .tile_reference_for_gid(snapshot.selected_gid)
                .is_none(),
        });
    }

    ShapeFillPreviewVisual {
        tiles,
        frame_style: preview_frame_style(
            document.map.tile_width,
            document.map.tile_height,
            min_x,
            min_y,
            max_x,
            max_y,
        ),
    }
}

fn preview_bounds(preview: crate::app_state::ShapeFillPreview) -> (u32, u32, u32, u32) {
    (
        preview.start_cell.0.min(preview.end_cell.0),
        preview.start_cell.1.min(preview.end_cell.1),
        preview.start_cell.0.max(preview.end_cell.0),
        preview.start_cell.1.max(preview.end_cell.1),
    )
}

fn preview_frame_style(
    tile_width: u32,
    tile_height: u32,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
) -> String {
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        min_x * tile_width,
        min_y * tile_height,
        (max_x - min_x + 1) * tile_width,
        (max_y - min_y + 1) * tile_height,
    )
}

fn object_class(selected: Option<u32>, object_id: u32, shape: &ObjectShape) -> &'static str {
    match (selected == Some(object_id), shape) {
        (true, ObjectShape::Rectangle) => "object-overlay rectangle selected",
        (true, ObjectShape::Point) => "object-overlay point selected",
        (false, ObjectShape::Rectangle) => "object-overlay rectangle",
        (false, ObjectShape::Point) => "object-overlay point",
    }
}

struct ShapeFillPreviewVisual {
    tiles: Vec<ShapeFillPreviewTile>,
    frame_style: String,
}

struct TileSelectionTransferPreviewVisual {
    tiles: Vec<ShapeFillPreviewTile>,
}

struct ShapeFillPreviewTile {
    x: i32,
    y: i32,
    style: String,
    fallback: bool,
}

fn active_tile_selection_overlay(
    document: &EditorDocument,
    snapshot: &AppState,
) -> Option<TileSelectionOverlayVisual> {
    if !is_tile_selection_tool(snapshot.tool) {
        return None;
    }
    let active_layer = document.map.layer(snapshot.active_layer)?;
    active_layer.as_tile()?;

    let closing_region = snapshot.tile_selection_closing;
    let (selection, selection_cells, preview, closing) =
        if let Some(preview_cells) = snapshot.tile_selection_preview_cells.clone() {
            let selection = selection_region_from_cells(&preview_cells)?;
            (selection, preview_cells, true, false)
        } else if let Some(selection) = snapshot.tile_selection_preview {
            (
                selection,
                selection_cells_from_region(selection),
                true,
                false,
            )
        } else if let (Some(selection), Some(selection_cells)) = (
            snapshot.tile_selection,
            snapshot.tile_selection_cells.clone(),
        ) {
            (selection, selection_cells, false, false)
        } else if snapshot
            .tile_selection_closing_started_at
            .is_some_and(|started_at| started_at.elapsed() <= TILE_SELECTION_FADE_DURATION)
        {
            let selection = closing_region?;
            (
                selection,
                snapshot
                    .tile_selection_closing_cells
                    .clone()
                    .unwrap_or_else(|| selection_cells_from_region(selection)),
                false,
                true,
            )
        } else {
            return None;
        };
    Some(build_tile_selection_overlay(
        document,
        selection,
        selection_cells,
        preview,
        closing,
        snapshot.tile_selection_transfer.is_some(),
    ))
}

fn build_tile_selection_overlay(
    document: &EditorDocument,
    selection: TileSelectionRegion,
    selection_cells: std::collections::BTreeSet<(i32, i32)>,
    preview: bool,
    closing: bool,
    transfer_active: bool,
) -> TileSelectionOverlayVisual {
    let (min_x, min_y, max_x, max_y) = selection_bounds(selection);
    let width_in_cells = max_x - min_x + 1;
    let height_in_cells = max_y - min_y + 1;
    let irregular = !selection_cells_are_rectangular(selection, &selection_cells);
    let show_handles =
        !irregular && !transfer_active && (width_in_cells > 1 || height_in_cells > 1);
    let show_irregular_handles = irregular && (width_in_cells > 1 || height_in_cells > 1);
    let region_style = signed_preview_frame_style(
        document.map.tile_width,
        document.map.tile_height,
        min_x,
        min_y,
        max_x,
        max_y,
    );

    TileSelectionOverlayVisual {
        preview,
        closing,
        irregular,
        region_style,
        cell_styles: if irregular {
            selection_cells
                .into_iter()
                .map(|(x, y)| {
                    signed_preview_frame_style(
                        document.map.tile_width,
                        document.map.tile_height,
                        x,
                        y,
                        x,
                        y,
                    )
                })
                .collect()
        } else {
            Vec::new()
        },
        show_handles,
        show_irregular_handles,
        handles: if show_handles || show_irregular_handles {
            vec![
                TileSelectionHandleVisual::new("top-left", "left:-11px;top:-11px;"),
                TileSelectionHandleVisual::new("top-right", "right:-11px;top:-11px;"),
                TileSelectionHandleVisual::new("bottom-left", "left:-11px;bottom:-11px;"),
                TileSelectionHandleVisual::new("bottom-right", "right:-11px;bottom:-11px;"),
            ]
        } else {
            Vec::new()
        },
    }
}

fn active_tile_selection_transfer_preview(
    document: &EditorDocument,
    snapshot: &AppState,
) -> Option<TileSelectionTransferPreviewVisual> {
    let transfer = snapshot.tile_selection_transfer.as_ref()?;
    let selection = snapshot.tile_selection?;
    let (min_x, min_y, _, _) = selection_bounds(selection);
    let mut tiles = Vec::new();

    for local_y in 0..transfer.height {
        for local_x in 0..transfer.width {
            let x = min_x + local_x as i32;
            let y = min_y + local_y as i32;
            let gid = transfer.tiles[(local_y * transfer.width + local_x) as usize];
            if gid == 0 {
                continue;
            }
            let style = preview_tile_style_signed(document, &snapshot.image_cache, gid, x, y)
                .unwrap_or_else(|| {
                    signed_cell_style(document.map.tile_width, document.map.tile_height, x, y)
                });
            tiles.push(ShapeFillPreviewTile {
                x,
                y,
                style,
                fallback: document.map.tile_reference_for_gid(gid).is_none(),
            });
        }
    }

    Some(TileSelectionTransferPreviewVisual { tiles })
}

struct TileSelectionOverlayVisual {
    preview: bool,
    closing: bool,
    irregular: bool,
    region_style: String,
    cell_styles: Vec<String>,
    show_handles: bool,
    show_irregular_handles: bool,
    handles: Vec<TileSelectionHandleVisual>,
}

struct TileSelectionHandleVisual {
    position: &'static str,
    style: &'static str,
}

impl TileSelectionHandleVisual {
    const fn new(position: &'static str, style: &'static str) -> Self {
        Self { position, style }
    }
}

fn dismiss_selection_from_outside_map_click(state: &mut AppState, x: f64, y: f64) {
    if !is_tile_selection_tool(state.tool) || state.tile_selection.is_none() {
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let active_layer = session.document().map.layer(state.active_layer);
    if active_layer.is_none_or(|layer| layer.as_tile().is_none()) {
        return;
    }
    if cell_from_surface(state, x, y).is_some() {
        return;
    }

    if state.tile_selection_transfer.is_some() {
        return;
    }
    clear_tile_selection_immediately(state);
    state.status = "Selection cleared.".to_string();
}

fn handle_canvas_click(state: &mut AppState, x: f64, y: f64) {
    dismiss_selection_from_outside_map_click(state, x, y);

    let Some(session) = state.session.as_ref() else {
        return;
    };
    let active_layer = session.document().map.layer(state.active_layer);
    if active_layer.is_none_or(|layer| layer.as_tile().is_none()) {
        return;
    }
    let Some((cell_x, cell_y)) = cell_from_surface(state, x, y) else {
        return;
    };
    apply_cell_tool(state, cell_x, cell_y);
}

fn signed_preview_frame_style(
    tile_width: u32,
    tile_height: u32,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
) -> String {
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        min_x * tile_width as i32,
        min_y * tile_height as i32,
        ((max_x - min_x + 1) as u32) * tile_width,
        ((max_y - min_y + 1) as u32) * tile_height,
    )
}
