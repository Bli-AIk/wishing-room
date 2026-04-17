use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::workspace;

const THUMB_SIZE: u32 = 120;
const THUMBS_DIR: &str = ".thumbs";

thread_local! {
    static THUMB_CACHE: RefCell<HashMap<String, Option<Texture2D>>> =
        RefCell::new(HashMap::new());
}

/// Return the thumbnail path for a given TMX file inside a workspace.
fn thumb_path_for(workspace_name: &str, tmx_file_name: &str) -> Option<PathBuf> {
    let root = workspace::workspaces_root()?;
    let dir = root.join(workspace_name).join(THUMBS_DIR);
    let stem = Path::new(tmx_file_name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    Some(dir.join(format!("{stem}.png")))
}

/// Clear the thumbnail cache (call when switching workspaces).
pub(crate) fn invalidate_cache() {
    THUMB_CACHE.with(|c| c.borrow_mut().clear());
}

/// Try to load a cached thumbnail texture for a workspace map.
/// Returns `Some(texture)` if a thumbnail exists, `None` otherwise.
/// Results are cached in memory to avoid repeated disk reads.
pub(crate) fn get_thumb(workspace_name: &str, tmx_file_name: &str) -> Option<Texture2D> {
    let key = format!("{workspace_name}/{tmx_file_name}");
    THUMB_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
        let result = load_from_disk(workspace_name, tmx_file_name);
        cache.insert(key, result.clone());
        result
    })
}

/// Generate a thumbnail for the currently loaded map and save it.
/// Should be called once the tile map has been rendered at least once.
pub(crate) fn generate_and_save(state: &mut AppState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let map = &session.document().map;
    let map_px_w = map.total_pixel_width() as f32;
    let map_px_h = map.total_pixel_height() as f32;
    if map_px_w < 1.0 || map_px_h < 1.0 {
        return;
    }
    let tile_w = map.tile_width as f32;
    let tile_h = map.tile_height as f32;

    // Determine the workspace and file name from the session's file path.
    let file_path = &session.document().file_path;
    let tmx_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let ws_name = &state.active_workspace;
    let Some(save_path) = thumb_path_for(ws_name, &tmx_name) else {
        return;
    };

    // Calculate thumbnail dimensions preserving aspect ratio.
    let scale = (THUMB_SIZE as f32 / map_px_w).min(THUMB_SIZE as f32 / map_px_h);
    let thumb_w = (map_px_w * scale).ceil() as u32;
    let thumb_h = (map_px_h * scale).ceil() as u32;
    if thumb_w == 0 || thumb_h == 0 {
        return;
    }

    // Render the map to a small render target (all layers, no highlights).
    let rt = render_target(thumb_w, thumb_h);
    rt.texture.set_filter(FilterMode::Nearest);
    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, map_px_w, map_px_h));
    cam.render_target = Some(rt.clone());
    set_camera(&cam);
    clear_background(MacroquadColor::from_rgba(0, 0, 0, 0));

    render_all_tile_layers(
        map,
        &state.tileset_textures,
        &state.tile_textures,
        tile_w,
        tile_h,
    );

    set_default_camera();

    // Read pixels and save as PNG.
    let image = rt.texture.get_texture_data();
    if let Some(parent) = save_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    image.export_png(&save_path.to_string_lossy());
    invalidate_cache();
    crate::logging::append(&format!("thumbnail saved: {}", save_path.display()));
}

fn load_from_disk(workspace_name: &str, tmx_file_name: &str) -> Option<Texture2D> {
    let path = thumb_path_for(workspace_name, tmx_file_name)?;
    if !path.exists() {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    let tex = Texture2D::from_file_with_format(&bytes, None);
    tex.set_filter(FilterMode::Nearest);
    Some(tex)
}

/// Render all visible tile layers (simplified, no active-layer highlight).
fn render_all_tile_layers(
    map: &taled_core::Map,
    tileset_textures: &BTreeMap<usize, Texture2D>,
    tile_textures: &BTreeMap<(usize, u32), Texture2D>,
    tile_w: f32,
    tile_h: f32,
) {
    for layer in &map.layers {
        if !layer.visible() {
            continue;
        }
        let Some(tile_layer) = layer.as_tile() else {
            continue;
        };
        let alpha = layer.opacity();
        let color = MacroquadColor::new(1.0, 1.0, 1.0, alpha);
        render_thumb_layer(
            map,
            tileset_textures,
            tile_textures,
            tile_layer,
            tile_w,
            tile_h,
            color,
        );
    }
}

fn render_thumb_layer(
    map: &taled_core::Map,
    tileset_textures: &BTreeMap<usize, Texture2D>,
    tile_textures: &BTreeMap<(usize, u32), Texture2D>,
    tile_layer: &taled_core::TileLayer,
    tile_w: f32,
    tile_h: f32,
    color: MacroquadColor,
) {
    for row in 0..map.height {
        for col in 0..map.width {
            let idx = (row * map.width + col) as usize;
            let gid = tile_layer.tiles.get(idx).copied().unwrap_or(0);
            if gid != 0 {
                draw_thumb_tile(
                    map,
                    tileset_textures,
                    tile_textures,
                    gid,
                    col,
                    row,
                    tile_w,
                    tile_h,
                    color,
                );
            }
        }
    }
}

fn draw_thumb_tile(
    map: &taled_core::Map,
    tileset_textures: &BTreeMap<usize, Texture2D>,
    tile_textures: &BTreeMap<(usize, u32), Texture2D>,
    gid: u32,
    col: u32,
    row: u32,
    tile_w: f32,
    tile_h: f32,
    color: MacroquadColor,
) {
    let Some(tile_ref) = map.tile_reference_for_gid(gid) else {
        return;
    };
    let dx = col as f32 * tile_w;
    let dy = row as f32 * tile_h;
    let ts_idx = tile_ref.tileset_index;
    let (flip_h, flip_v, flip_d) = taled_core::tile_flip_flags(gid);
    let (rotation, flip_x, flip_y) = crate::canvas::tile_transform(flip_h, flip_v, flip_d);
    let pivot = Some(Vec2::new(dx + tile_w / 2.0, dy + tile_h / 2.0));

    // Collection-of-images tile
    if let Some(tex) = tile_textures.get(&(ts_idx, tile_ref.local_id)) {
        draw_texture_ex(
            tex,
            dx,
            dy,
            color,
            DrawTextureParams {
                dest_size: Some(Vec2::new(tile_w, tile_h)),
                rotation,
                flip_x,
                flip_y,
                pivot,
                ..Default::default()
            },
        );
        return;
    }

    let Some(texture) = tileset_textures.get(&ts_idx) else {
        return;
    };
    let ts = &tile_ref.tileset.tileset;
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
            dest_size: Some(Vec2::new(tile_w, tile_h)),
            rotation,
            flip_x,
            flip_y,
            pivot,
        },
    );
}
