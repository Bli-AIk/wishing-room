use ply_engine::prelude::*;

use crate::app_state::{AppState, ViewfinderSnapAnim};
use crate::theme::PlyTheme;

pub(crate) struct PaletteTile {
    pub(crate) gid: u32,
    pub(crate) tileset_index: usize,
    pub(crate) local_id: u32,
}

const SNAP_DURATION: f64 = 0.25;
const DRAG_THRESHOLD: f32 = 5.0;
const PINCH_ZOOM_IN: f64 = 0.7;
const PINCH_ZOOM_OUT: f64 = 1.4;

fn vf_grid_size(level: u8) -> (i32, i32) {
    match level {
        0 => (9, 3),
        2 => (3, 1),
        _ => (6, 2),
    }
}

/// Render the tile viewfinder with dynamic zoom and touch interaction.
/// The parent element must have `on_press` and `overflow(clip)`.
pub(crate) fn render_viewfinder(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let ts_info = active_tileset_info(state);
    let Some((ts_cols, ts_rows, tile_ids, first_gid, ts_idx)) = ts_info else {
        ui.text("No tileset", |t| t.font_size(12).color(theme.muted_text));
        return;
    };

    // Handle pinch zoom before computing grid dimensions.
    handle_viewfinder_pinch(state, ts_cols, ts_rows);

    let (vf_cols, vf_rows) = vf_grid_size(state.viewfinder_zoom_level);

    // Palette area fills: screen_width - divider(1) - side_panel(62).
    let palette_w = screen_width() - 63.0;
    let palette_h = 114.0;
    let cell_w = palette_w / vf_cols as f32;
    let cell_h = palette_h / vf_rows as f32;

    // Drive the snap easing animation (must run before grid layout).
    tick_snap_animation(state);

    // Touch handling (just_pressed / just_released from parent on_press).
    handle_viewfinder_touch(
        ui, state, cell_w, cell_h, ts_cols, ts_rows, first_gid, &tile_ids, vf_cols, vf_rows,
    );

    // Clamp offset to valid range.
    let max_x = (ts_cols - vf_cols).max(0) as f32;
    let max_y = (ts_rows - vf_rows).max(0) as f32;
    state.viewfinder_offset.0 = state.viewfinder_offset.0.clamp(0.0, max_x);
    state.viewfinder_offset.1 = state.viewfinder_offset.1.clamp(0.0, max_y);

    // Which tiles are visible?
    let off_x = state.viewfinder_offset.0;
    let off_y = state.viewfinder_offset.1;
    let base_col = off_x.floor() as i32;
    let base_row = off_y.floor() as i32;
    let frac_x = (off_x - base_col as f32) * cell_w;
    let frac_y = (off_y - base_row as f32) * cell_h;

    let extra_c = if frac_x > 0.01 { 1 } else { 0 };
    let extra_r = if frac_y > 0.01 { 1 } else { 0 };
    let rcols = vf_cols + extra_c;
    let rrows = vf_rows + extra_r;
    let grid_w = rcols as f32 * cell_w;
    let grid_h = rrows as f32 * cell_h;

    // Floating grid — passthrough so parent on_press still receives touch.
    ui.element()
        .id("vf-grid")
        .width(fixed!(grid_w))
        .height(fixed!(grid_h))
        .floating(|f| {
            f.attach_parent()
                .offset((-frac_x, -frac_y))
                .passthrough()
                .clip_by_parent()
        })
        .layout(|l| l.direction(TopToBottom))
        .children(|ui| {
            for r in 0..rrows {
                render_viewfinder_row(
                    ui,
                    state,
                    theme,
                    base_col,
                    base_row + r,
                    rcols,
                    ts_cols,
                    ts_rows,
                    &tile_ids,
                    first_gid,
                    ts_idx,
                    grid_w,
                    cell_w,
                    cell_h,
                );
            }
        });
}

fn render_viewfinder_row(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    base_col: i32,
    row: i32,
    rcols: i32,
    ts_cols: i32,
    ts_rows: i32,
    tile_ids: &[u32],
    first_gid: u32,
    ts_idx: usize,
    row_w: f32,
    cell_w: f32,
    cell_h: f32,
) {
    ui.element()
        .width(fixed!(row_w))
        .height(fixed!(cell_h))
        .layout(|l| l.direction(LeftToRight))
        .children(|ui| {
            for c in 0..rcols {
                render_viewfinder_cell(
                    ui,
                    state,
                    theme,
                    base_col + c,
                    row,
                    ts_cols,
                    ts_rows,
                    tile_ids,
                    first_gid,
                    ts_idx,
                    cell_w,
                    cell_h,
                );
            }
        });
}

// ── individual cell ──────────────────────────────────────────────────────────

fn render_viewfinder_cell(
    ui: &mut Ui,
    state: &mut AppState,
    _theme: &PlyTheme,
    col: i32,
    row: i32,
    ts_cols: i32,
    ts_rows: i32,
    tile_ids: &[u32],
    first_gid: u32,
    ts_idx: usize,
    cell_w: f32,
    cell_h: f32,
) {
    let valid = col >= 0 && col < ts_cols && row >= 0 && row < ts_rows;
    let grid_idx = if valid {
        (row * ts_cols + col) as usize
    } else {
        usize::MAX
    };
    let Some(&local_id) = tile_ids.get(grid_idx) else {
        ui.element()
            .width(fixed!(cell_w))
            .height(fixed!(cell_h))
            .empty();
        return;
    };

    let gid = first_gid + local_id;
    let is_selected = state.selected_gid == gid;

    let tile = PaletteTile {
        gid,
        tileset_index: ts_idx,
        local_id,
    };

    // For the selected tile, use a chip with blue border baked into the texture
    // (render-to-texture path, proven to work on Android).
    // `.border()` and `background_color` Rectangle don't render on Android OpenGL.
    let tile_tex = if is_selected {
        selected_tile_texture(state, &tile)
    } else {
        crop_tile_texture(state, &tile)
    };

    ui.element()
        .id(("vf-t", gid))
        .width(fixed!(cell_w))
        .height(fixed!(cell_h))
        .layout(|l| l.align(CenterX, CenterY))
        .children(|ui| {
            if let Some(tex) = tile_tex {
                let side = (cell_w - 2.0).min(cell_h - 2.0);
                ui.element()
                    .width(fixed!(side))
                    .height(fixed!(side))
                    .image(tex)
                    .empty();
            }
        });
}

// ── touch / drag / snap ──────────────────────────────────────────────────────

fn handle_viewfinder_touch(
    ui: &Ui,
    state: &mut AppState,
    cell_w: f32,
    cell_h: f32,
    ts_cols: i32,
    ts_rows: i32,
    first_gid: u32,
    tile_ids: &[u32],
    vf_cols: i32,
    vf_rows: i32,
) {
    if ui.just_pressed() {
        let (mx, my) = mouse_position();
        state.viewfinder_touch_active = true;
        state.viewfinder_dragging = false;
        state.viewfinder_drag_start_mouse = (mx, my);
        state.viewfinder_drag_start_offset = state.viewfinder_offset;
        state.viewfinder_snap_anim = None;
    }

    if state.viewfinder_touch_active {
        if touches().len() >= 2 {
            // Multi-touch in progress — suppress drag, reset anchor to avoid
            // jump when one finger lifts, and mark as non-tap interaction.
            let (mx, my) = mouse_position();
            state.viewfinder_drag_start_mouse = (mx, my);
            state.viewfinder_drag_start_offset = state.viewfinder_offset;
            state.viewfinder_dragging = true;
        } else {
            let (mx, my) = mouse_position();
            let dx = mx - state.viewfinder_drag_start_mouse.0;
            let dy = my - state.viewfinder_drag_start_mouse.1;

            if !state.viewfinder_dragging && (dx * dx + dy * dy).sqrt() > DRAG_THRESHOLD {
                state.viewfinder_dragging = true;
            }

            if state.viewfinder_dragging {
                let max_x = (ts_cols - vf_cols).max(0) as f32;
                let max_y = (ts_rows - vf_rows).max(0) as f32;
                let new_x = state.viewfinder_drag_start_offset.0 - dx / cell_w;
                let new_y = state.viewfinder_drag_start_offset.1 - dy / cell_h;
                state.viewfinder_offset = (new_x.clamp(0.0, max_x), new_y.clamp(0.0, max_y));
            }
        }
    }

    if ui.just_released() && state.viewfinder_touch_active {
        if state.viewfinder_dragging {
            let max_x = (ts_cols - vf_cols).max(0) as f32;
            let max_y = (ts_rows - vf_rows).max(0) as f32;
            let tx = state.viewfinder_offset.0.round().clamp(0.0, max_x);
            let ty = state.viewfinder_offset.1.round().clamp(0.0, max_y);
            state.viewfinder_snap_anim = Some(ViewfinderSnapAnim {
                start_time: get_time(),
                from: state.viewfinder_offset,
                to: (tx, ty),
            });
        } else {
            // Tap — select tile under finger.
            tap_select_tile(state, cell_w, cell_h, ts_cols, first_gid, tile_ids);
        }
        state.viewfinder_touch_active = false;
        state.viewfinder_dragging = false;
    }
}

fn handle_viewfinder_pinch(state: &mut AppState, ts_cols: i32, ts_rows: i32) {
    let touches = touches();
    if touches.len() < 2 {
        state.viewfinder_pinch_dist = None;
        return;
    }
    let dpi = screen_dpi_scale();
    let strip_top = state.safe_inset_top + 56.0;
    let strip_bottom = strip_top + 114.0;
    let t0 = touches[0].position / dpi;
    let t1 = touches[1].position / dpi;

    // Only handle if both touches are within the tile strip.
    if t0.y < strip_top || t0.y > strip_bottom || t1.y < strip_top || t1.y > strip_bottom {
        state.viewfinder_pinch_dist = None;
        return;
    }

    let dist = t0.distance(t1) as f64;

    if let Some(initial_dist) = state.viewfinder_pinch_dist {
        let ratio = dist / initial_dist;
        let old_level = state.viewfinder_zoom_level;
        if ratio > PINCH_ZOOM_OUT && old_level > 0 {
            state.viewfinder_zoom_level -= 1;
            state.viewfinder_pinch_dist = Some(dist);
        } else if ratio < PINCH_ZOOM_IN && old_level < 2 {
            state.viewfinder_zoom_level += 1;
            state.viewfinder_pinch_dist = Some(dist);
        }
        // Adjust offset to keep center tile visible after zoom change.
        if state.viewfinder_zoom_level != old_level {
            let (old_c, old_r) = vf_grid_size(old_level);
            let (new_c, new_r) = vf_grid_size(state.viewfinder_zoom_level);
            let cx = state.viewfinder_offset.0 + old_c as f32 / 2.0;
            let cy = state.viewfinder_offset.1 + old_r as f32 / 2.0;
            let max_x = (ts_cols - new_c).max(0) as f32;
            let max_y = (ts_rows - new_r).max(0) as f32;
            state.viewfinder_offset.0 = (cx - new_c as f32 / 2.0).clamp(0.0, max_x);
            state.viewfinder_offset.1 = (cy - new_r as f32 / 2.0).clamp(0.0, max_y);
            state.viewfinder_snap_anim = None;
        }
    } else if dist > 12.0 {
        state.viewfinder_pinch_dist = Some(dist);
    }
}

fn tick_snap_animation(state: &mut AppState) {
    let Some(anim) = state.viewfinder_snap_anim else {
        return;
    };
    let progress = ((get_time() - anim.start_time) / SNAP_DURATION).min(1.0) as f32;
    let e = ease_out_cubic(progress);
    state.viewfinder_offset = (
        anim.from.0 + (anim.to.0 - anim.from.0) * e,
        anim.from.1 + (anim.to.1 - anim.from.1) * e,
    );
    if progress >= 1.0 {
        state.viewfinder_offset = anim.to;
        state.viewfinder_snap_anim = None;
    }
}

fn tap_select_tile(
    state: &mut AppState,
    cell_w: f32,
    cell_h: f32,
    ts_cols: i32,
    first_gid: u32,
    tile_ids: &[u32],
) {
    let (mx, my) = mouse_position();
    let palette_y = state.safe_inset_top + 56.0;
    let local_x = mx;
    let local_y = my - palette_y;
    if local_x < 0.0 || local_y < 0.0 {
        return;
    }
    let off = state.viewfinder_offset;
    let col = (local_x / cell_w + off.0).floor() as i32;
    let row = (local_y / cell_h + off.1).floor() as i32;
    if col < 0 || col >= ts_cols || row < 0 {
        return;
    }
    let idx = (row * ts_cols + col) as usize;
    if let Some(&lid) = tile_ids.get(idx) {
        state.selected_gid = first_gid + lid;
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn active_tileset_info(state: &AppState) -> Option<(i32, i32, Vec<u32>, u32, usize)> {
    let session = state.session.as_ref()?;
    let ts = session.document().map.tilesets.get(state.active_tileset)?;
    let coi = ts.tileset.columns == 0 && !ts.tileset.tile_images.is_empty();
    let (cols, ids) = if coi {
        let mut ids: Vec<u32> = ts.tileset.tile_images.keys().copied().collect();
        ids.sort();
        ((ids.len() as f32).sqrt().ceil().max(1.0) as i32, ids)
    } else {
        (
            ts.tileset.columns.max(1) as i32,
            (0..ts.tileset.tile_count).collect(),
        )
    };
    let rows = (ids.len() as i32 + cols - 1) / cols;
    Some((cols, rows, ids, ts.first_gid, state.active_tileset))
}

// ── tile texture cropping (shared with tilesets.rs) ──────────────────────────

pub(crate) fn crop_tile_texture(state: &mut AppState, tile: &PaletteTile) -> Option<Texture2D> {
    if let Some(cached) = state.tile_chip_cache.get(&tile.gid) {
        return Some(cached.texture.clone());
    }
    let session = state.session.as_ref()?;
    let tile_ref = session.document().map.tile_reference_for_gid(tile.gid)?;
    let ts = &tile_ref.tileset.tileset;

    // Collection-of-images: each tile has its own texture
    if let Some(ind_tex) = state
        .tile_textures
        .get(&(tile.tileset_index, tile.local_id))
    {
        let tw = ind_tex.width();
        let th = ind_tex.height();
        let chip_size = 40.0;
        let scale = (chip_size / tw).min(chip_size / th);
        let rw = tw * scale;
        let rh = th * scale;
        let ox = (chip_size - rw) / 2.0;
        let oy = (chip_size - rh) / 2.0;
        let rt = render_target(chip_size as u32, chip_size as u32);
        rt.texture.set_filter(FilterMode::Nearest);
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, chip_size, chip_size));
        cam.render_target = Some(rt.clone());
        set_camera(&cam);
        clear_background(MacroquadColor::from_rgba(0x10, 0x11, 0x13, 255));
        draw_texture_ex(
            ind_tex,
            ox,
            oy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(rw, rh)),
                flip_y: true,
                ..Default::default()
            },
        );
        set_default_camera();
        let tex = rt.texture.clone();
        state.tile_chip_cache.insert(tile.gid, rt);
        return Some(tex);
    }

    // Standard sprite-sheet tileset
    let texture = state.tileset_textures.get(&tile.tileset_index)?;
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
            flip_y: true,
            ..Default::default()
        },
    );

    set_default_camera();
    let tex = rt.texture.clone();
    state.tile_chip_cache.insert(tile.gid, rt);
    Some(tex)
}

/// Returns a chip texture with a blue selection border baked in.
/// Uses render-to-texture (same path as `crop_tile_texture`) which is proven
/// to work on Android, unlike `.border()` or `background_color` rectangles.
fn selected_tile_texture(state: &mut AppState, tile: &PaletteTile) -> Option<Texture2D> {
    if let Some((cached_gid, ref rt)) = state.selected_chip_rt
        && cached_gid == tile.gid
    {
        return Some(rt.texture.clone());
    }

    let session = state.session.as_ref()?;
    let tile_ref = session.document().map.tile_reference_for_gid(tile.gid)?;
    let ts = &tile_ref.tileset.tileset;

    // Resolve texture source + crop rect
    let (tw, th, draw_params) = if let Some(ind) = state
        .tile_textures
        .get(&(tile.tileset_index, tile.local_id))
    {
        let w = ind.width();
        let h = ind.height();
        (w, h, (ind.clone(), None))
    } else {
        let texture = state.tileset_textures.get(&tile.tileset_index)?;
        let cols = ts.columns.max(1);
        let tw = ts.tile_width as f32;
        let th = ts.tile_height as f32;
        let sx = (tile.local_id % cols) as f32 * tw;
        let sy = (tile.local_id / cols) as f32 * th;
        (tw, th, (texture.clone(), Some(Rect::new(sx, sy, tw, th))))
    };

    let chip_size = 40.0;
    let border = 3.0;
    let inner = chip_size - border * 2.0;
    let scale = (inner / tw).min(inner / th);
    let rw = tw * scale;
    let rh = th * scale;
    let ox = (chip_size - rw) / 2.0;
    let oy = (chip_size - rh) / 2.0;

    let rt = render_target(chip_size as u32, chip_size as u32);
    rt.texture.set_filter(FilterMode::Nearest);
    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, chip_size, chip_size));
    cam.render_target = Some(rt.clone());
    set_camera(&cam);

    clear_background(MacroquadColor::from_rgba(10, 133, 255, 255));
    draw_rectangle(
        border,
        border,
        inner,
        inner,
        MacroquadColor::from_rgba(0x10, 0x11, 0x13, 255),
    );
    draw_texture_ex(
        &draw_params.0,
        ox,
        oy,
        WHITE,
        DrawTextureParams {
            source: draw_params.1,
            dest_size: Some(Vec2::new(rw, rh)),
            flip_y: true,
            ..Default::default()
        },
    );

    set_default_camera();
    let tex = rt.texture.clone();
    state.selected_chip_rt = Some((tile.gid, rt));
    Some(tex)
}
