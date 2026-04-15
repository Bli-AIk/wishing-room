use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::PlyTheme;
use crate::utdr_index::{GAME_KEYS, GAME_SHORT_LABELS};

use super::widgets::{bottom_nav, dashboard_nav_items, section_label};

// ── Main render ────────────────────────────────────────────────────

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let title = l10n::text(lang, "assets-title");

    // Page header
    ui.element()
        .id("assets-header")
        .width(grow!())
        .height(fixed!(56.0))
        .background_color(theme.background_elevated)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| l.align(CenterX, CenterY).padding((20, 16, 16, 16)))
        .children(|ui| {
            ui.text(&title, |t| t.font_size(17).color(theme.text).alignment(CenterX));
        });

    // Game selector chips
    game_selector(ui, state, theme);

    // Room list (scrollable)
    room_list(ui, state, theme);

    // Bottom navigation
    let items = dashboard_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Assets);
}

// ── Game selector (horizontal chip bar) ────────────────────────────

fn game_selector(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    ui.element()
        .id("game-selector")
        .width(grow!())
        .height(fixed!(44.0))
        .background_color(theme.background)
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((12, 8, 8, 8))
                .gap(8)
        })
        .children(|ui| {
            for (i, &key) in GAME_KEYS.iter().enumerate() {
                let is_active = state.utdr_selected_game == key;
                game_chip(ui, state, theme, key, GAME_SHORT_LABELS[i], is_active, i as u32);
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
    let bg = if active { theme.accent } else { theme.surface_elevated };
    let fg = if active {
        Color::u_rgb(0xff, 0xff, 0xff)
    } else {
        theme.muted_text
    };
    ui.element()
        .id(("chip", index))
        .width(grow!())
        .height(fixed!(30.0))
        .background_color(bg)
        .corner_radius(15.0)
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.utdr_selected_game = key.to_string();
            }
            ui.text(label, |t| t.font_size(13).color(fg).alignment(CenterX));
        });
}

// ── Room list ──────────────────────────────────────────────────────

fn room_list(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();

    // Get rooms for selected game
    let (game_label, rooms) = match state
        .utdr_index
        .as_ref()
        .and_then(|idx| idx.games.get(&state.utdr_selected_game))
    {
        Some(game) => (game.label.clone(), game.rooms.clone()),
        None => {
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    let msg = l10n::text(lang, "assets-no-index");
                    ui.text(&msg, |t| t.font_size(14).color(theme.muted_text));
                });
            return;
        }
    };

    // Filter rooms by search
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
        &[
            ("count", filtered.len().to_string()),
            ("game", game_label),
        ],
    );

    ui.element()
        .id("room-scroll")
        .width(grow!())
        .height(grow!())
        .background_color(theme.background)
        .overflow(|o| {
            o.scroll_y().scrollbar(|s| {
                s.thumb_color(theme.muted_text).track_color(theme.background)
            })
        })
        .layout(|l| {
            l.direction(TopToBottom)
                .align(Left, Top)
                .padding((16, 8, 16, 8))
                .gap(2)
        })
        .children(|ui| {
            section_label(ui, theme, &count_label);

            for (i, room) in filtered.iter().enumerate() {
                room_row(ui, state, theme, room, i as u32);
            }
        });
}

fn room_row(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    room: &crate::utdr_index::UtdrRoom,
    index: u32,
) {
    let size_kb = room.size as f32 / 1024.0;
    let size_text = if size_kb < 1.0 {
        format!("{} B", room.size)
    } else {
        format!("{:.1} KB", size_kb)
    };

    ui.element()
        .id(("room", index))
        .width(grow!())
        .height(fixed!(48.0))
        .background_color(theme.surface_elevated)
        .corner_radius(8.0)
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((14, 10, 14, 10))
                .gap(8)
        })
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                let lang = state.resolved_language();
                state.status = l10n::text_with_args(
                    lang,
                    "assets-room-tapped",
                    &[("room", room.name.clone())],
                );
            }
            // Room name
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    ui.text(&room.name, |t| t.font_size(15).color(theme.text));
                });
            // Size
            ui.element()
                .width(fit!())
                .height(grow!())
                .layout(|l| l.align(Right, CenterY))
                .children(|ui| {
                    ui.text(&size_text, |t| t.font_size(12).color(theme.muted_text));
                });
        });
}
