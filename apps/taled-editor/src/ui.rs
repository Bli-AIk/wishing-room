use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen, TRANSITION_SECS};
use crate::screens;
use crate::theme::PlyTheme;

fn ease_out_cubic(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}

struct TransitionSlide {
    from_screen: MobileScreen,
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
}

fn advance_transition(state: &mut AppState) -> Option<TransitionSlide> {
    let trans = state.page_transition.as_ref()?;
    let elapsed = (get_time() - trans.start_time) as f32;
    let progress = (elapsed / TRANSITION_SECS).min(1.0);
    if progress >= 1.0 {
        state.page_transition = None;
        return None;
    }
    let eased = ease_out_cubic(progress);
    let sw = screen_width();
    let sh = screen_height();
    let from_screen = trans.from_screen;
    let (from_x, from_y, to_x, to_y) = match trans.dir {
        crate::app_state::TransitionDir::Forward => (-sw * eased, 0.0, sw * (1.0 - eased), 0.0),
        crate::app_state::TransitionDir::Back => (sw * eased, 0.0, -sw * (1.0 - eased), 0.0),
        crate::app_state::TransitionDir::Up => (0.0, -sh * eased, 0.0, sh * (1.0 - eased)),
        crate::app_state::TransitionDir::Down => (0.0, sh * eased, 0.0, -sh * (1.0 - eased)),
    };
    Some(TransitionSlide {
        from_screen,
        from_x,
        from_y,
        to_x,
        to_y,
    })
}

fn render_screen(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, screen: MobileScreen) {
    match screen {
        MobileScreen::Dashboard => screens::dashboard::render(ui, state, theme),
        MobileScreen::Editor => screens::editor::render(ui, state, theme),
        MobileScreen::Tilesets => screens::tilesets::render(ui, state, theme),
        MobileScreen::Layers => screens::layers::render(ui, state, theme),
        MobileScreen::Objects => screens::objects::render(ui, state, theme),
        MobileScreen::Properties => screens::properties::render(ui, state, theme),
        MobileScreen::Settings => screens::settings::render(ui, state, theme),
        MobileScreen::Themes => screens::themes::render(ui, state, theme),
        MobileScreen::About => screens::about::render(ui, state, theme),
    }
}

/// Main UI render function called each frame.
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let slide = advance_transition(state);

    // Float controls: hidden during transition, fade in after.
    if state.page_transition.is_some() {
        state.float_controls_alpha = 0.0;
    } else if state.float_controls_alpha < 1.0 {
        state.float_controls_alpha = (state.float_controls_alpha + get_frame_time() * 5.0).min(1.0);
    }

    ui.element()
        .id("root")
        .width(grow!())
        .height(grow!())
        .background_color(theme.background)
        .layout(|l| l.direction(TopToBottom))
        .children(|ui| {
            // Safe-area spacer for camera cutouts / notches.
            if state.safe_inset_top > 0.0 {
                ui.element()
                    .id("safe-area-top")
                    .width(grow!())
                    .height(fixed!(state.safe_inset_top))
                    .background_color(theme.background)
                    .empty();
            }

            if let Some(ts) = slide {
                let sw = screen_width();
                let sh = screen_height();
                let safe = state.safe_inset_top;
                let content_h = sh - safe;

                // From screen (sliding out)
                let from = ts.from_screen;
                let saved = state.mobile_screen;
                state.mobile_screen = from;
                ui.element()
                    .id("trans-from")
                    .width(fixed!(sw))
                    .height(fixed!(content_h))
                    .background_color(theme.background)
                    .floating(|f| {
                        f.attach_root().offset((ts.from_x, safe + ts.from_y))
                    })
                    .layout(|l| l.direction(TopToBottom))
                    .children(|ui| {
                        render_screen(ui, state, theme, from);
                    });
                state.mobile_screen = saved;

                // To screen (sliding in)
                ui.element()
                    .id("trans-to")
                    .width(fixed!(sw))
                    .height(fixed!(content_h))
                    .background_color(theme.background)
                    .floating(|f| {
                        f.attach_root().offset((ts.to_x, safe + ts.to_y))
                    })
                    .layout(|l| l.direction(TopToBottom))
                    .children(|ui| {
                        render_screen(ui, state, theme, state.mobile_screen);
                    });
            } else {
                render_screen(ui, state, theme, state.mobile_screen);
            }
        });
}
