use crate::app_state::AppState;

/// Cancel tile selection transfer (e.g., when switching tools).
#[allow(dead_code)]
pub(crate) fn cancel_tile_selection_transfer(state: &mut AppState) {
    state.tile_selection = None;
    state.tile_selection_cells = None;
    state.tile_selection_preview = None;
    state.tile_selection_preview_cells = None;
    state.tile_selection_transfer = None;
}
