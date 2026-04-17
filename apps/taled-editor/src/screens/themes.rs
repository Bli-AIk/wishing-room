use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::{PlyTheme, ThemeChoice};

use super::widgets::{bottom_nav, dashboard_nav_items, page_header};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let title = l10n::text(lang, "themes-title");
    let back = l10n::text(lang, "common-back");
    page_header(
        ui,
        theme,
        &title,
        Some((&back, MobileScreen::Settings)),
        None,
        state,
    );

    ui.element()
        .id("themes-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(12).padding((14, 14, 0, 14)))
        .overflow(|o| o.scroll_y())
        .children(|ui| {
            // ── Current Theme ──
            caption_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "themes-current-title"),
            );
            current_theme_card(ui, state, theme);

            // ── Built-in ──
            caption_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "themes-built-in-title"),
            );
            theme_grid(
                ui,
                state,
                theme,
                &[
                    &[ThemeChoice::System, ThemeChoice::Dark],
                    &[ThemeChoice::Light],
                ],
            );

            // ── Catppuccin ──
            caption_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "themes-catppuccin-title"),
            );
            theme_grid(
                ui,
                state,
                theme,
                &[
                    &[
                        ThemeChoice::CatppuccinMocha,
                        ThemeChoice::CatppuccinMacchiato,
                    ],
                    &[ThemeChoice::CatppuccinFrappe, ThemeChoice::CatppuccinLatte],
                ],
            );

            // ── Custom ──
            caption_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "themes-custom-title"),
            );
            info_card(
                ui,
                theme,
                &[
                    &l10n::text(state.resolved_language(), "themes-custom-title"),
                    &l10n::text(state.resolved_language(), "themes-custom-description"),
                ],
            );

            // ── JSON Import/Export ──
            caption_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "themes-json-title"),
            );
            json_section(ui, state, theme);

            ui.element().width(grow!()).height(fixed!(20.0)).empty();
        });

    let items = dashboard_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Settings);
}

fn json_section(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    // Wrapping card (matches .review-info-card .review-note-card)
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(TopToBottom).padding((14, 14, 14, 14)).gap(14))
        .children(|ui| {
            // Textarea placeholder (dark box, matches Dioxus review-theme-textarea)
            let textarea_bg = Color::from(0x18181a_u32);
            ui.element()
                .width(grow!())
                .height(fixed!(176.0))
                .background_color(textarea_bg)
                .corner_radius(16.0)
                .border(|b| b.all(1).color(theme.border))
                .layout(|l| l.padding((14, 14, 14, 14)))
                .children(|ui| {
                    ui.text(
                        &l10n::text(state.resolved_language(), "themes-json-placeholder"),
                        |t| t.font_size(13).color(theme.muted_text),
                    );
                });

            // Button row: Export / Import / Clear
            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| l.direction(LeftToRight).gap(8))
                .children(|ui| {
                    pill_button(
                        ui,
                        theme,
                        "json-export",
                        &l10n::text(state.resolved_language(), "themes-export"),
                        false,
                    );
                    pill_button(
                        ui,
                        theme,
                        "json-import",
                        &l10n::text(state.resolved_language(), "themes-import"),
                        false,
                    );
                    pill_button(
                        ui,
                        theme,
                        "json-clear",
                        &l10n::text(state.resolved_language(), "themes-clear"),
                        true,
                    );
                });
        });
}

fn pill_button(ui: &mut Ui, theme: &PlyTheme, id: &'static str, label: &str, subtle: bool) {
    let text_color = if subtle { theme.muted_text } else { theme.text };
    ui.element()
        .id(id)
        .width(fit!())
        .height(fixed!(38.0))
        .background_color(theme.surface_elevated)
        .corner_radius(19.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(CenterX, CenterY).padding((0, 14, 0, 14)))
        .children(|ui| {
            ui.text(label, |t| t.font_size(14).color(text_color));
        });
}

fn caption_label(ui: &mut Ui, theme: &PlyTheme, text: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(24.0))
        .layout(|l| l.align(Left, Bottom))
        .children(|ui| {
            ui.text(text, |t| t.font_size(13).color(theme.muted_text));
        });
}

fn current_theme_card(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let choice_label = theme_choice_display(state, state.theme_choice);
    ui.element()
        .id("current-theme")
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(TopToBottom).padding((14, 14, 14, 14)).gap(12))
        .children(|ui| {
            ui.text(&choice_label, |t| t.font_size(16).color(theme.text));
            ui.text(&theme.name, |t| t.font_size(13).color(theme.muted_text));
            // 4 swatches
            swatch_row(
                ui,
                theme,
                &[
                    theme.background_elevated,
                    theme.surface_elevated,
                    theme.accent,
                    theme.text,
                ],
            );
        });
}

fn swatch_row(ui: &mut Ui, theme: &PlyTheme, colors: &[ply_engine::prelude::Color]) {
    ui.element()
        .width(grow!())
        .height(fixed!(22.0))
        .layout(|l| l.direction(LeftToRight).gap(6))
        .children(|ui| {
            for (i, c) in colors.iter().enumerate() {
                ui.element()
                    .id(("swatch", i as u32))
                    .width(grow!())
                    .height(fixed!(22.0))
                    .background_color(*c)
                    .corner_radius(11.0)
                    .border(|b| b.all(1).color(theme.border))
                    .empty();
            }
        });
}

fn theme_grid(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, rows: &[&[ThemeChoice]]) {
    ui.element()
        .width(grow!())
        .height(fit!())
        .layout(|l| l.direction(TopToBottom).gap(10))
        .children(|ui| {
            for row_choices in rows {
                theme_grid_row(ui, state, theme, row_choices);
            }
        });
}

fn theme_grid_row(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, choices: &[ThemeChoice]) {
    ui.element()
        .width(grow!())
        .height(fit!())
        .layout(|l| l.direction(LeftToRight).gap(10))
        .children(|ui| {
            for (i, choice) in choices.iter().enumerate() {
                theme_card(ui, state, theme, *choice, i);
            }
            // Spacer for single-item rows to maintain 2-column layout
            if choices.len() == 1 {
                ui.element().width(grow!()).height(fit!()).empty();
            }
        });
}

fn theme_card_id(choice: ThemeChoice) -> &'static str {
    match choice {
        ThemeChoice::System => "tc-system",
        ThemeChoice::Dark => "tc-dark",
        ThemeChoice::Light => "tc-light",
        ThemeChoice::CatppuccinLatte => "tc-latte",
        ThemeChoice::CatppuccinFrappe => "tc-frappe",
        ThemeChoice::CatppuccinMacchiato => "tc-macchiato",
        ThemeChoice::CatppuccinMocha => "tc-mocha",
        ThemeChoice::Custom => "tc-custom",
    }
}

fn theme_card(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    choice: ThemeChoice,
    _index: usize,
) {
    let palette = PlyTheme::from_choice(choice, &state.custom_theme);
    let is_active = state.theme_choice == choice;
    let border_c = if is_active {
        theme.accent
    } else {
        theme.border
    };
    let label = theme_choice_display(state, choice);
    ui.element()
        .id(theme_card_id(choice))
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(18.0)
        .border(|b| b.all(1).color(border_c))
        .layout(|l| l.direction(TopToBottom).padding((14, 14, 14, 14)).gap(12))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.theme_choice = choice;
            }
            ui.text(&label, |t| t.font_size(16).color(theme.text));
            ui.text(&palette.name, |t| t.font_size(13).color(theme.muted_text));
            ui.element().width(grow!()).height(grow!()).empty();
            swatch_row(
                ui,
                theme,
                &[
                    palette.background_elevated,
                    palette.surface_elevated,
                    palette.accent,
                    palette.text,
                ],
            );
        });
}

fn info_card(ui: &mut Ui, theme: &PlyTheme, lines: &[&str]) {
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(TopToBottom).padding((14, 14, 14, 14)).gap(14))
        .children(|ui| {
            for (i, line) in lines.iter().enumerate() {
                if i == 0 {
                    ui.text(line, |t| t.font_size(16).color(theme.text));
                } else {
                    ui.text(line, |t| t.font_size(13).color(theme.muted_text));
                }
            }
        });
}

fn theme_choice_display(state: &AppState, choice: ThemeChoice) -> String {
    let key = match choice {
        ThemeChoice::System => "settings-theme-system",
        ThemeChoice::Dark => "settings-theme-dark",
        ThemeChoice::Light => "settings-theme-light",
        ThemeChoice::CatppuccinLatte => "settings-theme-catppuccin-latte",
        ThemeChoice::CatppuccinFrappe => "settings-theme-catppuccin-frappe",
        ThemeChoice::CatppuccinMacchiato => "settings-theme-catppuccin-macchiato",
        ThemeChoice::CatppuccinMocha => "settings-theme-catppuccin-mocha",
        ThemeChoice::Custom => "settings-theme-custom",
    };
    l10n::text(state.resolved_language(), key)
}
