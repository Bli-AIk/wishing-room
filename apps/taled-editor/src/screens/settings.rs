use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::{PlyTheme, ThemeChoice};

use super::widgets::{bottom_nav, dashboard_nav_items, page_header, section_label};

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let title = l10n::text(state.resolved_language(), "settings-screen-title");
    page_header(
        ui,
        theme,
        &title,
        Some(("← Back", MobileScreen::Dashboard)),
        None,
        state,
    );

    // Content
    ui.element()
        .id("settings-content")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((8, 16, 8, 16)).gap(4))
        .overflow(|o| {
            o.scroll_y().scrollbar(|s| {
                s.width(3.0)
                    .thumb_color(theme.border_strong)
                    .track_color(theme.surface)
                    .hide_after_frames(120)
            })
        })
        .children(|ui| {
            section_label(ui, theme, "Theme");

            let choices: [(ThemeChoice, &str); 8] = [
                (ThemeChoice::System, "System"),
                (ThemeChoice::Dark, "Dark"),
                (ThemeChoice::Light, "Light"),
                (ThemeChoice::CatppuccinLatte, "Catppuccin Latte"),
                (ThemeChoice::CatppuccinFrappe, "Catppuccin Frappé"),
                (ThemeChoice::CatppuccinMacchiato, "Catppuccin Macchiato"),
                (ThemeChoice::CatppuccinMocha, "Catppuccin Mocha"),
                (ThemeChoice::Custom, "Custom"),
            ];

            for (i, (choice, label)) in choices.iter().enumerate() {
                let is_active = state.theme_choice == *choice;
                let choice_val = *choice;
                let bg = if is_active {
                    theme.accent_soft
                } else {
                    theme.surface
                };
                let text_color = if is_active { theme.accent } else { theme.text };

                ui.element()
                    .id(("theme-choice", i as u32))
                    .width(grow!())
                    .height(fixed!(44.0))
                    .background_color(bg)
                    .corner_radius(8.0)
                    .layout(|l| {
                        l.direction(LeftToRight)
                            .align(Left, CenterY)
                            .padding((0, 16, 0, 16))
                    })
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            state.theme_choice = choice_val;
                        }
                        ui.text(label, |t| t.font_size(15).color(text_color));
                        ui.element().width(grow!()).height(fixed!(1.0)).empty();
                        if is_active {
                            ui.text("✓", |t| t.font_size(16).color(theme.accent));
                        }
                    });
            }

            ui.element().width(grow!()).height(fixed!(16.0)).empty();

            section_label(ui, theme, "Grid");

            let grid_label = if state.show_grid {
                "Hide Grid"
            } else {
                "Show Grid"
            };
            ui.element()
                .id("toggle-grid")
                .width(grow!())
                .height(fixed!(44.0))
                .background_color(theme.surface)
                .corner_radius(8.0)
                .layout(|l| {
                    l.direction(LeftToRight)
                        .align(Left, CenterY)
                        .padding((0, 16, 0, 16))
                })
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.show_grid = !state.show_grid;
                    }
                    ui.text(grid_label, |t| t.font_size(15).color(theme.text));
                });

            ui.element().width(grow!()).height(fixed!(16.0)).empty();

            section_label(ui, theme, "About");

            ui.element()
                .id("about-btn")
                .width(grow!())
                .height(fixed!(44.0))
                .background_color(theme.surface)
                .corner_radius(8.0)
                .layout(|l| l.align(Left, CenterY).padding((0, 16, 0, 16)))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.navigate(MobileScreen::About);
                    }
                    ui.text("About Taled", |t| t.font_size(15).color(theme.text));
                });
        });

    let items = dashboard_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Settings);
}
