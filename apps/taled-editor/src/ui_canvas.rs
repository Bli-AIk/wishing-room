use std::{collections::BTreeMap, time::Duration};

use dioxus::prelude::*;
use taled_core::{EditorDocument, ObjectShape};

use crate::{
    app_state::{
        selection_bounds, selection_cells_are_rectangular, selection_cells_from_region, AppState,
        TileSelectionRegion, Tool,
    },
    edit_ops::{apply_cell_tool, dismiss_tile_selection},
    platform::log,
    touch_ops::{
        cell_from_surface,
        handle_touch_pointer_cancel, handle_touch_pointer_down, handle_touch_pointer_move,
        handle_touch_pointer_up, should_ignore_synthetic_click,
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
        "width:{}px;height:{}px;transform:translate({}px, {}px) scale({zoom});",
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
                    dismiss_selection_from_outside_map_click(&mut state, event.data().element_coordinates().x, event.data().element_coordinates().y);
                },
                onpointerdown: move |event| handle_touch_pointer_down(&mut state.write(), event),
                onpointermove: move |event| handle_touch_pointer_move(&mut state.write(), event),
                onpointerup: move |event| handle_touch_pointer_up(&mut state.write(), event),
                onpointercancel: move |event| handle_touch_pointer_cancel(&mut state.write(), event),
                div {
                    class: canvas_class,
                    style: canvas_style,
                    ontransitionend: move |_| state.write().camera_transition_active = false,
                    for (layer_index, layer) in map.layers.iter().enumerate() {
                        if let Some(tile_layer) = layer.as_tile() {
                            if tile_layer.visible {
                                for y in 0..tile_layer.height {
                                    for x in 0..tile_layer.width {
                                        if let Some(style) = tile_layer
                                            .tile_at(x, y)
                                            .filter(|gid| *gid != 0)
                                            .and_then(|gid| sprite_style(document, &snapshot.image_cache, gid, x, y))
                                        {
                                            div {
                                                key: "tile-{layer_index}-{x}-{y}",
                                                class: "tile-sprite",
                                                style: style,
                                            }
                                        }
                                    }
                                }
                            }
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
                                    "shape-fill-preview-tile fallback"
                                } else {
                                    "tile-preview shape-fill-preview-tile"
                                },
                                style: "{tile.style}",
                            }
                        }
                    })}

                    for y in 0..map.height {
                        for x in 0..map.width {
                            div {
                                key: "cell-{x}-{y}",
                                class: if !has_tile_selection_overlay && snapshot.selected_cell == Some((x, y)) {
                                    "cell-hitbox selected"
                                } else {
                                    "cell-hitbox"
                                },
                                style: cell_style(map.tile_width, map.tile_height, x, y),
                                onclick: move |_| {
                                    let mut state = state.write();
                                    if should_ignore_synthetic_click(&mut state) {
                                        return;
                                    }
                                    apply_cell_tool(&mut state, x, y);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
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

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let style =
                preview_tile_style(document, &snapshot.image_cache, snapshot.selected_gid, x, y)
                    .unwrap_or_else(|| {
                        cell_style(document.map.tile_width, document.map.tile_height, x, y)
                    });
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
    if snapshot.tool != Tool::Select {
        return None;
    }
    let active_layer = document.map.layer(snapshot.active_layer)?;
    active_layer.as_tile()?;

    let closing_region = snapshot.tile_selection_closing;
    let (selection, selection_cells, preview, closing) =
        if let Some(selection) = snapshot.tile_selection_preview {
            (
                selection,
                selection_cells_from_region(selection),
                true,
                false,
            )
        } else if let (Some(selection), Some(selection_cells)) =
            (snapshot.tile_selection, snapshot.tile_selection_cells.clone())
        {
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
        handles: if show_handles {
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
    if state.tool != Tool::Select || state.tile_selection.is_none() {
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
    dismiss_tile_selection(state);
    state.status = "Selection cleared.".to_string();
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
