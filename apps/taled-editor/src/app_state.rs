use std::collections::BTreeMap;
use std::time::Instant;

use taled_core::EditorSession;

#[cfg(target_os = "android")]
use crate::platform::log_path;
#[cfg(target_arch = "wasm32")]
use crate::session_ops::load_sample;
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use crate::{
    embedded_samples::embedded_samples,
    platform::{EMBEDDED_DEMO_MAP_PATH, log},
};
#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tool {
    Hand,
    Paint,
    Fill,
    ShapeFill,
    Erase,
    Select,
    AddRectangle,
    AddPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MobileScreen {
    Dashboard,
    Editor,
    Tilesets,
    Layers,
    Objects,
    Properties,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MobileTransition {
    None,
    HorizontalForward,
    HorizontalBackward,
    VerticalForward,
    VerticalBackward,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ActiveTouchPointer {
    pub(crate) pointer_id: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct SingleTouchGesture {
    pub(crate) pointer_id: i32,
    pub(crate) started_at: Instant,
    pub(crate) drag_active: bool,
    pub(crate) anchor_cell: Option<(u32, u32)>,
    pub(crate) last_applied_cell: Option<(u32, u32)>,
    pub(crate) last_surface_x: f64,
    pub(crate) last_surface_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PinchGesture {
    pub(crate) initial_distance: f64,
    pub(crate) initial_zoom_percent: i32,
    pub(crate) world_center_x: f64,
    pub(crate) world_center_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShapeFillPreview {
    pub(crate) start_cell: (u32, u32),
    pub(crate) end_cell: (u32, u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TileSelectionRegion {
    pub(crate) start_cell: (u32, u32),
    pub(crate) end_cell: (u32, u32),
}

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) path_input: String,
    pub(crate) save_as_input: String,
    pub(crate) session: Option<EditorSession>,
    pub(crate) image_cache: BTreeMap<usize, String>,
    pub(crate) active_layer: usize,
    pub(crate) selected_gid: u32,
    pub(crate) selected_cell: Option<(u32, u32)>,
    pub(crate) selected_object: Option<u32>,
    pub(crate) shape_fill_preview: Option<ShapeFillPreview>,
    pub(crate) tile_selection: Option<TileSelectionRegion>,
    pub(crate) tile_selection_preview: Option<TileSelectionRegion>,
    pub(crate) tool: Tool,
    pub(crate) layers_panel_expanded: bool,
    pub(crate) mobile_screen: MobileScreen,
    pub(crate) mobile_transition: MobileTransition,
    pub(crate) mobile_transition_nonce: u64,
    #[cfg(target_arch = "wasm32")]
    pub(crate) show_web_logs: bool,
    pub(crate) zoom_percent: i32,
    pub(crate) pan_x: i32,
    pub(crate) pan_y: i32,
    pub(crate) pending_canvas_center: bool,
    pub(crate) canvas_stage_client_origin: Option<(f64, f64)>,
    pub(crate) canvas_host_size: Option<(f64, f64)>,
    pub(crate) canvas_host_scroll_offset: (f64, f64),
    pub(crate) active_touch_points: Vec<ActiveTouchPointer>,
    pub(crate) single_touch_gesture: Option<SingleTouchGesture>,
    pub(crate) pinch_gesture: Option<PinchGesture>,
    pub(crate) touch_edit_batch_active: bool,
    pub(crate) suppress_click_until: Option<Instant>,
    pub(crate) camera_transition_active: bool,
    pub(crate) status: String,
}

impl Default for AppState {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let mobile_screen = web_query_param("screen")
                .map(|value| parse_mobile_screen(&value))
                .unwrap_or(MobileScreen::Dashboard);
            let path_input = EMBEDDED_DEMO_MAP_PATH.to_string();
            let mut state = Self {
                path_input: path_input.clone(),
                save_as_input: path_input,
                session: None,
                image_cache: BTreeMap::new(),
                active_layer: 0,
                selected_gid: 0,
                selected_cell: None,
                selected_object: None,
                shape_fill_preview: None,
                tile_selection: None,
                tile_selection_preview: None,
                tool: Tool::Paint,
                layers_panel_expanded: false,
                mobile_screen,
                mobile_transition: MobileTransition::None,
                mobile_transition_nonce: 0,
                show_web_logs: false,
                zoom_percent: 100,
                pan_x: 0,
                pan_y: 0,
                pending_canvas_center: false,
                canvas_stage_client_origin: None,
                canvas_host_size: None,
                canvas_host_scroll_offset: (0.0, 0.0),
                active_touch_points: Vec::new(),
                single_touch_gesture: None,
                pinch_gesture: None,
                touch_edit_batch_active: false,
                suppress_click_until: None,
                camera_transition_active: false,
                status: default_status_message(),
            };
            log("boot: constructing default web state");
            load_sample(&mut state);
            return state;
        }

        #[cfg(target_os = "android")]
        {
            let path_input = EMBEDDED_DEMO_MAP_PATH.to_string();
            let state = Self {
                path_input: path_input.clone(),
                save_as_input: path_input,
                session: None,
                image_cache: BTreeMap::new(),
                active_layer: 0,
                selected_gid: 0,
                selected_cell: None,
                selected_object: None,
                shape_fill_preview: None,
                tile_selection: None,
                tile_selection_preview: None,
                tool: Tool::Paint,
                layers_panel_expanded: false,
                mobile_screen: MobileScreen::Dashboard,
                mobile_transition: MobileTransition::None,
                mobile_transition_nonce: 0,
                zoom_percent: 100,
                pan_x: 0,
                pan_y: 0,
                pending_canvas_center: false,
                canvas_stage_client_origin: None,
                canvas_host_size: None,
                canvas_host_scroll_offset: (0.0, 0.0),
                active_touch_points: Vec::new(),
                single_touch_gesture: None,
                pinch_gesture: None,
                touch_edit_batch_active: false,
                suppress_click_until: None,
                camera_transition_active: false,
                status: default_status_message(),
            };
            log("boot: constructing default android state");
            log("boot: android startup skips auto-loading the demo map");
            return state;
        }

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
        {
            let path_input = std::env::current_dir()
                .ok()
                .map(EditorSession::sample_path_from_root)
                .map(|path| path.display().to_string())
                .unwrap_or_default();

            Self {
                path_input: path_input.clone(),
                save_as_input: path_input,
                session: None,
                image_cache: BTreeMap::new(),
                active_layer: 0,
                selected_gid: 0,
                selected_cell: None,
                selected_object: None,
                shape_fill_preview: None,
                tile_selection: None,
                tile_selection_preview: None,
                tool: Tool::Paint,
                layers_panel_expanded: false,
                mobile_screen: MobileScreen::Editor,
                mobile_transition: MobileTransition::None,
                mobile_transition_nonce: 0,
                zoom_percent: 100,
                pan_x: 0,
                pan_y: 0,
                pending_canvas_center: false,
                canvas_stage_client_origin: None,
                canvas_host_size: None,
                canvas_host_scroll_offset: (0.0, 0.0),
                active_touch_points: Vec::new(),
                single_touch_gesture: None,
                pinch_gesture: None,
                touch_edit_batch_active: false,
                suppress_click_until: None,
                camera_transition_active: false,
                status: default_status_message(),
            }
        }
    }
}

fn default_status_message() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        return format!(
            "Web preview ships {} embedded TMX samples. Default: {EMBEDDED_DEMO_MAP_PATH}.",
            embedded_samples().len()
        );
    }

    #[cfg(target_os = "android")]
    {
        let log_path = log_path().unwrap_or_default();
        return format!(
            "Android booted. Pick one of {} embedded TMX samples from Dashboard. Default: {EMBEDDED_DEMO_MAP_PATH}. Logs: {log_path}",
            embedded_samples().len()
        );
    }

    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    {
        "Load a Stage-1 compatible TMX file to begin.".to_string()
    }
}

#[cfg(target_arch = "wasm32")]
fn web_query_param(name: &str) -> Option<String> {
    let search = window()?.location().search().ok()?;
    let query = search.strip_prefix('?').unwrap_or(&search);
    for entry in query.split('&') {
        let Some((key, value)) = entry.split_once('=') else {
            continue;
        };
        if key == name {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn parse_mobile_screen(value: &str) -> MobileScreen {
    match value {
        "editor" => MobileScreen::Editor,
        "tilesets" => MobileScreen::Tilesets,
        "layers" => MobileScreen::Layers,
        "objects" => MobileScreen::Objects,
        "properties" => MobileScreen::Properties,
        "settings" => MobileScreen::Settings,
        _ => MobileScreen::Dashboard,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PaletteTile {
    pub(crate) gid: u32,
    pub(crate) tileset_index: usize,
    pub(crate) local_id: u32,
}
