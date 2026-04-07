use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::PlyTheme;

use super::widgets::page_header;

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let title = l10n::text(state.resolved_language(), "property-editor-title");
    page_header(
        ui,
        theme,
        &title,
        Some(("← Back", MobileScreen::Editor)),
        None,
        state,
    );

    ui.element()
        .id("properties-content")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((16, 16, 16, 16)))
        .children(|ui| {
            ui.text("Property editor", |t| {
                t.font_size(14).color(theme.muted_text)
            });
        });
}
