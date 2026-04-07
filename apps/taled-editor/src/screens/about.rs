use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::theme::PlyTheme;

use super::widgets::page_header;

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    page_header(
        ui,
        theme,
        "About",
        Some(("← Back", MobileScreen::Settings)),
        None,
        state,
    );

    ui.element()
        .id("about-content")
        .width(grow!())
        .height(grow!())
        .layout(|l| {
            l.direction(TopToBottom)
                .align(CenterX, CenterY)
                .gap(12)
                .padding((32, 32, 32, 32))
        })
        .children(|ui| {
            ui.text("Taled", |t| t.font_size(28).color(theme.text));
            ui.text("Tile Map Editor", |t| {
                t.font_size(14).color(theme.muted_text)
            });
            ui.text("v0.1.0", |t| t.font_size(12).color(theme.muted_text));
            ui.element().width(grow!()).height(fixed!(24.0)).empty();
            ui.text("Built with Ply Engine & taled-core", |t| {
                t.font_size(12).color(theme.muted_text)
            });
            ui.text("MIT License", |t| t.font_size(12).color(theme.muted_text));
        });
}
