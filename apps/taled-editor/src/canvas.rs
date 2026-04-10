use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::canvas_overlay::{TransferPreview, draw_grid, draw_selection_overlay, draw_transfer_preview};
use crate::theme::PlyTheme;

thread_local! {
    static REUSE_CANVAS_RT: RefCell<Option<(RenderTarget, u32, u32)>> = const { RefCell::new(None) };
    static REUSE_TILEMAP_RT: RefCell<Option<(RenderTarget, u32, u32)>> = const { RefCell::new(None) };
}

/// Reuse an existing MSAA render target when dimensions match, avoiding per-frame GPU allocation.
fn reuse_msaa_rt(
    storage: &'static std::thread::LocalKey<RefCell<Option<(RenderTarget, u32, u32)>>>,
    w: u32,
    h: u32,
) -> RenderTarget {
    storage.with_borrow_mut(|slot| {
        if let Some((rt, ew, eh)) = slot
            && *ew == w
            && *eh == h
        {
            return rt.clone();
        }
        let rt = render_target_msaa(w, h);
        rt.texture.set_filter(FilterMode::Nearest);
        *slot = Some((rt.clone(), w, h));
        rt
    })
}

/// Load tileset image data into macroquad Texture2D objects.
pub(crate) fn load_tileset_textures(state: &mut AppState) {
    state.tileset_textures.clear();
    state.tile_chip_cache.clear();
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
                crate::logging::append(&format!("tileset {index} image FAIL: {e}"));
            }
        }
    }
}

/// The Y coordinate where the canvas area begins (header + tile strip), excluding safe area.
pub(crate) const CANVAS_ORIGIN_Y: f32 = 170.0;

/// Render the tile map canvas area.
pub(crate) fn render_canvas(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let canvas_y = CANVAS_ORIGIN_Y + state.safe_inset_top;
    ui.element()
        .id("canvas-area")
        .width(grow!())
        .height(grow!())
        .background_color(theme.canvas_base)
        .overflow(|o| o.clip())
        .on_press(move |_, _| {})
        .children(|ui| {
            crate::touch_ops::handle_canvas_interaction(ui, state, canvas_y);

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

            // Collect transfer preview data for semi-transparent floating tile rendering.
            let transfer_preview = state.tile_selection_transfer.as_ref().and_then(|tr| {
                let region = state.tile_selection.as_ref()?;
                let (ox, oy, _, _) = crate::app_state::selection_bounds(region);
                Some(TransferPreview {
                    origin_x: ox,
                    origin_y: oy,
                    width: tr.width,
                    height: tr.height,
                    tiles: tr.tiles.clone(),
                    mask: tr.mask.clone(),
                })
            });

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
                    transfer_preview.as_ref(),
                    tiles_dirty,
                );
                state.perf_info = perf;
                state.canvas_rebuild_count += 1;
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
                            .clip_by_parent()
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
        .floating(|f| {
            f.attach_parent()
                .offset((0.0, 0.0))
                .passthrough()
                .z_index(100)
        })
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
    transfer_preview: Option<&TransferPreview>,
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

    let rt = reuse_msaa_rt(&REUSE_CANVAS_RT, scaled_w as u32, scaled_h as u32);
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

    // Draw transfer floating tiles with opaque backing to hide underlying tiles.
    if let Some(tp) = transfer_preview {
        let bg: MacroquadColor = theme.canvas_base.into();
        draw_transfer_preview(tp, map, textures, tile_w, tile_h, zoom, scaled_h, bg);
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

    let t3 = get_time();

    set_default_camera();

    ply_engine::renderer::TEXTURE_MANAGER
        .lock()
        .expect("texture manager lock")
        .cache(CANVAS_CACHE_KEY.to_string(), rt);

    let t4 = get_time();
    let ms = |a: f64, b: f64| ((b - a) * 1000.0) as f32;
    let perf = format!(
        "tile:{:.1} comp:{:.1} sel:{:.1}({sel_n}) flush:{:.1} tot:{:.1}ms",
        ms(t0, t1),
        ms(t1, t2),
        ms(t2, t3),
        ms(t3, t4),
        ms(t0, t4)
    );
    eprintln!("[perf] {perf}");
    perf
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

    let rt = reuse_msaa_rt(&REUSE_TILEMAP_RT, map_px_w as u32, map_px_h as u32);
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
