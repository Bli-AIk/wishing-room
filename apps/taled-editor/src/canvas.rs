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
                build_and_cache_canvas(
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
                );
                state.canvas_dirty = false;
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
            render_debug_overlay(ui, &state.debug_info.clone());
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

/// Build the composited canvas texture (tile map + optional grid) at the given zoom,
/// and cache the RenderTarget in the global TextureManager to persist across frames.
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
) {
    let scaled_w = map_px_w * zoom;
    let scaled_h = map_px_h * zoom;
    let map_texture = render_tile_map(
        map,
        textures,
        active_layer,
        tile_w,
        tile_h,
        map_px_w,
        map_px_h,
        theme,
    );
    map_texture.set_filter(FilterMode::Nearest);

    let rt = render_target_msaa(scaled_w as u32, scaled_h as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, scaled_w, scaled_h));
    cam.render_target = Some(rt.clone());
    set_camera(&cam);

    let cb: MacroquadColor = theme.canvas_base.into();
    clear_background(cb);
    draw_texture_ex(
        &map_texture,
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

    // Draw selection overlays on top of the grid.
    let cell_w = tile_w * zoom;
    let cell_h = tile_h * zoom;
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
}

/// Retrieve the cached canvas texture, keeping it alive in the TextureManager.
fn get_cached_canvas() -> Option<Texture2D> {
    ply_engine::renderer::TEXTURE_MANAGER
        .lock()
        .expect("texture manager lock")
        .get(CANVAS_CACHE_KEY)
        .cloned()
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
) -> Texture2D {
    let empty_color: MacroquadColor = theme.empty_tile.into();

    render_to_texture(map_px_w, map_px_h, || {
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
    })
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

/// Draw selection overlay for a set of cells.
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

    // Animated border pulse: opacity oscillates between 0.42 and 0.96 over 880ms.
    let t = get_time();
    let pulse = 0.69 + 0.27 * ((t * std::f64::consts::TAU / 0.88).sin() as f32);
    let border = MacroquadColor::new(0.96, 0.97, 0.98, pulse);

    for &(cx, cy) in cells {
        let px = cx as f32 * cell_w;
        // Flip Y: the map texture goes through two render targets (double-invert = correct),
        // but this overlay is drawn in the second render target only (single-invert = flipped).
        let py = canvas_h - (cy + 1) as f32 * cell_h;
        draw_rectangle(px, py, cell_w, cell_h, fill);

        // Border edges: after Y-flip, top/bottom drawing edges are swapped on screen.
        let has_left = cells.contains(&(cx - 1, cy));
        let has_right = cells.contains(&(cx + 1, cy));
        let has_top = cells.contains(&(cx, cy - 1));
        let has_bottom = cells.contains(&(cx, cy + 1));
        if !has_top {
            draw_line(px, py + cell_h, px + cell_w, py + cell_h, 1.0, border);
        }
        if !has_bottom {
            draw_line(px, py, px + cell_w, py, 1.0, border);
        }
        if !has_left {
            draw_line(px, py, px, py + cell_h, 1.0, border);
        }
        if !has_right {
            draw_line(px + cell_w, py, px + cell_w, py + cell_h, 1.0, border);
        }
    }
}
