use std::collections::BTreeMap;

use taled_core::EditorSession;

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use crate::embedded_samples::{embedded_sample, embedded_samples};
#[cfg(target_os = "android")]
use crate::platform::log_path;
use crate::{app_state::AppState, platform::log};
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use crate::{demo::load_embedded_demo_session, platform::EMBEDDED_DEMO_MAP_PATH};
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use taled_core::Layer;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub(crate) fn open_document(state: &mut AppState) {
    match EditorSession::load(&state.path_input) {
        Ok(session) => {
            state.status = format!("Opened {}.", state.path_input);
            state.save_as_input = state.path_input.clone();
            install_session(state, session);
        }
        Err(error) => state.status = format!("Open failed: {error}"),
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn open_document(state: &mut AppState) {
    let requested = state.path_input.trim().to_string();
    if requested.is_empty() {
        load_sample(state);
        return;
    }
    if embedded_sample(&requested).is_some() {
        log(format!(
            "boot: requested embedded demo map from web preview: {requested}"
        ));
        load_embedded_sample(state, &requested);
        return;
    }

    log(format!(
        "boot: rejected web open request for unsupported path {requested}"
    ));
    state.status = format!(
        "Web preview only ships embedded samples: {}.",
        embedded_sample_paths()
    );
}

#[cfg(target_os = "android")]
pub(crate) fn open_document(state: &mut AppState) {
    let requested = state.path_input.trim().to_string();
    if requested.is_empty() {
        load_sample(state);
        return;
    }
    if embedded_sample(&requested).is_some() {
        log(format!(
            "boot: requested embedded demo map from android app: {requested}"
        ));
        load_embedded_sample(state, &requested);
        return;
    }

    log(format!(
        "boot: rejected android open request for unsupported path {requested}"
    ));
    state.status = format!(
        "Android build only ships embedded samples: {}.",
        embedded_sample_paths()
    );
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub(crate) fn load_sample(state: &mut AppState) {
    state.path_input = std::env::current_dir()
        .ok()
        .map(EditorSession::sample_path_from_root)
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    open_document(state);
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn load_sample(state: &mut AppState) {
    load_embedded_sample(state, EMBEDDED_DEMO_MAP_PATH);
}

#[cfg(target_os = "android")]
pub(crate) fn load_sample(state: &mut AppState) {
    load_embedded_sample(state, EMBEDDED_DEMO_MAP_PATH);
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub(crate) fn load_embedded_sample(state: &mut AppState, path: &str) {
    state.path_input = path.to_string();
    state.save_as_input = path.to_string();
    log("boot: starting embedded demo load");
    match load_embedded_demo_session(path) {
        Ok(session) => {
            install_session(state, session);
            state.status = embedded_loaded_status(path);
            log("boot: embedded demo load completed");
        }
        Err(error) => {
            state.status = format!("Embedded demo load failed: {error}");
            log(format!("boot: embedded demo load failed: {error}"));
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub(crate) fn load_embedded_sample(state: &mut AppState, path: &str) {
    state.path_input = path.to_string();
    state.status = "Embedded sample loading is only wired for web/android previews.".to_string();
}

#[cfg(target_arch = "wasm32")]
fn embedded_loaded_status(path: &str) -> String {
    let label = embedded_sample(path).map_or(path, |sample| sample.title);
    format!("Loaded embedded sample {label} ({path}).")
}

#[cfg(target_os = "android")]
fn embedded_loaded_status(path: &str) -> String {
    let label = embedded_sample(path).map_or(path, |sample| sample.title);
    let log_path = log_path().unwrap_or_default();
    format!("Loaded embedded sample {label} ({path}). Logs: {log_path}")
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
fn embedded_sample_paths() -> String {
    embedded_samples()
        .iter()
        .map(|sample| sample.path)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn adjust_zoom(state: &mut AppState, delta: i32) {
    state.zoom_percent = (state.zoom_percent + delta).clamp(25, 400);
}

pub(crate) fn adjust_zoom_around_view_center(state: &mut AppState, delta: i32) {
    if state.session.is_none() {
        return;
    }
    let Some((host_width, host_height)) = canvas_host_size_or_default(state) else {
        return;
    };

    let current_zoom = f64::from(state.zoom_percent) / 100.0;
    let new_zoom_percent = (state.zoom_percent + delta).clamp(25, 400);
    let new_zoom = f64::from(new_zoom_percent) / 100.0;
    let center_x = host_width * 0.5;
    let center_y = host_height * 0.5;
    let world_center_x = (center_x - f64::from(state.pan_x)) / current_zoom;
    let world_center_y = (center_y - f64::from(state.pan_y)) / current_zoom;

    state.zoom_percent = new_zoom_percent;
    state.pan_x = (center_x - world_center_x * new_zoom).round() as i32;
    state.pan_y = (center_y - world_center_y * new_zoom).round() as i32;
    state.status = format!("Zoom {}%.", state.zoom_percent);
}

pub(crate) fn animate_camera_to_center(state: &mut AppState) {
    let Some((target_pan_x, target_pan_y)) = centered_pan_for_current_zoom(state) else {
        return;
    };
    state.pan_x = target_pan_x;
    state.pan_y = target_pan_y;
    state.camera_transition_active = true;
    state.status = "Centered camera.".to_string();
}

pub(crate) fn animate_camera_to_fit_map(state: &mut AppState) {
    let Some((target_zoom_percent, target_pan_x, target_pan_y)) = fit_map_view(state) else {
        return;
    };
    state.zoom_percent = target_zoom_percent;
    state.pan_x = target_pan_x;
    state.pan_y = target_pan_y;
    state.camera_transition_active = true;
    state.status = format!("Fit map to view at {}%.", state.zoom_percent);
}

pub(crate) fn save_document(state: &mut AppState) {
    let Some(session) = state.session.as_mut() else {
        state.status = "Nothing to save.".to_string();
        return;
    };

    match session.save() {
        Ok(()) => state.status = format!("Saved {}.", session.document().file_path.display()),
        Err(error) => state.status = format!("Save failed: {error}"),
    }
}

pub(crate) fn save_as_document(state: &mut AppState) {
    let Some(session) = state.session.as_mut() else {
        state.status = "Nothing to save.".to_string();
        return;
    };

    match session.save_as(&state.save_as_input) {
        Ok(()) => {
            state.path_input = state.save_as_input.clone();
            state.status = format!("Saved as {}.", state.save_as_input);
        }
        Err(error) => state.status = format!("Save-as failed: {error}"),
    }
}

fn install_session(state: &mut AppState, session: EditorSession) {
    let mut image_cache = BTreeMap::new();
    for (index, _) in session.document().map.tilesets.iter().enumerate() {
        match session.tileset_image_data_uri(index) {
            Ok(uri) => {
                image_cache.insert(index, uri);
            }
            Err(error) => {
                state.status = format!("Loaded map, but image cache failed: {error}");
                log(format!(
                    "boot: image cache failed for tileset {index}: {error}"
                ));
            }
        }
    }

    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    log_session_summary(&session, image_cache.len());

    let selected_gid = default_selected_gid(&session);
    state.active_layer = 0;
    state.selected_gid = selected_gid;
    state.selected_cell = None;
    state.selected_object = None;
    state.layers_panel_expanded = false;
    state.zoom_percent = 100;
    let (default_pan_x, default_pan_y) = default_mobile_center_pan(&session, state.zoom_percent);
    state.pan_x = default_pan_x;
    state.pan_y = default_pan_y;
    state.pending_canvas_center = true;
    state.camera_transition_active = false;
    state.active_touch_points.clear();
    state.single_touch_gesture = None;
    state.pinch_gesture = None;
    state.suppress_click_until = None;
    state.canvas_host_scroll_offset = (0.0, 0.0);
    state.canvas_host_size = None;
    state.image_cache = image_cache;
    state.session = Some(session);
}

fn default_mobile_center_pan(session: &EditorSession, zoom_percent: i32) -> (i32, i32) {
    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    {
        const DEFAULT_HOST_WIDTH: f64 = 384.0;
        const DEFAULT_HOST_HEIGHT: f64 = 241.0;

        let map = &session.document().map;
        let zoom = f64::from(zoom_percent) / 100.0;
        let map_width = f64::from(map.total_pixel_width()) * zoom;
        let map_height = f64::from(map.total_pixel_height()) * zoom;
        let pan_x = ((DEFAULT_HOST_WIDTH - map_width) * 0.5).round() as i32;
        let pan_y = ((DEFAULT_HOST_HEIGHT - map_height) * 0.5).round() as i32;
        return (pan_x, pan_y);
    }

    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    {
        let _ = (session, zoom_percent);
        (0, 0)
    }
}

fn centered_pan_for_current_zoom(state: &AppState) -> Option<(i32, i32)> {
    let session = state.session.as_ref()?;
    let (host_width, host_height) = canvas_host_size_or_default(state)?;
    let map = &session.document().map;
    let zoom = f64::from(state.zoom_percent) / 100.0;
    let map_width = f64::from(map.total_pixel_width()) * zoom;
    let map_height = f64::from(map.total_pixel_height()) * zoom;
    let pan_x = ((host_width - map_width) * 0.5).round() as i32;
    let pan_y = ((host_height - map_height) * 0.5).round() as i32;
    Some((pan_x, pan_y))
}

fn fit_map_view(state: &AppState) -> Option<(i32, i32, i32)> {
    let session = state.session.as_ref()?;
    let (host_width, host_height) = canvas_host_size_or_default(state)?;
    let map = &session.document().map;
    let map_width = f64::from(map.total_pixel_width()).max(1.0);
    let map_height = f64::from(map.total_pixel_height()).max(1.0);
    let fit_scale = (host_width / map_width).min(host_height / map_height);
    let target_zoom_percent = (fit_scale * 100.0).floor() as i32;
    let target_zoom_percent = target_zoom_percent.clamp(25, 400);
    let zoom = f64::from(target_zoom_percent) / 100.0;
    let scaled_width = map_width * zoom;
    let scaled_height = map_height * zoom;
    let pan_x = ((host_width - scaled_width) * 0.5).round() as i32;
    let pan_y = ((host_height - scaled_height) * 0.5).round() as i32;
    Some((target_zoom_percent, pan_x, pan_y))
}

fn canvas_host_size_or_default(state: &AppState) -> Option<(f64, f64)> {
    if let Some(size) = state.canvas_host_size {
        return Some(size);
    }

    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    {
        return Some((384.0, 241.0));
    }

    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    {
        let _ = state;
        None
    }
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
fn log_session_summary(session: &EditorSession, image_cache_len: usize) {
    let document = session.document();
    let map = &document.map;
    let tile_count: usize = map
        .layers
        .iter()
        .filter_map(Layer::as_tile)
        .map(|layer| layer.tiles.iter().filter(|gid| **gid != 0).count())
        .sum();
    let object_count: usize = map
        .layers
        .iter()
        .filter_map(Layer::as_object)
        .map(|layer| layer.objects.len())
        .sum();
    let width_px = map.total_pixel_width();
    let height_px = map.total_pixel_height();

    log(format!(
        "boot: map loaded path={} size={}x{} tiles surface={}x{} px layers={} tilesets={} painted_tiles={} objects={} cached_images={}",
        document.file_path.display(),
        map.width,
        map.height,
        width_px,
        height_px,
        map.layers.len(),
        map.tilesets.len(),
        tile_count,
        object_count,
        image_cache_len,
    ));

    if width_px > 2048 || height_px > 2048 {
        log(format!(
            "boot: warning surface {}x{} px exceeds 2048 on at least one axis; this matches the suspected Android rendering limit on your device",
            width_px, height_px,
        ));
    }
}

fn default_selected_gid(session: &EditorSession) -> u32 {
    session
        .document()
        .map
        .tilesets
        .iter()
        .find(|tileset| tileset.tileset.name != "collision" && tileset.tileset.tile_count > 1)
        .or_else(|| session.document().map.tilesets.first())
        .map(|tileset| tileset.first_gid)
        .unwrap_or(0)
}
