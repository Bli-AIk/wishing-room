use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::theme::PlyTheme;

pub(crate) struct PaletteTile {
    pub(crate) gid: u32,
    pub(crate) tileset_index: usize,
    pub(crate) local_id: u32,
}

pub(crate) fn collect_palette_preview(state: &AppState, limit: usize) -> Vec<PaletteTile> {
    let mut palette = Vec::with_capacity(limit);
    let Some(session) = state.session.as_ref() else {
        return palette;
    };
    for (tileset_index, tileset) in session.document().map.tilesets.iter().enumerate() {
        for local_id in 0..tileset.tileset.tile_count {
            palette.push(PaletteTile {
                gid: tileset.first_gid + local_id,
                tileset_index,
                local_id,
            });
            if palette.len() >= limit {
                return palette;
            }
        }
    }
    palette
}

/// Render tile chips in a 2-row column-first grid (matching CSS grid-auto-flow: column).
pub(crate) fn render_tile_chip_grid(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    palette: &[PaletteTile],
) {
    let num_cols = palette.len().div_ceil(2);

    // Pre-compute indices for each row (column-first: col*2+row)
    let row_indices: [Vec<usize>; 2] = [
        (0..num_cols)
            .map(|c| c * 2)
            .filter(|&i| i < palette.len())
            .collect(),
        (0..num_cols)
            .map(|c| c * 2 + 1)
            .filter(|&i| i < palette.len())
            .collect(),
    ];

    for indices in &row_indices {
        ui.element()
            .width(fit!())
            .height(fixed!(44.0))
            .layout(|l| l.direction(LeftToRight).align(Left, Top).gap(6))
            .children(|ui| {
                for &idx in indices {
                    render_tile_chip(ui, state, theme, &palette[idx]);
                }
            });
    }
}

fn render_tile_chip(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, tile: &PaletteTile) {
    let is_selected = state.selected_gid == tile.gid;
    let chip_bg = Color::u_rgb(0x10, 0x11, 0x13);
    let border_color = if is_selected {
        theme.accent
    } else {
        theme.border
    };
    let border_width = if is_selected { 2 } else { 1 };
    let radius = if is_selected { 0.0 } else { 8.0 };

    let tile_tex = crop_tile_texture(state, tile);
    let gid = tile.gid;

    ui.element()
        .id(("tile-chip", gid))
        .width(fixed!(44.0))
        .height(fixed!(44.0))
        .background_color(chip_bg)
        .corner_radius(radius)
        .border(|b| b.all(border_width).color(border_color))
        .overflow(|o| o.clip())
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.selected_gid = gid;
            }
            if let Some(tex) = tile_tex {
                ui.element()
                    .width(fixed!(40.0))
                    .height(fixed!(40.0))
                    .image(tex)
                    .empty();
            }
        });
}

fn crop_tile_texture(state: &mut AppState, tile: &PaletteTile) -> Option<Texture2D> {
    if let Some(cached) = state.tile_chip_cache.get(&tile.gid) {
        return Some(cached.texture.clone());
    }
    let session = state.session.as_ref()?;
    let texture = state.tileset_textures.get(&tile.tileset_index)?;
    let tile_ref = session.document().map.tile_reference_for_gid(tile.gid)?;

    let ts = &tile_ref.tileset.tileset;
    let cols = ts.columns.max(1);
    let tw = ts.tile_width as f32;
    let th = ts.tile_height as f32;
    let sx = (tile.local_id % cols) as f32 * tw;
    let sy = (tile.local_id / cols) as f32 * th;

    let chip_size = 40.0;
    let scale = (chip_size / tw).min(chip_size / th);
    let rw = tw * scale;
    let rh = th * scale;
    let ox = (chip_size - rw) / 2.0;
    let oy = (chip_size - rh) / 2.0;

    // Keep the full RenderTarget alive — Android frees the GL framebuffer on drop.
    let rt = render_target(chip_size as u32, chip_size as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, chip_size, chip_size));
    cam.render_target = Some(rt.clone());
    set_camera(&cam);

    clear_background(MacroquadColor::from_rgba(0x10, 0x11, 0x13, 255));
    draw_texture_ex(
        texture,
        ox,
        oy,
        WHITE,
        DrawTextureParams {
            source: Some(Rect::new(sx, sy, tw, th)),
            dest_size: Some(Vec2::new(rw, rh)),
            ..Default::default()
        },
    );

    set_default_camera();
    let tex = rt.texture.clone();
    state.tile_chip_cache.insert(tile.gid, rt);
    Some(tex)
}
