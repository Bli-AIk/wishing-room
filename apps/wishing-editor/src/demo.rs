#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use crate::{
    embedded_samples::{embedded_sample_assets, embedded_sample},
    platform::log,
};
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use wishing_core::EditorSession;

#[cfg(target_arch = "wasm32")]
pub(crate) fn load_embedded_demo_session(path: &str) -> wishing_core::Result<EditorSession> {
    let label = embedded_sample(path).map_or(path, |sample| sample.title);
    log(format!("boot: loading embedded demo map {path} ({label})"));
    EditorSession::load_embedded(path, embedded_sample_assets())
}

#[cfg(target_os = "android")]
pub(crate) fn load_embedded_demo_session(path: &str) -> wishing_core::Result<EditorSession> {
    let label = embedded_sample(path).map_or(path, |sample| sample.title);
    log(format!("boot: loading embedded demo map {path} ({label})"));
    EditorSession::load_embedded(path, embedded_sample_assets())
}
