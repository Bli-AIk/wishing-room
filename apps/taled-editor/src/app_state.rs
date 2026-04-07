use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use ply_engine::prelude::Texture2D;
use taled_core::EditorSession;

use crate::l10n::{
    AppLanguagePreference, SupportedLanguage, detect_device_locale_tag, resolve_language,
};
use crate::theme::{ThemeChoice, ThemePaletteData, default_custom_theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum Tool {
    Hand,
    Paint,
    Fill,
    ShapeFill,
    Erase,
    Select,
    MagicWand,
    SelectSameTile,
    AddRectangle,
    AddPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum TileSelectionMode {
    Replace,
    Add,
    Subtract,
    Intersect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ShapeFillMode {
    Rectangle,
    Ellipse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum MobileScreen {
    Dashboard,
    Editor,
    Tilesets,
    Layers,
    Objects,
    Properties,
    Settings,
    Themes,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub(crate) struct ActiveTouchPointer {
    pub(crate) pointer_id: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SingleTouchGesture {
    pub(crate) pointer_id: i32,
    pub(crate) started_at: Instant,
    pub(crate) drag_active: bool,
    pub(crate) anchor_cell: Option<(i32, i32)>,
    pub(crate) last_applied_cell: Option<(u32, u32)>,
    pub(crate) last_surface_x: f64,
    pub(crate) last_surface_y: f64,
    pub(crate) pan_remainder_x: f64,
    pub(crate) pan_remainder_y: f64,
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
    pub(crate) start_cell: (i32, i32),
    pub(crate) end_cell: (i32, i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum TileSelectionHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct TileClipboard {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) tiles: Vec<u32>,
    pub(crate) mask: Vec<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum TileSelectionTransferMode {
    Copy,
    Cut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TileSelectionTransfer {
    pub(crate) source_layer: usize,
    pub(crate) source_selection: TileSelectionRegion,
    pub(crate) source_mask: Vec<bool>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) tiles: Vec<u32>,
    pub(crate) mask: Vec<bool>,
    pub(crate) mode: TileSelectionTransferMode,
}

#[allow(dead_code)]
pub(crate) struct AppState {
    pub(crate) session: Option<EditorSession>,
    pub(crate) tileset_textures: BTreeMap<usize, Texture2D>,
    pub(crate) active_layer: usize,
    pub(crate) selected_gid: u32,
    pub(crate) selected_cell: Option<(u32, u32)>,
    pub(crate) selected_object: Option<u32>,
    pub(crate) shape_fill_preview: Option<ShapeFillPreview>,
    pub(crate) tile_clipboard: Option<TileClipboard>,
    pub(crate) tile_selection: Option<TileSelectionRegion>,
    pub(crate) tile_selection_cells: Option<BTreeSet<(i32, i32)>>,
    pub(crate) tile_selection_preview: Option<TileSelectionRegion>,
    pub(crate) tile_selection_preview_cells: Option<BTreeSet<(i32, i32)>>,
    pub(crate) tile_selection_closing: Option<TileSelectionRegion>,
    pub(crate) tile_selection_closing_cells: Option<BTreeSet<(i32, i32)>>,
    pub(crate) tile_selection_closing_started_at: Option<Instant>,
    pub(crate) tile_selection_last_tap_at: Option<Instant>,
    pub(crate) tile_selection_transfer: Option<TileSelectionTransfer>,
    pub(crate) tile_selection_mode: TileSelectionMode,
    pub(crate) shape_fill_mode: ShapeFillMode,
    pub(crate) tool: Tool,
    pub(crate) layers_panel_expanded: bool,
    pub(crate) mobile_screen: MobileScreen,
    pub(crate) language_preference: AppLanguagePreference,
    pub(crate) theme_choice: ThemeChoice,
    pub(crate) custom_theme: ThemePaletteData,
    pub(crate) theme_json_buffer: String,
    pub(crate) device_locale_tag: String,
    pub(crate) about_contributors_expanded: bool,
    pub(crate) zoom_percent: i32,
    pub(crate) pan_x: f32,
    pub(crate) pan_y: f32,
    pub(crate) active_touch_points: Vec<ActiveTouchPointer>,
    pub(crate) single_touch_gesture: Option<SingleTouchGesture>,
    pub(crate) pinch_gesture: Option<PinchGesture>,
    pub(crate) touch_edit_batch_active: bool,
    pub(crate) camera_transition_active: bool,
    pub(crate) status: String,
    pub(crate) canvas_texture: Option<Texture2D>,
    pub(crate) canvas_dirty: bool,
    pub(crate) show_grid: bool,
}

impl AppState {
    pub(crate) fn new() -> Self {
        let device_locale_tag = detect_device_locale_tag();
        Self {
            session: None,
            tileset_textures: BTreeMap::new(),
            active_layer: 0,
            selected_gid: 0,
            selected_cell: None,
            selected_object: None,
            shape_fill_preview: None,
            tile_clipboard: None,
            tile_selection: None,
            tile_selection_cells: None,
            tile_selection_preview: None,
            tile_selection_preview_cells: None,
            tile_selection_closing: None,
            tile_selection_closing_cells: None,
            tile_selection_closing_started_at: None,
            tile_selection_last_tap_at: None,
            tile_selection_transfer: None,
            tile_selection_mode: TileSelectionMode::Replace,
            shape_fill_mode: ShapeFillMode::Rectangle,
            tool: Tool::Paint,
            layers_panel_expanded: false,
            mobile_screen: MobileScreen::Dashboard,
            language_preference: AppLanguagePreference::Auto,
            theme_choice: ThemeChoice::Dark,
            custom_theme: default_custom_theme(),
            theme_json_buffer: String::new(),
            device_locale_tag,
            about_contributors_expanded: false,
            zoom_percent: 100,
            pan_x: 0.0,
            pan_y: 0.0,
            active_touch_points: Vec::new(),
            single_touch_gesture: None,
            pinch_gesture: None,
            touch_edit_batch_active: false,
            camera_transition_active: false,
            status: "Welcome to Taled".to_string(),
            canvas_texture: None,
            canvas_dirty: true,
            show_grid: true,
        }
    }

    pub(crate) fn resolved_language(&self) -> SupportedLanguage {
        resolve_language(self.language_preference, &self.device_locale_tag)
    }

    pub(crate) fn navigate(&mut self, screen: MobileScreen) {
        self.mobile_screen = screen;
    }
}

#[allow(dead_code)]
pub(crate) fn is_tile_selection_tool(tool: Tool) -> bool {
    matches!(tool, Tool::Select | Tool::MagicWand | Tool::SelectSameTile)
}

#[allow(dead_code)]
pub(crate) fn selection_bounds(region: &TileSelectionRegion) -> (i32, i32, i32, i32) {
    let min_x = region.start_cell.0.min(region.end_cell.0);
    let min_y = region.start_cell.1.min(region.end_cell.1);
    let max_x = region.start_cell.0.max(region.end_cell.0);
    let max_y = region.start_cell.1.max(region.end_cell.1);
    (min_x, min_y, max_x, max_y)
}
