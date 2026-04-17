use std::collections::{BTreeMap, BTreeSet};

use ply_engine::prelude::*;

use crate::theme::PlyTheme;

/// Lightweight snapshot of transfer-mode floating tiles for canvas preview rendering.
pub(crate) struct TransferPreview {
    pub(crate) origin_x: i32,
    pub(crate) origin_y: i32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) tiles: Vec<u32>,
    pub(crate) mask: Vec<bool>,
}

pub(super) fn draw_grid(cols: u32, rows: u32, cell_w: f32, cell_h: f32, theme: &PlyTheme) {
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

/// Draw semi-transparent floating tiles for transfer-mode preview.
/// First draws an opaque background to hide underlying tiles, then renders
/// the transfer tiles at 50% opacity so they don't blend with the map.
pub(super) fn draw_transfer_preview(
    tp: &TransferPreview,
    map: &taled_core::Map,
    textures: &BTreeMap<usize, Texture2D>,
    tile_textures: &BTreeMap<(usize, u32), Texture2D>,
    tile_w: f32,
    tile_h: f32,
    zoom: f32,
    canvas_h: f32,
    bg_color: MacroquadColor,
) {
    let color = MacroquadColor::new(1.0, 1.0, 1.0, 0.5);
    let zw = tile_w * zoom;
    let zh = tile_h * zoom;
    // First pass: draw opaque background to mask underlying tiles.
    for row in 0..tp.height {
        for col in 0..tp.width {
            let idx = (row * tp.width + col) as usize;
            if !tp.mask.get(idx).copied().unwrap_or(false) {
                continue;
            }
            let dx = (tp.origin_x + col as i32) as f32 * zw;
            let dy = canvas_h - (tp.origin_y + row as i32 + 1) as f32 * zh;
            draw_rectangle(dx, dy, zw, zh, bg_color);
        }
    }
    // Second pass: draw tiles at half opacity.
    for row in 0..tp.height {
        for col in 0..tp.width {
            let idx = (row * tp.width + col) as usize;
            if !tp.mask.get(idx).copied().unwrap_or(false) {
                continue;
            }
            let gid = tp.tiles.get(idx).copied().unwrap_or(0);
            if gid == 0 {
                continue;
            }
            let Some(tile_ref) = map.tile_reference_for_gid(gid) else {
                continue;
            };
            let ts = &tile_ref.tileset.tileset;
            let dx = (tp.origin_x + col as i32) as f32 * zw;
            let dy = canvas_h - (tp.origin_y + row as i32 + 1) as f32 * zh;

            let (flip_h, flip_v, flip_d) = taled_core::tile_flip_flags(gid);
            let (rotation, flip_x, flip_y) = crate::canvas::tile_transform(flip_h, flip_v, flip_d);
            let pivot = Some(Vec2::new(dx + zw / 2.0, dy + zh / 2.0));

            if let Some(tile_tex) = tile_textures.get(&(tile_ref.tileset_index, tile_ref.local_id))
            {
                // COI tiles: draw at actual size, bottom-aligned to the grid cell.
                let tex_w = tile_tex.width() * zoom;
                let tex_h = tile_tex.height() * zoom;
                let row_abs = (tp.origin_y + row as i32) as f32;
                let coi_dy = canvas_h - row_abs * zh - tex_h;
                let pivot = Some(Vec2::new(dx + tex_w / 2.0, coi_dy + tex_h / 2.0));
                draw_texture_ex(
                    tile_tex,
                    dx,
                    coi_dy,
                    color,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(tex_w, tex_h)),
                        rotation,
                        flip_x,
                        flip_y,
                        pivot,
                        ..Default::default()
                    },
                );
            } else if let Some(texture) = textures.get(&tile_ref.tileset_index) {
                let cols_in_ts = (ts.image.width / ts.tile_width).max(1);
                let src_col = tile_ref.local_id % cols_in_ts;
                let src_row = tile_ref.local_id / cols_in_ts;
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
                        dest_size: Some(Vec2::new(zw, zh)),
                        rotation,
                        flip_x,
                        flip_y,
                        pivot,
                    },
                );
            }
        }
    }
}

/// Draw selection overlay for a set of cells using horizontal span merging.
/// `is_preview` uses a lighter fill for drag-in-progress feedback.
/// `canvas_h` is needed to flip Y: the render-target Camera2D inverts Y
/// relative to the map texture (which passes through an extra render target).
pub(super) fn draw_selection_overlay(
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
