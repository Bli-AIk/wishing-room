use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::icons::IconId;
use crate::l10n;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, dashboard_nav_items, page_header};

static LOGO_BYTES: &[u8] = include_bytes!("../../../../assets/branding/taled.png");

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    // Lazily create the logo texture with nearest-neighbor filtering
    if state.logo_texture.is_none() {
        let tex = Texture2D::from_file_with_format(LOGO_BYTES, None);
        tex.set_filter(FilterMode::Nearest);
        state.logo_texture = Some(tex);
    }
    let title = l10n::text(state.resolved_language(), "settings-about-caption");
    page_header(
        ui,
        theme,
        &title,
        Some(("Back", MobileScreen::Settings)),
        None,
        state,
    );

    // clip_x prevents long URLs (no whitespace = can't word-wrap) from
    // expanding min_dimensions.width and pushing content past the right edge.
    ui.element()
        .id("about-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(12).padding((14, 14, 0, 14)))
        .overflow(|o| o.scroll_y().clip_x())
        .children(|ui| {
            about_body_content(ui, state, theme);
        });

    let items = dashboard_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Settings);
}

fn about_body_content(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    hero_section(ui, state, theme);

    row_card(
        ui,
        theme,
        &l10n::text(state.resolved_language(), "settings-about-license-title"),
        &l10n::text(state.resolved_language(), "settings-about-license-value"),
    );

    disclosure_card(ui, state, theme);

    info_card_with_links(
        ui,
        state,
        theme,
        &[
            "settings-about-repository-title",
            "settings-about-repository-description",
            "settings-about-repository-contributing",
        ],
        &[("settings-about-github", "https://github.com/Bli-AIk/taled")],
    );

    info_card_with_links(
        ui,
        state,
        theme,
        &[
            "settings-about-stack-title",
            "settings-about-stack-description",
        ],
        &[
            ("settings-about-ply", "https://plyx.iz.rs"),
            ("settings-about-rust", "https://www.rust-lang.org/community"),
            (
                "settings-about-rs-tiled",
                "https://github.com/mapeditor/rs-tiled",
            ),
            ("settings-about-fluent", "https://projectfluent.org/"),
            ("settings-about-crates", "https://crates.io/"),
        ],
    );

    // Dedicated Ply tribute card
    info_card_with_links(
        ui,
        state,
        theme,
        &[
            "settings-about-ply-tribute-title",
            "settings-about-ply-tribute-description",
            "settings-about-ply-tribute-star",
        ],
        &[(
            "settings-about-ply-github",
            "https://github.com/TheRedDeveloper/ply-engine",
        )],
    );

    info_card_with_links(
        ui,
        state,
        theme,
        &[
            "settings-about-thanks-title",
            "settings-about-thanks-description",
        ],
        &[
            ("settings-about-tiled", "https://www.mapeditor.org/"),
            ("settings-about-undertale", "https://undertale.com/"),
            ("settings-about-deltarune", "https://deltarune.com/"),
            (
                "settings-about-open-utdr",
                "https://github.com/Bli-AIk/open-utdr-maps",
            ),
        ],
    );

    ui.element().width(grow!()).height(fixed!(20.0)).empty();
}

fn hero_section(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let logo_tex = state
        .logo_texture
        .clone()
        .unwrap_or_else(Texture2D::empty);
    ui.element()
        .id("about-hero")
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(22.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| {
            l.direction(TopToBottom)
                .align(Left, Top)
                .gap(12)
                .padding((20, 18, 18, 18))
        })
        .children(|ui| {
            // Logo centered in a full-width wrapper
            ui.element()
                .width(grow!())
                .height(fixed!(84.0))
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.element()
                        .width(fixed!(84.0))
                        .height(fixed!(84.0))
                        .image(logo_tex)
                        .empty();
                });
            ui.text("Taled", |t| {
                t.font_size(16).color(theme.text).alignment(CenterX)
            });
            ui.text(
                &l10n::text(state.resolved_language(), "settings-about-description"),
                |t| t.font_size(13).color(theme.muted_text).alignment(CenterX),
            );
        });
}

fn row_card(ui: &mut Ui, theme: &PlyTheme, left: &str, right: &str) {
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((0, 16, 0, 16))
        })
        .children(|ui| {
            // Inner row (matching .review-setting-row min-height 44px)
            ui.element()
                .width(grow!())
                .height(fixed!(44.0))
                .layout(|l| l.direction(LeftToRight).align(Left, CenterY).gap(10))
                .children(|ui| {
                    ui.text(left, |t| t.font_size(15).color(theme.text));
                    ui.element().width(grow!()).height(fixed!(1.0)).empty();
                    ui.text(right, |t| t.font_size(13).color(theme.muted_text));
                });
        });
}

fn disclosure_card(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let disc_label = if state.about_contributors_expanded {
        l10n::text(
            state.resolved_language(),
            "settings-about-contributors-hide",
        )
    } else {
        l10n::text(
            state.resolved_language(),
            "settings-about-contributors-show",
        )
    };
    let chevron = if state.about_contributors_expanded {
        "∧"
    } else {
        "∨"
    };

    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(TopToBottom).padding((0, 16, 0, 16)))
        .children(|ui| {
            // Header row: "Contributors" | "Show contributor list ∨"
            ui.element()
                .id("contrib-toggle")
                .width(grow!())
                .height(fixed!(44.0))
                .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.about_contributors_expanded = !state.about_contributors_expanded;
                    }
                    ui.text(
                        &l10n::text(
                            state.resolved_language(),
                            "settings-about-contributors-title",
                        ),
                        |t| t.font_size(15).color(theme.text),
                    );
                    ui.element().width(grow!()).height(fixed!(1.0)).empty();
                    ui.text(&disc_label, |t| t.font_size(13).color(theme.muted_text));
                    ui.text(chevron, |t| t.font_size(13).color(theme.muted_text));
                });
            if state.about_contributors_expanded {
                separator(ui, theme);
                ui.element()
                    .width(grow!())
                    .height(fixed!(36.0))
                    .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
                    .children(|ui| {
                        ui.text(
                            &l10n::text(
                                state.resolved_language(),
                                "settings-about-contributors-value",
                            ),
                            |t| t.font_size(15).color(theme.text),
                        );
                        ui.element().width(grow!()).height(fixed!(1.0)).empty();
                        ui.text(
                            &l10n::text(
                                state.resolved_language(),
                                "settings-about-contributor-role",
                            ),
                            |t| t.font_size(13).color(theme.muted_text),
                        );
                    });
            }
        });
}

/// Reference CSS `.review-about-link-url` link color used across all themes.
const LINK_URL_COLOR: Color = Color::u_rgb(0x74, 0xa8, 0xff);

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
fn info_card_with_links(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    text_keys: &[&str],
    links: &[(&str, &str)],
) {
    let lang = state.resolved_language();
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(TopToBottom).padding((14, 16, 14, 16)).gap(14))
        .children(|ui| {
            for (i, key) in text_keys.iter().enumerate() {
                let text = l10n::text(lang, key);
                if i == 0 {
                    ui.text(&text, |t| t.font_size(15).color(theme.text));
                } else {
                    ui.text(&text, |t| t.font_size(13).color(theme.muted_text));
                }
            }
            if !links.is_empty() {
                ui.element()
                    .width(grow!())
                    .height(fit!())
                    .layout(|l| l.direction(TopToBottom).gap(10))
                    .children(|ui| {
                        for &(title_key, url) in links {
                            let title = l10n::text(lang, title_key);
                            let url_owned = url.to_string();
                            ui.element()
                                .width(grow!())
                                .height(fit!())
                                .layout(|l| {
                                    l.direction(LeftToRight).align(Left, CenterY).gap(4)
                                })
                                .on_press(move |_, _| {})
                                .children(|ui| {
                                    if ui.just_released() {
                                        crate::platform::open_url(&url_owned);
                                    }
                                    // Left: title + URL stacked
                                    ui.element()
                                        .width(grow!())
                                        .height(fit!())
                                        .layout(|l| l.direction(TopToBottom).gap(2))
                                        .children(|ui| {
                                            ui.text(&title, |t| {
                                                t.font_size(14).color(theme.text)
                                            });
                                            ui.text(&url_owned, |t| {
                                                t.font_size(12).color(LINK_URL_COLOR)
                                            });
                                        });
                                    // Right: chevron icon
                                    let chev_tex = state.icon_cache.get(IconId::ChevronRight);
                                    ui.element()
                                        .width(fixed!(14.0))
                                        .height(fixed!(14.0))
                                        .background_color(theme.muted_text)
                                        .image(chev_tex)
                                        .empty();
                                });
                        }
                    });
            }
        });
}

fn separator(ui: &mut Ui, theme: &PlyTheme) {
    ui.element()
        .width(grow!())
        .height(fixed!(1.0))
        .background_color(theme.border)
        .empty();
}
