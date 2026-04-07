use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items, page_header};

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let title = l10n::text(state.resolved_language(), "nav-layers");
    page_header(
        ui,
        theme,
        &title,
        Some(("← Back", MobileScreen::Editor)),
        None,
        state,
    );

    // Layer list
    ui.element()
        .id("layers-content")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(2).padding((8, 16, 8, 16)))
        .overflow(|o| o.scroll_y())
        .children(|ui| {
            let Some(session) = state.session.as_ref() else {
                ui.text("No map loaded", |t| t.font_size(14).color(theme.muted_text));
                return;
            };
            let map = &session.document().map;

            for (i, layer) in map.layers.iter().enumerate() {
                let is_active = state.active_layer == i;
                let bg = if is_active {
                    theme.accent_soft
                } else {
                    theme.surface
                };

                ui.element()
                    .id(("layer", i as u32))
                    .width(grow!())
                    .height(fixed!(52.0))
                    .background_color(bg)
                    .corner_radius(8.0)
                    .border(|b| b.bottom(1).color(theme.border))
                    .layout(|l| {
                        l.direction(LeftToRight)
                            .align(Left, CenterY)
                            .padding((0, 12, 0, 12))
                            .gap(8)
                    })
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            state.active_layer = i;
                        }

                        // Visibility icon
                        let vis_icon = if layer.visible() { "👁" } else { "—" };
                        ui.text(vis_icon, |t| t.font_size(16).color(theme.muted_text));

                        // Layer name
                        let name = layer.name();
                        let display_name = if name.is_empty() {
                            format!("Layer {}", i)
                        } else {
                            name.to_string()
                        };
                        ui.text(&display_name, |t| t.font_size(15).color(theme.text));

                        ui.element().width(grow!()).height(fixed!(1.0)).empty();

                        // Layer type
                        let kind = if layer.as_tile().is_some() {
                            "Tile"
                        } else {
                            "Object"
                        };
                        ui.text(kind, |t| t.font_size(12).color(theme.muted_text));
                    });
            }
        });

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Layers);
}
