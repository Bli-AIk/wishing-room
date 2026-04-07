use ply_engine::prelude::*;
use taled_core::Layer;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items, page_header};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let title = l10n::text(state.resolved_language(), "nav-objects");
    page_header(
        ui,
        theme,
        &title,
        Some(("← Back", MobileScreen::Editor)),
        None,
        state,
    );

    ui.element()
        .id("objects-content")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(8).padding((16, 16, 16, 16)))
        .children(|ui| {
            let Some(session) = state.session.as_ref() else {
                ui.text("No map loaded", |t| t.font_size(14).color(theme.muted_text));
                return;
            };
            let map = &session.document().map;
            let object_count: usize = map
                .layers
                .iter()
                .filter_map(Layer::as_object)
                .map(|ol| ol.objects.len())
                .sum();
            ui.text(&format!("{} objects in this map", object_count), |t| {
                t.font_size(14).color(theme.muted_text)
            });
        });

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Objects);
}
