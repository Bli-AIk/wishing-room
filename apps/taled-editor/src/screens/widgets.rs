use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::PlyTheme;

// ── Page header ─────────────────────────────────────────────

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn page_header(
    ui: &mut Ui,
    theme: &PlyTheme,
    title: &str,
    left_action: Option<(&str, MobileScreen)>,
    right_action: Option<(&str, MobileScreen)>,
    state: &mut AppState,
) {
    ui.element()
        .id("header")
        .width(grow!())
        .height(fixed!(44.0))
        .background_color(theme.background_elevated)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((0, 12, 0, 12))
        })
        .children(|ui| {
            // Left button
            if let Some((label, target)) = left_action {
                ui.element()
                    .id("header-left")
                    .width(fixed!(60.0))
                    .height(fixed!(32.0))
                    .layout(|l| l.align(Left, CenterY))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            state.navigate(target);
                        }
                        ui.text(label, |t| t.font_size(15).color(theme.accent));
                    });
            } else {
                ui.element().width(fixed!(60.0)).height(fixed!(1.0)).empty();
            }

            // Title
            ui.element()
                .width(grow!())
                .height(fixed!(32.0))
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(title, |t| {
                        t.font_size(17).color(theme.text).alignment(CenterX)
                    });
                });

            // Right button
            if let Some((label, target)) = right_action {
                ui.element()
                    .id("header-right")
                    .width(fixed!(60.0))
                    .height(fixed!(32.0))
                    .layout(|l| l.align(Right, CenterY))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            state.navigate(target);
                        }
                        ui.text(label, |t| {
                            t.font_size(15).color(theme.accent).alignment(Right)
                        });
                    });
            } else {
                ui.element().width(fixed!(60.0)).height(fixed!(1.0)).empty();
            }
        });
}

// ── Bottom navigation bar ──────────────────────────────────

pub(crate) struct NavItem {
    pub(crate) label_key: &'static str,
    pub(crate) screen: MobileScreen,
}

pub(crate) fn dashboard_nav_items() -> [NavItem; 3] {
    [
        NavItem {
            label_key: "nav-projects",
            screen: MobileScreen::Dashboard,
        },
        NavItem {
            label_key: "nav-assets",
            screen: MobileScreen::Dashboard,
        },
        NavItem {
            label_key: "nav-settings",
            screen: MobileScreen::Settings,
        },
    ]
}

pub(crate) fn editor_nav_items() -> [NavItem; 4] {
    [
        NavItem {
            label_key: "nav-tilesets",
            screen: MobileScreen::Tilesets,
        },
        NavItem {
            label_key: "nav-layers",
            screen: MobileScreen::Layers,
        },
        NavItem {
            label_key: "nav-objects",
            screen: MobileScreen::Objects,
        },
        NavItem {
            label_key: "nav-settings",
            screen: MobileScreen::Settings,
        },
    ]
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn bottom_nav(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    items: &[NavItem],
    active: MobileScreen,
) {
    ui.element()
        .id("bottom-nav")
        .width(grow!())
        .height(fixed!(56.0))
        .background_color(theme.surface)
        .border(|b| b.top(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY))
        .children(|ui| {
            for (i, item) in items.iter().enumerate() {
                let is_active = item.screen == active;
                let label = l10n::text(state.resolved_language(), item.label_key);
                let color = if is_active {
                    theme.accent
                } else {
                    theme.muted_text
                };
                let target = item.screen;
                ui.element()
                    .id(("nav-item", i as u32))
                    .width(grow!())
                    .height(grow!())
                    .layout(|l| l.align(CenterX, CenterY).gap(2))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            state.navigate(target);
                        }
                        ui.text(&label, |t| t.font_size(11).color(color).alignment(CenterX));
                    });
            }
        });
}

// ── Reusable button ────────────────────────────────────────

#[allow(dead_code)]
pub(crate) fn action_button(ui: &mut Ui, id: &'static str, label: &str, theme: &PlyTheme) -> bool {
    let mut clicked = false;
    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(44.0))
        .background_color(theme.surface_elevated)
        .corner_radius(10.0)
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                clicked = true;
            }
            let bg = if ui.pressed() {
                theme.border
            } else {
                theme.surface_elevated
            };
            let _ = bg;
            ui.text(label, |t| {
                t.font_size(16).color(theme.text).alignment(CenterX)
            });
        });
    clicked
}

// ── Section header (category label) ────────────────────────

pub(crate) fn section_label(ui: &mut Ui, theme: &PlyTheme, text: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(28.0))
        .layout(|l| l.align(Left, Bottom).padding((0, 16, 0, 16)))
        .children(|ui| {
            ui.text(text, |t| t.font_size(13).color(theme.muted_text));
        });
}
