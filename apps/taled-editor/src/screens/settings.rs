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
            settings_card_single(ui, theme, |ui, theme| {
                info_row(
                    ui,
                    theme,
                    &l10n::text(state.resolved_language(), "settings-language-caption"),
                    "Auto",
                );
            });

            // ── Theme ──
            section_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "settings-theme-caption"),
            );
            segmented_theme_control(ui, state, theme);

            // ── Diagnostics ──
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

            // ── Export ──
            section_label(
                ui,
                theme,
                &l10n::text(state.resolved_language(), "settings-export-caption"),
            );
            settings_card(ui, theme, |ui, theme| {
                toggle_row(
                    ui,
                    theme,
                    "exp-json",
                    &l10n::text(state.resolved_language(), "settings-export-json"),
                    true,
                );
                separator_row(ui, theme);
                toggle_row(
                    ui,
                    theme,
                    "exp-xml",
                    &l10n::text(state.resolved_language(), "settings-export-xml"),
                    true,
                );
                separator_row(ui, theme);
                toggle_row(
                    ui,
                    theme,
                    "exp-png",
                    &l10n::text(state.resolved_language(), "settings-export-png"),
                    true,
                );
            });

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
                "Taled",
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

fn about_entry_card(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    title: &str,
    subtitle: &str,
    description: &str,
    link_label: &str,
    target: MobileScreen,
) {
    ui.element()
        .id("entry-card")
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| {
            l.direction(TopToBottom)
                .align(CenterX, Top)
                .padding((14, 14, 14, 14))
                .gap(14)
        })
        .children(|ui| {
            ui.text(title, |t| {
                t.font_size(16).color(theme.text).alignment(CenterX)
            });
            ui.text(subtitle, |t| {
                t.font_size(13).color(theme.muted_text).alignment(CenterX)
            });
            if !description.is_empty() {
                ui.text(description, |t| {
                    t.font_size(13).color(theme.muted_text).alignment(CenterX)
                });
            }
            ui.element()
                .id("entry-link")
                .width(fit!())
                .height(fixed!(24.0))
                .layout(|l| {
                    l.direction(LeftToRight)
                        .align(CenterX, CenterY)
                        .gap(4)
                })
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

fn settings_card_single(ui: &mut Ui, theme: &PlyTheme, content: impl FnOnce(&mut Ui, &PlyTheme)) {
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
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

fn toggle_row(ui: &mut Ui, theme: &PlyTheme, id: &'static str, label: &str, enabled: bool) {
    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(44.0))
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
        .children(|ui| {
            ui.text(label, |t| t.font_size(15).color(theme.text));
            ui.element().width(grow!()).height(fixed!(1.0)).empty();
            let bg = if enabled {
                theme.accent
            } else {
                theme.border_strong
            };
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
        });
}

fn info_row(ui: &mut Ui, theme: &PlyTheme, label: &str, value: &str) {
    let dropdown_bg = Color::from(0x242426_u32);
    ui.element()
        .width(grow!())
        .height(fixed!(44.0))
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
        .children(|ui| {
            ui.text(label, |t| t.font_size(15).color(theme.text));
            ui.element().width(grow!()).height(fixed!(1.0)).empty();
            // Dropdown-style box matching Dioxus .review-select-input
            ui.element()
                .width(fixed!(132.0))
                .height(fixed!(38.0))
                .background_color(dropdown_bg)
                .corner_radius(12.0)
                .border(|b| b.all(1).color(theme.border))
                .layout(|l| l.align(Left, CenterY).padding((0, 12, 0, 12)))
                .children(|ui| {
                    ui.text(value, |t| t.font_size(15).color(theme.text));
                });
        });
}

fn separator_row(ui: &mut Ui, theme: &PlyTheme) {
    ui.element()
        .width(grow!())
        .height(fixed!(1.0))
        .background_color(theme.border)
        .empty();
}

/// Dioxus `.review-segmented` — 3-button segmented control for Dark / Light / System.
fn segmented_theme_control(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let labels = [
        l10n::text(state.resolved_language(), "settings-theme-dark"),
        l10n::text(state.resolved_language(), "settings-theme-light"),
        l10n::text(state.resolved_language(), "settings-theme-system"),
    ];
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(TopToBottom).padding((8, 8, 8, 8)))
        .children(|ui| {
            segmented_bar(ui, theme, &labels, 0);
        });
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
fn segmented_bar(ui: &mut Ui, theme: &PlyTheme, labels: &[String], active_idx: usize) {
    let seg_bg = Color::from(0x2c2c2e_u32);
    let active_bg = Color::from(0x4d4d52_u32);
    ui.element()
        .width(grow!())
        .height(fixed!(48.0))
        .background_color(seg_bg)
        .corner_radius(16.0)
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((4, 4, 4, 4))
                .gap(2)
        })
        .children(|ui| {
            for (i, label) in labels.iter().enumerate() {
                let is_active = i == active_idx;
                let btn_bg = if is_active { active_bg } else { seg_bg };
                let text_color = if is_active { theme.text } else { theme.muted_text };
                ui.element()
                    .id(("seg-btn", i as u32))
                    .width(grow!())
                    .height(fixed!(40.0))
                    .background_color(btn_bg)
                    .corner_radius(12.0)
                    .layout(|l| l.align(Left, CenterY))
                    .children(|ui| {
                        ui.text(label, |t| {
                            t.font_size(14).color(text_color).alignment(CenterX)
                        });
                    });
            }
        });
}
