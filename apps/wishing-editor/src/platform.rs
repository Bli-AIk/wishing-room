#[allow(dead_code)]
pub(crate) const EMBEDDED_DEMO_MAP_PATH: &str = crate::embedded_samples::DEFAULT_EMBEDDED_SAMPLE_PATH;

#[cfg(target_arch = "wasm32")]
pub(crate) fn install() {
    crate::web_diag::install();
}

#[cfg(target_os = "android")]
pub(crate) fn install() {
    crate::android_diag::install();
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub(crate) fn install() {}

#[cfg(target_arch = "wasm32")]
pub(crate) fn log(message: impl Into<String>) {
    crate::web_diag::log(message.into());
}

#[cfg(target_os = "android")]
pub(crate) fn log(message: impl Into<String>) {
    crate::android_diag::log(message.into());
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub(crate) fn log(_message: impl Into<String>) {}

#[cfg(target_arch = "wasm32")]
pub(crate) fn mark_app_rendered() {
    crate::web_diag::mark_app_rendered();
}

#[cfg(target_os = "android")]
pub(crate) fn mark_app_rendered() {
    crate::android_diag::mark_app_rendered();
}

#[cfg(target_os = "android")]
pub(crate) fn log_path() -> Option<String> {
    Some(crate::android_diag::log_path())
}
