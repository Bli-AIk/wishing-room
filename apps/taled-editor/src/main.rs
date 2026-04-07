mod app_state;
mod canvas;
mod edit_ops;
mod embedded_samples;
mod l10n;
mod screens;
mod session_ops;
mod theme;
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

#[macroquad::main(window_conf)]
async fn main() {
    let mut ply = Ply::<()>::new(&DEFAULT_FONT).await;
    let mut state = AppState::new();
    load_embedded_sample(&mut state);

    loop {
        let theme = PlyTheme::from_choice(state.theme_choice, &state.custom_theme);
        clear_background(MacroquadColor::from_rgba(
            theme.background.r as u8,
            theme.background.g as u8,
            theme.background.b as u8,
            255,
        ));

        let mut ui = ply.begin();

        ui::render(&mut ui, &mut state, &theme);

        ui.show(|_| {}).await;
        next_frame().await;
    }
}
