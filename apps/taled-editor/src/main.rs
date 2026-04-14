mod app_state;
mod canvas;
mod canvas_objects;
mod canvas_overlay;
mod edit_ops;
mod embedded_samples;
mod icons;
mod l10n;
mod logging;
mod platform;
mod screens;
mod selection_ops;
mod selection_transform;
mod session_ops;
#[cfg(feature = "system-fonts")]
mod system_font;
mod theme;
mod touch_ops;
mod ui;
mod workspace;

use ply_engine::prelude::*;

use app_state::AppState;
use session_ops::load_embedded_sample;
use theme::PlyTheme;

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "Taled".to_owned(),
            window_width: 384,
            window_height: 688,
            high_dpi: true,
            sample_count: 1,
            platform: miniquad::conf::Platform {
                webgl_version: miniquad::conf::WebGLVersion::WebGL2,
                ..Default::default()
            },
            ..Default::default()
        },
        draw_call_vertex_capacity: 200000,
        draw_call_index_capacity: 200000,
        ..Default::default()
    }
}

static DEFAULT_FONT: FontAsset = FontAsset::Path("assets/fonts/NotoSansCJK-Regular.ttc");

/// Picks the best available CJK font at runtime.
///
/// With the `system-fonts` feature enabled, tries the OS-provided CJK font
/// first (saving ~19 MB of bundled data). Falls back to the embedded asset.
fn resolve_font() -> &'static FontAsset {
    #[cfg(feature = "system-fonts")]
    if let Some(font) = system_font::find_system_cjk_font() {
        return font;
    }
    &DEFAULT_FONT
}

#[macroquad::main(window_conf)]
async fn main() {
    if let Some(dir) = platform::files_dir() {
        logging::init(&dir);
    }
    logging::append(&format!(
        "screen {}x{} dpi={:.1}",
        screen_width() as u32,
        screen_height() as u32,
        macroquad::miniquad::window::dpi_scale(),
    ));

    let mut ply = Ply::<()>::new(resolve_font()).await;
    let mut state = AppState::new();

    // Extract embedded samples into the builtin workspace (idempotent).
    workspace::ensure_builtin_workspace();
    state.workspace_list = workspace::list_workspaces()
        .into_iter()
        .map(|w| w.name)
        .collect();

    load_embedded_sample(&mut state);
    logging::append(&format!("loaded default sample: {}", state.status));

    let mut frame_count: u32 = 0;
    let mut prev_show_ms: f32 = 0.0;
    let mut prev_next_ms: f32 = 0.0;
    let mut safe_inset_queried = false;
    loop {
        let ft0 = get_time();

        let theme = PlyTheme::from_choice(state.theme_choice, &state.custom_theme);
        let bg: MacroquadColor = theme.background_elevated.into();
        clear_background(bg);

        // Query safe area inset once the window is attached (needs first frame).
        if !safe_inset_queried {
            let px = platform::safe_inset_top();
            let dpi = macroquad::miniquad::window::dpi_scale();
            state.safe_inset_top = if dpi > 0.0 { px as f32 / dpi } else { 0.0 };
            logging::append(&format!(
                "safe_inset_top: {}px → {:.1}lp (dpi={:.1})",
                px, state.safe_inset_top, dpi
            ));
            safe_inset_queried = true;
        }

        if platform::is_back_pressed() {
            state.navigate_back();
        }

        // Poll for SAF directory picker result (unconditional — state may
        // have been reset if the Activity was recreated by Android).
        if let Some(result) = platform::poll_import_result() {
            logging::append(&format!("SAF import result received: {result}"));
            workspace::handle_import_result(&mut state, &result);
        }

        let mut ui = ply.begin();

        let ft1 = get_time();

        ui::render(&mut ui, &mut state, &theme);

        let ft2 = get_time();

        // Log perf data every ~0.5 seconds
        frame_count = frame_count.wrapping_add(1);
        if frame_count.is_multiple_of(30) {
            let ms = |a: f64, b: f64| ((b - a) * 1000.0) as f32;
            logging::append(&format!(
                "f={frame_count} fps={} ft={:.1}ms loop=[begin:{:.1} render:{:.1} show:{:.1} next:{:.1}] cvs_n={} perf=[{}] scr={:?}",
                get_fps(),
                get_frame_time() * 1000.0,
                ms(ft0, ft1),
                ms(ft1, ft2),
                prev_show_ms,
                prev_next_ms,
                state.canvas_rebuild_count,
                state.perf_info,
                state.mobile_screen,
            ));
            state.canvas_rebuild_count = 0;
        }

        let ts0 = get_time();
        ui.show(|_| {}).await;
        let ts1 = get_time();
        next_frame().await;
        let ts2 = get_time();
        prev_show_ms = ((ts1 - ts0) * 1000.0) as f32;
        prev_next_ms = ((ts2 - ts1) * 1000.0) as f32;
    }
}
