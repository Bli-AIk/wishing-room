use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::screens;
use crate::theme::PlyTheme;

/// Main UI render function called each frame.
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    ui.element()
        .id("root")
        .width(grow!())
        .height(grow!())
        .background_color(theme.background)
        .layout(|l| l.direction(TopToBottom))
        .children(|ui| match state.mobile_screen {
            MobileScreen::Dashboard => {
                screens::dashboard::render(ui, state, theme);
            }
            MobileScreen::Editor => {
                screens::editor::render(ui, state, theme);
            }
            MobileScreen::Tilesets => {
                screens::tilesets::render(ui, state, theme);
            }
            MobileScreen::Layers => {
                screens::layers::render(ui, state, theme);
            }
            MobileScreen::Objects => {
                screens::objects::render(ui, state, theme);
            }
            MobileScreen::Properties => {
                screens::properties::render(ui, state, theme);
            }
            MobileScreen::Settings => {
                screens::settings::render(ui, state, theme);
            }
            MobileScreen::Themes => {
                screens::themes::render(ui, state, theme);
            }
            MobileScreen::About => {
                screens::about::render(ui, state, theme);
            }
        });
}
