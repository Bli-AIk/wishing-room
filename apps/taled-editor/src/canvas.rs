use std::collections::{BTreeMap, BTreeSet};

use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::theme::PlyTheme;

/// Load tileset image data into macroquad Texture2D objects.
pub(crate) fn load_tileset_textures(state: &mut AppState) {
    state.tileset_textures.clear();
    let Some(session) = state.session.as_ref() else {
        return;
    };
    for (index, _ts_ref) in session.document().map.tilesets.iter().enumerate() {
        match session.tileset_image_bytes(index) {
            Ok(bytes) => {
                let texture = Texture2D::from_file_with_format(&bytes, None);
                texture.set_filter(FilterMode::Nearest);
                state.tileset_textures.insert(index, texture);
            }
            Err(e) => {
                eprintln!("Failed to load tileset {index} image: {e}");
            }
        }
    }
}

/// The Y coordinate where the canvas area begins (header + tile strip).
pub(crate) const CANVAS_ORIGIN_Y: f32 = 170.0;

/// Render the tile map canvas area.
pub(crate) fn render_canvas(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    ui.element()
        .id("canvas-area")
        .width(grow!())
        .height(grow!())
        .background_color(theme.canvas_base)
        .overflow(|o| o.clip())
        .on_press(move |_, _| {})
        .children(|ui| {
            crate::touch_ops::handle_canvas_interaction(ui, state, CANVAS_ORIGIN_Y);

            let Some(session) = state.session.as_ref() else {
                ui.text("No map loaded", |t| t.font_size(14).color(theme.muted_text));
                return;
            };

            let map = &session.document().map;
            let tile_w = map.tile_width as f32;
            let tile_h = map.tile_height as f32;
            let map_px_w = map.total_pixel_width() as f32;
            let map_px_h = map.total_pixel_height() as f32;
            let zoom = state.zoom_percent as f32 / 100.0;
            let scaled_w = map_px_w * zoom;
            let scaled_h = map_px_h * zoom;

            // Collect selection cells for overlay rendering.
            let sel_cells = state.tile_selection_cells.clone();
            let preview_cells = state.tile_selection_preview_cells.clone();

            // Only rebuild the canvas texture when content or zoom changed.
            let needs_rebuild =
                state.canvas_dirty || state.canvas_cached_zoom != state.zoom_percent;
            if needs_rebuild || get_cached_canvas().is_none() {
                let tiles_dirty =
                    state.tiles_dirty || state.canvas_cached_zoom != state.zoom_percent;
                let perf = build_and_cache_canvas(
                    map,
                    &state.tileset_textures,
                    state.active_layer,
                    state.show_grid,
                    tile_w,
                    tile_h,
                    map_px_w,
                    map_px_h,
                    zoom,
                    theme,
                    sel_cells.as_ref(),
                    preview_cells.as_ref(),
                    tiles_dirty,
                );
                state.perf_info = perf;
                state.canvas_dirty = false;
                state.tiles_dirty = false;
                state.canvas_cached_zoom = state.zoom_percent;
            }

            if let Some(cached) = get_cached_canvas() {
                ui.element()
                    .width(fixed!(scaled_w))
                    .height(fixed!(scaled_h))
                    .image(cached)
                    .floating(|f| {
                        f.attach_parent()
                            .offset((state.pan_x, state.pan_y))
                            .passthrough()
                    })
                    .empty();
            }

            crate::screens::editor_toolbar::render_floating_controls(ui, state, theme);
            let overlay = if state.perf_info.is_empty() {
                state.debug_info.clone()
            } else {
                format!("{} | {}", state.debug_info, state.perf_info)
            };
            render_debug_overlay(ui, &overlay);
        });
}

fn render_debug_overlay(ui: &mut Ui, info: &str) {
    if info.is_empty() {
        return;
    }
    ui.element()
        .width(grow!())
        .height(fixed!(16.0))
        .floating(|f| f.attach_parent().offset((0.0, 0.0)).passthrough())
        .children(|ui| {
            ui.text(info, |t| {
                t.font_size(10).color(Color::from((255u8, 255, 0, 200)))
            });
        });
}

/// Cache key for the composited canvas render target in the global TextureManager.
const CANVAS_CACHE_KEY: &str = "taled-canvas";

/// Cache key for the tile-map-only render target (reused when only selection changes).
const TILEMAP_CACHE_KEY: &str = "taled-tilemap";

/// Build the composited canvas texture (tile map + optional grid + overlays) at the given zoom,
/// and cache the RenderTarget in the global TextureManager to persist across frames.
///
/// When `tiles_dirty` is false and a cached tilemap exists, the expensive tile rendering
/// is skipped entirely — only the composition step (scale + grid + selection overlay) runs.
/// Returns a timing string for the debug overlay (tilemap_ms | compose_ms | overlay_ms).
fn build_and_cache_canvas(
    map: &taled_core::Map,
    textures: &BTreeMap<usize, Texture2D>,
    active_layer: usize,
    show_grid: bool,
    tile_w: f32,
    tile_h: f32,
    map_px_w: f32,
    map_px_h: f32,
    zoom: f32,
    theme: &PlyTheme,
    selection_cells: Option<&BTreeSet<(i32, i32)>>,
    preview_cells: Option<&BTreeSet<(i32, i32)>>,
    tiles_dirty: bool,
) -> String {
    let scaled_w = map_px_w * zoom;
    let scaled_h = map_px_h * zoom;

    let t0 = get_time();

    // Reuse cached tilemap when tile data hasn't changed.
    let tilemap_tex = {
        let mut tm = ply_engine::renderer::TEXTURE_MANAGER
            .lock()
            .expect("texture manager lock");
        if !tiles_dirty {
            if let Some(tex) = tm.get(TILEMAP_CACHE_KEY) {
                tex.clone()
            } else {
                drop(tm);
                cache_tilemap(
                    map,
                    textures,
                    active_layer,
                    tile_w,
                    tile_h,
                    map_px_w,
                    map_px_h,
                    theme,
                )
            }
        } else {
            drop(tm);
            cache_tilemap(
                map,
                textures,
                active_layer,
                tile_w,
                tile_h,
                map_px_w,
                map_px_h,
                theme,
            )
        }
    };

    let t1 = get_time();

    let rt = render_target_msaa(scaled_w as u32, scaled_h as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, scaled_w, scaled_h));
    cam.render_target = Some(rt.clone());
    set_camera(&cam);

    let cb: MacroquadColor = theme.canvas_base.into();
    clear_background(cb);
    draw_texture_ex(
        &tilemap_tex,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2::new(scaled_w, scaled_h)),
            ..Default::default()
        },
    );
    if show_grid {
        draw_grid(map.width, map.height, tile_w * zoom, tile_h * zoom, theme);
    }

    let t2 = get_time();

    // Draw selection overlays on top of the grid.
    let cell_w = tile_w * zoom;
    let cell_h = tile_h * zoom;
    let sel_n = selection_cells.map_or(0, BTreeSet::len) + preview_cells.map_or(0, BTreeSet::len);
    if let Some(cells) = selection_cells {
        draw_selection_overlay(cells, cell_w, cell_h, scaled_h, false);
    }
    if let Some(cells) = preview_cells {
        draw_selection_overlay(cells, cell_w, cell_h, scaled_h, true);
    }

    set_default_camera();

    ply_engine::renderer::TEXTURE_MANAGER
        .lock()
        .expect("texture manager lock")
        .cache(CANVAS_CACHE_KEY.to_string(), rt);

    let t3 = get_time();
    let ms = |a: f64, b: f64| ((b - a) * 1000.0) as f32;
    format!(
        "tile:{:.1} comp:{:.1} sel:{:.1}({sel_n}) tot:{:.1}ms",
        ms(t0, t1),
        ms(t1, t2),
        ms(t2, t3),
        ms(t0, t3)
    )
}

/// Retrieve the cached canvas texture, keeping both canvas and tilemap alive in TextureManager.
fn get_cached_canvas() -> Option<Texture2D> {
    let mut tm = ply_engine::renderer::TEXTURE_MANAGER
        .lock()
        .expect("texture manager lock");
    let _ = tm.get(TILEMAP_CACHE_KEY);
    tm.get(CANVAS_CACHE_KEY).cloned()
}

/// Render the tile map and cache the RenderTarget in TextureManager.
/// Caching the full RenderTarget (not just Texture2D) keeps the GL framebuffer alive,
/// preventing the texture from going black on Android after the render pass is freed.
fn cache_tilemap(
    map: &taled_core::Map,
    textures: &BTreeMap<usize, Texture2D>,
    active_layer: usize,
    tile_w: f32,
    tile_h: f32,
    map_px_w: f32,
    map_px_h: f32,
    theme: &PlyTheme,
) -> Texture2D {
    let rt = render_tile_map(
        map,
        textures,
        active_layer,
        tile_w,
        tile_h,
        map_px_w,
        map_px_h,
        theme,
    );
    rt.texture.set_filter(FilterMode::Nearest);
    ply_engine::renderer::TEXTURE_MANAGER
        .lock()
        .expect("texture manager lock")
        .cache(TILEMAP_CACHE_KEY.to_string(), rt)
        .clone()
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
fn render_tile_map(
    map: &taled_core::Map,
    textures: &BTreeMap<usize, Texture2D>,
    active_layer: usize,
    tile_w: f32,
    tile_h: f32,
    map_px_w: f32,
    map_px_h: f32,
    theme: &PlyTheme,
) -> RenderTarget {
    let empty_color: MacroquadColor = theme.empty_tile.into();

    let rt = render_target_msaa(map_px_w as u32, map_px_h as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, map_px_w, map_px_h));
    cam.render_target = Some(rt.clone());
    set_camera(&cam);

    clear_background(MacroquadColor::from_rgba(0, 0, 0, 0));

    for (layer_idx, layer) in map.layers.iter().enumerate() {
        if !layer.visible() {
            continue;
        }
        let Some(tile_layer) = layer.as_tile() else {
            continue;
        };

        let alpha = if layer_idx == active_layer { 1.0 } else { 0.6 };
        let color = MacroquadColor::new(1.0, 1.0, 1.0, alpha);

        for row in 0..map.height {
            for col in 0..map.width {
                let idx = (row * map.width + col) as usize;
                let gid = tile_layer.tiles.get(idx).copied().unwrap_or(0);
                let dx = col as f32 * tile_w;
                let dy = row as f32 * tile_h;

                if gid == 0 {
                    if layer_idx == active_layer {
                        draw_rectangle(dx, dy, tile_w, tile_h, empty_color);
                    }
                    continue;
                }
                let Some(tile_ref) = map.tile_reference_for_gid(gid) else {
                    continue;
                };
                let tileset_index = tile_ref.tileset_index;
                let Some(texture) = textures.get(&tileset_index) else {
                    continue;
                };

                let ts = &tile_ref.tileset.tileset;
                let cols_in_tileset = (ts.image.width / ts.tile_width).max(1);
                let src_col = tile_ref.local_id % cols_in_tileset;
                let src_row = tile_ref.local_id / cols_in_tileset;
                let sx = src_col as f32 * ts.tile_width as f32;
                let sy = src_row as f32 * ts.tile_height as f32;

                draw_texture_ex(
                    texture,
                    dx,
                    dy,
                    color,
                    DrawTextureParams {
                        source: Some(Rect::new(
                            sx,
                            sy,
                            ts.tile_width as f32,
                            ts.tile_height as f32,
                        )),
                        dest_size: Some(Vec2::new(tile_w, tile_h)),
                        ..Default::default()
                    },
                );
            }
        }
    }

    set_default_camera();
    rt
}

fn draw_grid(cols: u32, rows: u32, cell_w: f32, cell_h: f32, theme: &PlyTheme) {
    let grid_color: MacroquadColor = theme.grid_line.into();
    let total_w = cols as f32 * cell_w;
    let total_h = rows as f32 * cell_h;

    for col in 0..=cols {
        let x = col as f32 * cell_w;
        draw_line(x, 0.0, x, total_h, 1.0, grid_color);
    }
    for row in 0..=rows {
        let y = row as f32 * cell_h;
        draw_line(0.0, y, total_w, y, 1.0, grid_color);
    }
}

/// Draw selection overlay for a set of cells using horizontal span merging.
/// `is_preview` uses a lighter fill for drag-in-progress feedback.
/// `canvas_h` is needed to flip Y: the render-target Camera2D inverts Y
/// relative to the map texture (which passes through an extra render target).
fn draw_selection_overlay(
    cells: &BTreeSet<(i32, i32)>,
    cell_w: f32,
    cell_h: f32,
    canvas_h: f32,
    is_preview: bool,
) {
    if cells.is_empty() {
        return;
    }
    let fill_alpha = if is_preview { 0.10 } else { 0.16 };
    let fill = MacroquadColor::new(0.0, 0.0, 0.0, fill_alpha);

    let t = get_time();
    let pulse = 0.69 + 0.27 * ((t * std::f64::consts::TAU / 0.88).sin() as f32);
    let border = MacroquadColor::new(0.96, 0.97, 0.98, pulse);

    // Group cells by row, then merge consecutive x-values into horizontal spans.
    let mut rows: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
    for &(cx, cy) in cells {
        rows.entry(cy).or_default().push(cx);
    }
    // (y, x_start, x_end) inclusive
    let mut spans: Vec<(i32, i32, i32)> = Vec::new();
    for (&y, xs) in &mut rows {
        xs.sort_unstable();
        let mut i = 0;
        while i < xs.len() {
            let start = xs[i];
            let mut end = start;
            while i + 1 < xs.len() && xs[i + 1] == end + 1 {
                i += 1;
                end = xs[i];
            }
            spans.push((y, start, end));
            i += 1;
        }
    }

    for &(row, x_start, x_end) in &spans {
        let px = x_start as f32 * cell_w;
        let py = canvas_h - (row + 1) as f32 * cell_h;
        let span_w = (x_end - x_start + 1) as f32 * cell_w;
        draw_rectangle(px, py, span_w, cell_h, fill);

        // Left / right borders (only at span boundaries).
        if !cells.contains(&(x_start - 1, row)) {
            draw_line(px, py, px, py + cell_h, 1.0, border);
        }
        if !cells.contains(&(x_end + 1, row)) {
            let rx = px + span_w;
            draw_line(rx, py, rx, py + cell_h, 1.0, border);
        }

        // Top / bottom borders: merge consecutive cells that need a border into sub-spans.
        draw_merged_h_border(cells, x_start, x_end, row, -1, cell_w, py + cell_h, border);
        draw_merged_h_border(cells, x_start, x_end, row, 1, cell_w, py, border);
    }
}

/// Draw a merged horizontal border line for cells in `x_start..=x_end` at `row`.
/// `dy` is -1 for top border or +1 for bottom border (map-coordinate neighbor offset).
fn draw_merged_h_border(
    cells: &BTreeSet<(i32, i32)>,
    x_start: i32,
    x_end: i32,
    row: i32,
    dy: i32,
    cell_w: f32,
    line_y: f32,
    color: MacroquadColor,
) {
    let mut seg_start: Option<i32> = None;
    for x in x_start..=x_end {
        if !cells.contains(&(x, row + dy)) {
            if seg_start.is_none() {
                seg_start = Some(x);
            }
        } else if let Some(ss) = seg_start {
            let lx = ss as f32 * cell_w;
            let rx = x as f32 * cell_w;
            draw_line(lx, line_y, rx, line_y, 1.0, color);
            seg_start = None;
        }
    }
    if let Some(ss) = seg_start {
        let lx = ss as f32 * cell_w;
        let rx = (x_end + 1) as f32 * cell_w;
        draw_line(lx, line_y, rx, line_y, 1.0, color);
    }
}
