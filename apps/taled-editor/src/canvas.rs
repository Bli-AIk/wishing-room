use std::collections::BTreeMap;

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

/// Render the tile map canvas area.
pub(crate) fn render_canvas(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    _canvas_width: f32,
    _canvas_height: f32,
) {
    ui.element()
        .id("canvas-area")
        .width(grow!())
        .height(grow!())
        .background_color(theme.canvas_base)
        .overflow(|o| o.clip())
        .children(|ui| {
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

            let map_texture = render_tile_map(
                map,
                &state.tileset_textures,
                state.active_layer,
                tile_w,
                tile_h,
                map_px_w,
                map_px_h,
            );

            let show_grid = state.show_grid;
            let full_texture = render_to_texture(scaled_w, scaled_h, || {
                clear_background(MacroquadColor::from_rgba(0, 0, 0, 0));
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
            });

            ui.element()
                .width(fixed!(scaled_w))
                .height(fixed!(scaled_h))
                .image(full_texture)
                .floating(|f| f.offset((state.pan_x, state.pan_y)).passthrough())
                .empty();
        });
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
) -> Texture2D {
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
                    if gid == 0 {
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

                    let dx = col as f32 * tile_w;
                    let dy = row as f32 * tile_h;

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
    let grid_color = MacroquadColor::new(
        theme.grid_line.r / 255.0,
        theme.grid_line.g / 255.0,
        theme.grid_line.b / 255.0,
        theme.grid_line.a / 255.0,
    );
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
