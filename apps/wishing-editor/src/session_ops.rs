use std::collections::BTreeMap;

use wishing_core::EditorSession;

#[cfg(target_os = "android")]
use crate::platform::log_path;
use crate::{app_state::AppState, platform::log};
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use crate::embedded_samples::{embedded_sample, embedded_samples};
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use crate::{demo::load_embedded_demo_session, platform::EMBEDDED_DEMO_MAP_PATH};
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use wishing_core::Layer;

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
        log(format!("boot: requested embedded demo map from web preview: {requested}"));
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
        log(format!("boot: requested embedded demo map from android app: {requested}"));
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
    state.image_cache = image_cache;
    state.session = Some(session);
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
