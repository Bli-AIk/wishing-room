use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items, page_header};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let title = l10n::text(lang, "properties-title");
    let back = l10n::text(lang, "common-back");
    let done = l10n::text(lang, "common-done");
    page_header(
        ui,
        theme,
        &title,
        Some((&back, MobileScreen::Editor)),
        Some((&done, MobileScreen::Editor)),
        state,
    );

    ui.element()
        .id("props-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY))
        .children(|ui| {
            let msg = l10n::text(lang, "screen-not-implemented");
            ui.text(&msg, |t| t.font_size(16).color(theme.muted_text));
        });

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Properties);
}
