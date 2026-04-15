use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::icons::nav_icon_id;
use crate::l10n;
use crate::theme::PlyTheme;

/// Dioxus CSS `.review-header-action` / `.review-link-button` uses `#b6b6bb`.
pub(crate) const HEADER_ACTION_COLOR: Color = Color::u_rgb(0xb6, 0xb6, 0xbb);

// ── Review-style page header (3-column grid: 92px | 1fr | 92px) ─────

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
#[allow(dead_code)]
pub(crate) fn review_header(
    ui: &mut Ui,
    theme: &PlyTheme,
    title: &str,
    left_label: Option<&str>,
    right_label: Option<&str>,
) {
    ui.element()
        .id("header")
        .width(grow!())
        .height(fixed!(56.0))
        .background_color(theme.background_elevated)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((20, 16, 16, 16))
                .gap(6)
        })
        .children(|ui| {
            // Left column (92px)
            ui.element()
                .width(fixed!(92.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    if let Some(label) = left_label {
                        ui.text(label, |t| t.font_size(14).color(HEADER_ACTION_COLOR));
                    }
                });

            // Center: title
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    ui.text(title, |t| {
                        t.font_size(17).color(theme.text).alignment(CenterX)
                    });
                });

            // Right column (92px)
            ui.element()
                .width(fixed!(92.0))
                .height(grow!())
                .layout(|l| l.align(Right, CenterY))
                .children(|ui| {
                    if let Some(label) = right_label {
                        ui.text(label, |t| {
                            t.font_size(14).color(HEADER_ACTION_COLOR).alignment(Right)
                        });
                    }
                });
        });
}

// ── Page header with navigation ─────────────────────────────────────

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
        .height(fixed!(56.0))
        .background_color(theme.background_elevated)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((20, 16, 16, 16))
                .gap(6)
        })
        .children(|ui| {
            // Left column (92px)
            if let Some((label, target)) = left_action {
                ui.element()
                    .width(fixed!(92.0))
                    .height(grow!())
                    .layout(|l| l.align(Left, CenterY))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            if target == MobileScreen::Editor {
                                state.navigate_down(target);
                            } else {
                                state.navigate_back_to(target);
                            }
                        }
                        ui.text(label, |t| t.font_size(14).color(HEADER_ACTION_COLOR));
                    });
            } else {
                ui.element().width(fixed!(92.0)).height(fixed!(1.0)).empty();
            }

            // Center: title
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    ui.text(title, |t| {
                        t.font_size(17).color(theme.text).alignment(CenterX)
                    });
                });

            // Right column (92px)
            if let Some((label, target)) = right_action {
                ui.element()
                    .width(fixed!(92.0))
                    .height(grow!())
                    .layout(|l| l.align(Right, CenterY))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            if target == MobileScreen::Editor {
                                state.navigate_down(target);
                            } else {
                                state.navigate_back_to(target);
                            }
                        }
                        ui.text(label, |t| {
                            t.font_size(14).color(HEADER_ACTION_COLOR).alignment(Right)
                        });
                    });
            } else {
                ui.element().width(fixed!(92.0)).height(fixed!(1.0)).empty();
            }
        });
}

// ── Bottom navigation bar ──────────────────────────────────────────

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

pub(crate) fn editor_nav_items() -> [NavItem; 3] {
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
            label_key: "nav-properties",
            screen: MobileScreen::Properties,
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
        .height(fixed!(72.0))
        .background_color(theme.background_elevated)
        .border(|b| b.top(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((10, 12, 22, 12))
                .gap(8)
        })
        .children(|ui| {
            let mut active_found = false;
            for (i, item) in items.iter().enumerate() {
                let is_active = if !active_found && item.screen == active {
                    active_found = true;
                    true
                } else {
                    false
                };
                let label = l10n::text(state.resolved_language(), item.label_key);
                let is_disabled = item.label_key == "nav-assets";
                let color = if is_disabled {
                    Color::u_rgb(0x6e, 0x6e, 0x73)
                } else if is_active {
                    theme.accent
                } else {
                    theme.muted_text
                };
                let target = item.screen;
                let label_key = item.label_key;
                ui.element()
                    .id(("nav-item", i as u32))
                    .width(grow!())
                    .height(grow!())
                    .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(6))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            if is_disabled {
                                let tool_label = l10n::text(state.resolved_language(), label_key);
                                state.status = l10n::text_with_args(
                                    state.resolved_language(),
                                    "tool-status-not-implemented",
                                    &[("tool", tool_label)],
                                );
                            } else if active == MobileScreen::Editor {
                                state.navigate_up(target);
                            } else if (active.is_editor_subtab() && target.is_editor_subtab())
                                || (active.is_dashboard_tab() && target.is_dashboard_tab())
                            {
                                state.navigate_tab(target);
                            } else {
                                state.navigate(target);
                            }
                        }
                        // Nav icon (24x24)
                        let icon_id = nav_icon_id(item.label_key);
                        let icon_tex = state.icon_cache.get(icon_id);
                        ui.element()
                            .width(fixed!(24.0))
                            .height(fixed!(24.0))
                            .background_color(color)
                            .image(icon_tex)
                            .empty();
                        ui.text(&label, |t| t.font_size(12).color(color));
                    });
            }
        });
}

// ── Reusable button ────────────────────────────────────────────────

#[expect(dead_code)] // reason: will be used for interactive buttons on various screens
pub(crate) fn action_button(ui: &mut Ui, id: &'static str, label: &str, theme: &PlyTheme) -> bool {
    let mut clicked = false;
    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(44.0))
        .background_color(theme.surface_elevated)
        .corner_radius(10.0)
        .layout(|l| l.align(Left, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                clicked = true;
            }
            ui.text(label, |t| {
                t.font_size(16).color(theme.text).alignment(CenterX)
            });
        });
    clicked
}

// ── Section header (category label) ────────────────────────────────

pub(crate) fn section_label(ui: &mut Ui, theme: &PlyTheme, text: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(32.0))
        .layout(|l| l.align(Left, Bottom).padding((0, 0, 4, 0)))
        .children(|ui| {
            ui.text(text, |t| t.font_size(13).color(theme.muted_text));
        });
}
