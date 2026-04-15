use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::icons::IconId;
use crate::l10n;
use crate::theme::PlyTheme;

use super::widgets::{HEADER_ACTION_COLOR, bottom_nav, dashboard_nav_items, section_label};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let title = l10n::text(state.resolved_language(), "settings-title");
    // Settings uses title-only bar (no back button), matching Dioxus
    title_only_header(ui, theme, &title);

    ui.element()
        .id("settings-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((14, 14, 0, 14)).gap(12))
        .overflow(|o| {
            o.scroll_y().scrollbar(|s| {
                s.width(3.0)
                    .thumb_color(theme.border_strong)
                    .track_color(theme.surface)
                    .hide_after_frames(120)
            })
        })
        .children(|ui| {
            // ── Language ──
            section_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "settings-language-caption"),
            );
            language_selector(ui, state, theme);

            // ── Theme ──
            section_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "settings-theme-caption"),
            );
            about_entry_card(
                ui,
                state,
                theme,
                "theme-card",
                "theme-link",
                &crate::theme::theme_choice_display_label(state),
                &theme.name.clone(),
                &l10n::text(state.resolved_language(), "settings-theme-description"),
                &l10n::text(state.resolved_language(), "settings-theme-open"),
                MobileScreen::Themes,
            );

            // ── Diagnostics (developer mode only) ──
            if state.developer_mode {
                section_label(
                    ui,
                    theme,
                    &l10n::text(state.resolved_language(), "settings-diagnostics-caption"),
                );
                info_note_card(
                    ui,
                    theme,
                    &[
                        &l10n::text(state.resolved_language(), "settings-status-title"),
                        &state.status.clone(),
                    ],
                );
            }

            // ── Other ──
            section_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "settings-other-caption"),
            );
            developer_mode_card(ui, state, theme);

            // ── About ──
            section_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "settings-about-caption"),
            );
            about_entry_card(
                ui,
                state,
                theme,
                "about-card",
                "about-link",
                &format!("Taled v{}", env!("CARGO_PKG_VERSION")),
                &l10n::text(state.resolved_language(), "settings-about-description"),
                "",
                &l10n::text(state.resolved_language(), "settings-about-open"),
                MobileScreen::About,
            );

            ui.element().width(grow!()).height(fixed!(20.0)).empty();
        });

    let items = dashboard_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Settings);
}

fn title_only_header(ui: &mut Ui, theme: &PlyTheme, title: &str) {
    ui.element()
        .id("header")
        .width(grow!())
        .height(fixed!(56.0))
        .background_color(theme.background_elevated)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| {
            l.direction(TopToBottom)
                .align(Left, CenterY)
                .padding((0, 16, 0, 16))
        })
        .children(|ui| {
            ui.text(title, |t| {
                t.font_size(17).color(theme.text).alignment(CenterX)
            });
        });
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
fn about_entry_card(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    card_id: &'static str,
    link_id: &'static str,
    title: &str,
    subtitle: &str,
    description: &str,
    link_label: &str,
    target: MobileScreen,
) {
    ui.element()
        .id(card_id)
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(TopToBottom).padding((14, 14, 14, 14)).gap(14))
        .children(|ui| {
            // Wrap title in full-width centered container
            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(title, |t| t.font_size(16).color(theme.text));
                });
            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(subtitle, |t| t.font_size(13).color(theme.muted_text));
                });
            if !description.is_empty() {
                ui.element()
                    .width(grow!())
                    .height(fit!())
                    .layout(|l| l.align(CenterX, CenterY))
                    .children(|ui| {
                        ui.text(description, |t| t.font_size(13).color(theme.muted_text));
                    });
            }
            // Center the link button via a full-width wrapper
            ui.element()
                .width(grow!())
                .height(fixed!(24.0))
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.element()
                        .id(link_id)
                        .width(fit!())
                        .height(fixed!(24.0))
                        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY).gap(4))
                        .on_press(move |_, _| {})
                        .children(|ui| {
                            if ui.just_released() {
                                state.navigate(target);
                            }
                            ui.text(link_label, |t| t.font_size(14).color(HEADER_ACTION_COLOR));
                            let chev_tex = state.icon_cache.get(IconId::ChevronRight);
                            ui.element()
                                .width(fixed!(14.0))
                                .height(fixed!(14.0))
                                .background_color(HEADER_ACTION_COLOR)
                                .image(chev_tex)
                                .empty();
                        });
                });
        });
}

fn settings_card(ui: &mut Ui, theme: &PlyTheme, content: impl FnOnce(&mut Ui, &PlyTheme)) {
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(14.0)
        .layout(|l| l.direction(TopToBottom).padding((0, 16, 0, 16)))
        .children(|ui| {
            content(ui, theme);
        });
}

fn info_note_card(ui: &mut Ui, theme: &PlyTheme, lines: &[&str]) {
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

fn developer_mode_card(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let enabled = state.developer_mode;
    let label = l10n::text(state.resolved_language(), "settings-developer-mode");
    settings_card(ui, theme, |ui, theme| {
        ui.element()
            .id("dev-mode-toggle")
            .width(grow!())
            .height(fixed!(44.0))
            .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
            .on_press(move |_, _| {})
            .children(|ui| {
                if ui.just_released() {
                    state.developer_mode = !state.developer_mode;
                }
                ui.text(&label, |t| t.font_size(15).color(theme.text));
                ui.element().width(grow!()).height(fixed!(1.0)).empty();
                toggle_indicator(ui, theme, enabled);
            });
    });
}

fn toggle_indicator(ui: &mut Ui, theme: &PlyTheme, enabled: bool) {
    let bg = if enabled { theme.accent } else { theme.border_strong };
    ui.element()
        .width(fixed!(52.0))
        .height(fixed!(32.0))
        .background_color(bg)
        .corner_radius(16.0)
        .layout(|l| {
            let align_x = if enabled { Right } else { Left };
            l.align(align_x, CenterY).padding((0, 3, 0, 3))
        })
        .children(|ui| {
            ui.element()
                .width(fixed!(26.0))
                .height(fixed!(26.0))
                .background_color(theme.accent_text)
                .corner_radius(13.0)
                .empty();
        });
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
fn language_selector(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    use crate::l10n::AppLanguagePreference;
    let lang = state.resolved_language();
    let pref = state.language_preference;
    let display = match pref {
        AppLanguagePreference::Auto => l10n::text(lang, "settings-language-auto"),
        AppLanguagePreference::English => l10n::text(lang, "settings-language-english"),
        AppLanguagePreference::SimplifiedChinese => l10n::text(lang, "settings-language-zh-hans"),
    };
    let dropdown_bg = Color::from(0x242426_u32);
    ui.element()
        .id("lang-card")
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(TopToBottom).padding((0, 16, 0, 16)))
        .children(|ui| {
            ui.element()
                .id("lang-row")
                .width(grow!())
                .height(fixed!(44.0))
                .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
                .children(|ui| {
                    ui.text(&l10n::text(lang, "settings-language-caption"), |t| {
                        t.font_size(15).color(theme.text)
                    });
                    ui.element().width(grow!()).height(fixed!(1.0)).empty();
                    ui.element()
                        .id("lang-pick")
                        .width(fixed!(132.0))
                        .height(fixed!(38.0))
                        .background_color(dropdown_bg)
                        .corner_radius(12.0)
                        .border(|b| b.all(1).color(theme.border))
                        .layout(|l| {
                            l.direction(LeftToRight)
                                .align(Left, CenterY)
                                .padding((0, 12, 0, 12))
                        })
                        .on_press(move |_, _| {})
                        .children(|ui| {
                            if ui.just_released() {
                                state.show_language_popup = !state.show_language_popup;
                            }
                            ui.text(&display, |t| t.font_size(15).color(theme.text));
                            ui.element().width(grow!()).height(fixed!(1.0)).empty();
                            let chev = state.icon_cache.get(IconId::ChevronRight);
                            ui.element()
                                .width(fixed!(12.0))
                                .height(fixed!(12.0))
                                .background_color(theme.muted_text)
                                .image(chev)
                                .empty();
                        });
                });
        });
}

/// Floating popup overlay for language selection (rendered on top of everything).
#[expect(clippy::excessive_nesting)] // reason: Ply UI floating popup needs nested closures
pub(crate) fn language_popup_overlay(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    use crate::l10n::AppLanguagePreference;
    if !state.show_language_popup {
        return;
    }
    let lang = state.resolved_language();
    let options: [(AppLanguagePreference, &str); 3] = [
        (AppLanguagePreference::Auto, "settings-language-auto"),
        (AppLanguagePreference::English, "settings-language-english"),
        (
            AppLanguagePreference::SimplifiedChinese,
            "settings-language-zh-hans",
        ),
    ];
    let sw = screen_width();
    let sh = screen_height();

    // Backdrop: semi-transparent black overlay
    ui.element()
        .id("lang-backdrop")
        .width(fixed!(sw))
        .height(fixed!(sh))
        .background_color(Color::u_rgba(0, 0, 0, 120))
        .floating(|f| f.attach_root().offset((0.0, 0.0)))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.show_language_popup = false;
            }
        });

    // Popup card centered on screen
    let popup_w: f32 = 260.0;
    let popup_h: f32 = 186.0;
    let popup_x = (sw - popup_w) / 2.0;
    let popup_y = (sh - popup_h) / 2.0;
    ui.element()
        .id("lang-popup")
        .width(fixed!(popup_w))
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(16.0)
        .border(|b| b.all(1).color(theme.border))
        .floating(|f| f.attach_root().offset((popup_x, popup_y)))
        .layout(|l| l.direction(TopToBottom).padding((12, 0, 12, 0)))
        .children(|ui| {
            ui.text(&l10n::text(lang, "settings-language-caption"), |t| {
                t.font_size(16).color(theme.text).alignment(CenterX)
            });
            ui.element()
                .width(grow!())
                .height(fixed!(1.0))
                .background_color(theme.border)
                .empty();

            let current_pref = state.language_preference;
            for (i, (pref_val, key)) in options.iter().enumerate() {
                let label = l10n::text(lang, key);
                let is_selected = *pref_val == current_pref;
                let row_id = match i {
                    0 => "lang-opt-0",
                    1 => "lang-opt-1",
                    _ => "lang-opt-2",
                };
                let row_bg = if is_selected {
                    theme.accent
                } else {
                    theme.surface
                };
                let text_color = if is_selected {
                    theme.accent_text
                } else {
                    theme.text
                };
                let pref_copy = *pref_val;
                ui.element()
                    .id(row_id)
                    .width(grow!())
                    .height(fixed!(48.0))
                    .background_color(row_bg)
                    .layout(|l| l.align(Left, CenterY).padding((0, 16, 0, 16)))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            state.language_preference = pref_copy;
                            state.show_language_popup = false;
                        }
                        ui.text(&label, |t| t.font_size(15).color(text_color));
                    });
                if i < 2 {
                    ui.element()
                        .width(grow!())
                        .height(fixed!(1.0))
                        .background_color(theme.border)
                        .empty();
                }
            }
        });
}
