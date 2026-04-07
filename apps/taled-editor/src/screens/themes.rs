use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::theme::{PALETTES, PlyTheme};

use super::widgets::page_header;

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    page_header(
        ui,
        theme,
        "Themes",
        Some(("← Back", MobileScreen::Settings)),
        None,
        state,
    );

    ui.element()
        .id("themes-content")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(8).padding((16, 16, 16, 16)))
        .overflow(|o| o.scroll_y())
        .children(|ui| {
            for (i, palette_fn) in PALETTES.iter().enumerate() {
                let palette = palette_fn();
                ui.element()
                    .id(("theme-preview", i as u32))
                    .width(grow!())
                    .height(fixed!(60.0))
                    .background_color(palette.background)
                    .corner_radius(8.0)
                    .layout(|l| {
                        l.direction(LeftToRight)
                            .align(Left, CenterY)
                            .padding((0, 16, 0, 16))
                            .gap(8)
                    })
                    .children(|ui| {
                        ui.text(&palette.name, |t| t.font_size(15).color(palette.text));
                        ui.element().width(grow!()).height(fixed!(1.0)).empty();
                        let colors = [palette.accent, palette.surface, palette.border];
                        for (j, c) in colors.iter().enumerate() {
                            ui.element()
                                .id(("swatch", (i * 10 + j) as u32))
                                .width(fixed!(20.0))
                                .height(fixed!(20.0))
                                .background_color(*c)
                                .corner_radius(10.0)
                                .empty();
                        }
                    });
            }
        });
}
