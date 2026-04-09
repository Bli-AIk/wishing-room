mod app_state;
mod canvas;
mod edit_ops;
mod embedded_samples;
mod icons;
mod l10n;
mod platform;
mod screens;
mod session_ops;
#[cfg(feature = "system-fonts")]
mod system_font;
mod theme;
mod touch_ops;
mod ui;

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
            sample_count: 4,
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
    let mut ply = Ply::<()>::new(resolve_font()).await;
    let mut state = AppState::new();
    load_embedded_sample(&mut state);

    loop {
        let theme = PlyTheme::from_choice(state.theme_choice, &state.custom_theme);
        let bg: MacroquadColor = theme.background_elevated.into();
        clear_background(bg);

        if platform::is_back_pressed() {
            state.navigate_back();
        }

        let mut ui = ply.begin();

        ui::render(&mut ui, &mut state, &theme);

        ui.show(|_| {}).await;
        next_frame().await;
    }
}
