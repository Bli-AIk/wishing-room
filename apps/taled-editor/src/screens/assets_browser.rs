use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::icons::IconId;
use crate::l10n;
use crate::theme::PlyTheme;
use crate::utdr_index::{GAME_KEYS, GAME_SHORT_LABELS};

use super::widgets::{bottom_nav, dashboard_nav_items, section_label};

// ── Main render ────────────────────────────────────────────────────

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let title = l10n::text(lang, "assets-title");

    // Page header (TopToBottom so text gets full width for CenterX alignment)
    ui.element()
        .id("assets-header")
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
            ui.text(&title, |t| {
                t.font_size(17).color(theme.text).alignment(CenterX)
            });
        });

    search_bar(ui, state, theme);
    download_banner(ui, state, theme);
    game_selector(ui, state, theme);
    room_list(ui, state, theme);

    let items = dashboard_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Assets);
}

// ── Search bar ─────────────────────────────────────────────────────

fn search_bar(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let input_val = ui.get_text_value("assets-search");
    if input_val != state.utdr_search {
        state.utdr_search = input_val.to_string();
    }

    // padding: (top, right, bottom, left)
    ui.element()
        .id("search-row")
        .width(grow!())
        .height(fixed!(44.0))
        .background_color(theme.background)
        .layout(|l| l.align(CenterX, CenterY).padding((6, 16, 6, 16)))
        .children(|ui| {
            ui.element()
                .id("assets-search")
                .width(grow!())
                .height(fixed!(32.0))
                .background_color(theme.surface_elevated)
                .corner_radius(8.0)
                .layout(|l| l.padding((0, 10, 0, 10)).align(Left, CenterY))
                .text_input(|t| {
                    t.font_size(14)
                        .text_color(theme.text)
                        .cursor_color(theme.text)
                        .placeholder("Search rooms...")
                        .max_length(64)
                })
                .empty();
        });
}

// ── Download progress banner ───────────────────────────────────────

fn download_banner(ui: &mut Ui, state: &AppState, theme: &PlyTheme) {
    let status = match &state.download_status {
        Some(crate::utdr_download::DownloadStatus::InProgress(msg)) => msg.as_str(),
        Some(crate::utdr_download::DownloadStatus::Error(msg)) => msg.as_str(),
        None => return,
    };
    let is_err = matches!(
        &state.download_status,
        Some(crate::utdr_download::DownloadStatus::Error(_))
    );
    let bg = if is_err { theme.danger } else { theme.accent };
    ui.element()
        .id("download-banner")
        .width(grow!())
        .height(fixed!(28.0))
        .background_color(bg)
        .layout(|l| {
            l.direction(TopToBottom)
                .align(Left, CenterY)
                .padding((0, 16, 0, 16))
        })
        .children(|ui| {
            ui.text(status, |t| {
                t.font_size(12)
                    .color(Color::u_rgb(0xff, 0xff, 0xff))
                    .alignment(CenterX)
            });
        });
}

// ── Game selector (horizontal chip bar, centered) ──────────────────

fn game_selector(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    // padding: (top, right, bottom, left)
    ui.element()
        .id("game-selector")
        .width(grow!())
        .height(fixed!(40.0))
        .background_color(theme.background)
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((4, 16, 4, 16))
                .gap(8)
        })
        .children(|ui| {
            for (i, &key) in GAME_KEYS.iter().enumerate() {
                let is_active = state.utdr_selected_game == key;
                game_chip(
                    ui,
                    state,
                    theme,
                    key,
                    GAME_SHORT_LABELS[i],
                    is_active,
                    i as u32,
                );
            }
        });
}

fn game_chip(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    key: &'static str,
    label: &str,
    active: bool,
    index: u32,
) {
    let bg = if active {
        theme.accent
    } else {
        theme.surface_elevated
    };
    let fg = if active {
        Color::u_rgb(0xff, 0xff, 0xff)
    } else {
        theme.muted_text
    };
    ui.element()
        .id(("chip", index))
        .width(fit!())
        .height(fixed!(30.0))
        .background_color(bg)
        .corner_radius(15.0)
        .layout(|l| l.align(CenterX, CenterY).padding((0, 14, 0, 14)))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.utdr_selected_game = key.to_string();
            }
            ui.text(label, |t| t.font_size(13).color(fg));
        });
}

// ── Room list ──────────────────────────────────────────────────────

fn room_list(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();

    let game_key = state.utdr_selected_game.clone();

    let (game_label, rooms) = match state
        .utdr_index
        .as_ref()
        .and_then(|idx| idx.games.get(&state.utdr_selected_game))
    {
        Some(game) => (game.label.clone(), game.rooms.clone()),
        None => {
            no_index_placeholder(ui, state, theme);
            return;
        }
    };

    let search = state.utdr_search.to_lowercase();
    let filtered: Vec<_> = if search.is_empty() {
        rooms.iter().collect()
    } else {
        rooms
            .iter()
            .filter(|r| r.name.to_lowercase().contains(&search))
            .collect()
    };

    let count_label = l10n::text_with_args(
        lang,
        "assets-room-count",
        &[("count", filtered.len().to_string()), ("game", game_label)],
    );

    // padding: (top, right, bottom, left)
    ui.element()
        .id("room-scroll")
        .width(grow!())
        .height(grow!())
        .background_color(theme.background)
        .overflow(|o| {
            o.scroll_y().clip_x().scrollbar(|s| {
                s.width(6.0)
                    .corner_radius(3.0)
                    .thumb_color(theme.border_strong)
                    .track_color(theme.surface)
            })
        })
        .layout(|l| {
            l.direction(TopToBottom)
                .align(Left, Top)
                .padding((4, 16, 8, 16))
        })
        .children(|ui| {
            section_label(ui, theme, &count_label);
            for (i, room) in filtered.iter().enumerate() {
                room_row(ui, state, theme, room, &game_key, i as u32, i == 0);
            }
        });
}

fn no_index_placeholder(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    ui.element()
        .width(grow!())
        .height(grow!())
        .layout(|l| l.align(CenterX, CenterY))
        .children(|ui| {
            let msg = l10n::text(lang, "assets-no-index");
            ui.text(&msg, |t| t.font_size(14).color(theme.muted_text));
        });
}

fn room_row(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    room: &crate::utdr_index::UtdrRoom,
    game_key: &str,
    index: u32,
    is_first: bool,
) {
    let size_kb = room.size as f32 / 1024.0;
    let size_text = if size_kb < 1.0 {
        format!("{} B", room.size)
    } else {
        format!("{:.1} KB", size_kb)
    };
    let (repo, branch) = state
        .utdr_index
        .as_ref()
        .map(|idx| (idx.repo.clone(), idx.branch.clone()))
        .unwrap_or_else(|| ("Bli-AIk/open-utdr-maps".into(), "main".into()));

    // padding: (top, right, bottom, left)
    ui.element()
        .id(("room", index))
        .width(grow!())
        .height(fixed!(64.0))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((10, 0, 10, 0))
                .gap(12)
        })
        .border(|b| {
            if is_first {
                b
            } else {
                b.top(1).color(theme.border)
            }
        })
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() && state.download_rx.is_none() {
                let path = room.path.clone();
                crate::utdr_download::start_room_download(state, &path, &repo, &branch);
            }

            // Thumbnail (fetched from GitHub, or placeholder)
            let thumb = crate::utdr_thumbs::get(game_key, &room.name, &repo, &branch);
            room_thumb(ui, state, theme, thumb, index);

            // Text block: room name + size
            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| l.direction(TopToBottom).gap(3))
                .children(|ui| {
                    ui.text(&room.name, |t| t.font_size(15).color(theme.text));
                    ui.text(&size_text, |t| t.font_size(12).color(theme.muted_text));
                });
        });
}

fn room_thumb(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    thumb: Option<Texture2D>,
    index: u32,
) {
    if let Some(tex) = thumb {
        ui.element()
            .id(("room-thumb", index))
            .width(fixed!(44.0))
            .height(fixed!(44.0))
            .corner_radius(10.0)
            .background_color(theme.surface)
            .border(|b| b.all(1).color(theme.border))
            .layout(|l| l.align(CenterX, CenterY))
            .image(tex)
            .empty();
    } else {
        ui.element()
            .id(("room-thumb", index))
            .width(fixed!(44.0))
            .height(fixed!(44.0))
            .corner_radius(10.0)
            .background_color(theme.surface)
            .border(|b| b.all(1).color(theme.border))
            .layout(|l| l.align(CenterX, CenterY))
            .children(|ui| {
                let icon = state.icon_cache.get(IconId::NavAssets);
                ui.element()
                    .width(fixed!(20.0))
                    .height(fixed!(20.0))
                    .background_color(theme.muted_text)
                    .image(icon)
                    .empty();
            });
    }
}
