use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items, page_header};

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let title = l10n::text(state.resolved_language(), "nav-tilesets");
    page_header(
        ui,
        theme,
        &title,
        Some(("← Back", MobileScreen::Editor)),
        None,
        state,
    );

    ui.element()
        .id("tilesets-content")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(8).padding((8, 16, 8, 16)))
        .overflow(|o| o.scroll_y())
        .children(|ui| {
            let Some(session) = state.session.as_ref() else {
                ui.text("No map loaded", |t| t.font_size(14).color(theme.muted_text));
                return;
            };
            let map = &session.document().map;

            for (i, ts_ref) in map.tilesets.iter().enumerate() {
                let ts = &ts_ref.tileset;

                ui.element()
                    .id(("tileset-card", i as u32))
                    .width(grow!())
                    .height(fit!(min: 64.0))
                    .background_color(theme.surface)
                    .corner_radius(8.0)
                    .layout(|l| l.direction(TopToBottom).gap(4).padding((8, 12, 8, 12)))
                    .children(|ui| {
                        ui.text(&ts.name, |t| t.font_size(15).color(theme.text));
                        let rows = ts.tile_count / ts.columns.max(1);
                        let info = format!(
                            "{}×{} tiles, {}×{} px each",
                            ts.columns, rows, ts.tile_width, ts.tile_height,
                        );
                        ui.text(&info, |t| t.font_size(12).color(theme.muted_text));

                        // Show tileset image if available
                        if let Some(texture) = state.tileset_textures.get(&i) {
                            ui.element()
                                .id(("ts-img", i as u32))
                                .width(grow!())
                                .height(fixed!(120.0))
                                .image(texture.clone())
                                .corner_radius(4.0)
                                .empty();
                        }
                    });
            }
        });

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Tilesets);
}
