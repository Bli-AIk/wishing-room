use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use ply_engine::prelude::{RenderTarget, Texture2D, get_time};
use taled_core::EditorSession;

use crate::icons::IconTintCache;
use crate::l10n::{
    AppLanguagePreference, SupportedLanguage, detect_device_locale_tag, resolve_language,
};
use crate::theme::{ThemeChoice, ThemePaletteData, default_custom_theme};

/// Read the initial screen index set by JS (`window.taled_initial_screen`).
#[cfg(target_arch = "wasm32")]
fn read_initial_screen() -> MobileScreen {
    unsafe extern "C" {
        fn taled_get_initial_screen() -> i32;
    }
    // SAFETY: `taled_get_initial_screen` is provided by ply_bundle.js.
    let idx = unsafe { taled_get_initial_screen() };
    MobileScreen::from_index(idx)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_initial_screen() -> MobileScreen {
    MobileScreen::Dashboard
}

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

impl MobileScreen {
    #[cfg(target_arch = "wasm32")]
    fn from_index(idx: i32) -> Self {
        match idx {
            1 => Self::Editor,
            2 => Self::Tilesets,
            3 => Self::Layers,
            4 => Self::Objects,
            5 => Self::Properties,
            6 => Self::Settings,
            7 => Self::Themes,
            8 => Self::About,
            _ => Self::Dashboard,
        }
    }

    pub(crate) fn is_editor_subtab(self) -> bool {
        matches!(
            self,
            Self::Tilesets | Self::Layers | Self::Objects | Self::Properties
        )
    }

    pub(crate) fn is_dashboard_tab(self) -> bool {
        matches!(self, Self::Dashboard | Self::Settings)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransitionDir {
    Forward,
    Back,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PageTransition {
    pub(crate) from_screen: MobileScreen,
    pub(crate) start_time: f64,
    pub(crate) dir: TransitionDir,
}

pub(crate) const TRANSITION_SECS: f32 = 0.2;

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
    pub(crate) outside_existing_selection: bool,
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

/// Tracks which kind of action was last pushed, so undo/redo can dispatch
/// to the correct stack (document edits vs. selection changes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UndoActionKind {
    DocumentEdit,
    SelectionChange,
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

/// Snap-to-grid easing animation for the tile-strip viewfinder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ViewfinderSnapAnim {
    pub(crate) start_time: f64,
    pub(crate) from: (f32, f32),
    pub(crate) to: (f32, f32),
}

#[allow(dead_code)]
pub(crate) struct AppState {
    pub(crate) session: Option<EditorSession>,
    pub(crate) tileset_textures: BTreeMap<usize, Texture2D>,
    /// Per-tile textures for collection-of-images tilesets ((tileset_index, local_id) → texture).
    pub(crate) tile_textures: BTreeMap<(usize, u32), Texture2D>,
    /// Cached cropped tile chip render targets (gid → RenderTarget). Invalidated on tileset reload.
    /// We keep the full RenderTarget alive so Android doesn't free the backing GL framebuffer.
    pub(crate) tile_chip_cache: BTreeMap<u32, RenderTarget>,
    /// Render-target for the currently selected tile chip (with blue border baked in).
    pub(crate) selected_chip_rt: Option<(u32, RenderTarget)>,
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
    pub(crate) hidden_layers: BTreeSet<usize>,
    pub(crate) property_panel_expanded: bool,
    pub(crate) sheet_zoom: f32,
    pub(crate) sheet_zoom_key: u32,
    pub(crate) sheet_pinch_dist: Option<f64>,
    pub(crate) page_transition: Option<PageTransition>,
    /// Opacity for editor floating controls (0..1), fades in after page transition.
    pub(crate) float_controls_alpha: f32,
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
    pub(crate) canvas_dirty: bool,
    /// Set when tile data changes (paint/erase/fill/delete/place/undo of edits).
    /// When false, the cached tilemap texture is reused (skipping expensive re-render).
    pub(crate) tiles_dirty: bool,
    /// Cached zoom percent for detecting zoom changes that need re-render.
    pub(crate) canvas_cached_zoom: i32,
    /// Interleaved undo action order (document edits + selection changes).
    pub(crate) undo_action_order: Vec<UndoActionKind>,
    pub(crate) redo_action_order: Vec<UndoActionKind>,
    /// Selection-specific undo/redo stacks (stores previous selection cells).
    pub(crate) selection_undo_stack: Vec<Option<BTreeSet<(i32, i32)>>>,
    pub(crate) selection_redo_stack: Vec<Option<BTreeSet<(i32, i32)>>>,
    pub(crate) show_grid: bool,
    pub(crate) active_tileset: usize,
    pub(crate) icon_cache: IconTintCache,
    pub(crate) logo_texture: Option<Texture2D>,
    pub(crate) debug_info: String,
    pub(crate) perf_info: String,
    /// Diagnostic: last eye toggle event (layer index, new state).
    pub(crate) last_eye_toggle: Option<(usize, bool)>,
    /// How many times the canvas was rebuilt since last log entry.
    pub(crate) canvas_rebuild_count: u32,
    /// Frame countdown for deferred centering (0 = done, >0 = frames remaining).
    pub(crate) pending_canvas_center: u8,
    pub(crate) center_debug: String,
    /// Top safe-area inset in logical pixels (for camera cutouts / notches).
    pub(crate) safe_inset_top: f32,
    /// True while the user is dragging the pan joystick.
    pub(crate) joystick_active: bool,
    /// Current knob offset from joystick center (logical px).
    pub(crate) joystick_offset: (f32, f32),
    /// True while the user is dragging the zoom slider.
    pub(crate) zoom_slider_active: bool,
    /// Current handle offset from slider center (logical px).
    pub(crate) zoom_slider_offset: f32,
    /// Fractional zoom accumulator for smooth slider zoom.
    pub(crate) zoom_accumulator: f32,
    /// Viewfinder offset in tile units (fractional during drag).
    pub(crate) viewfinder_offset: (f32, f32),
    /// True while a touch is active on the viewfinder palette.
    pub(crate) viewfinder_touch_active: bool,
    /// True when the active touch has exceeded the drag threshold.
    pub(crate) viewfinder_dragging: bool,
    /// Mouse position when the viewfinder touch started.
    pub(crate) viewfinder_drag_start_mouse: (f32, f32),
    /// Viewfinder offset when the touch started.
    pub(crate) viewfinder_drag_start_offset: (f32, f32),
    /// In-progress snap-to-grid easing animation.
    pub(crate) viewfinder_snap_anim: Option<ViewfinderSnapAnim>,
    /// Viewfinder zoom level: 0 = 9×3, 1 = 6×2, 2 = 3×1.
    pub(crate) viewfinder_zoom_level: u8,
    /// Initial pinch distance for viewfinder zoom gesture.
    pub(crate) viewfinder_pinch_dist: Option<f64>,
    /// Whether the language-selection popup is currently visible.
    pub(crate) show_language_popup: bool,
    /// Name of the currently active workspace (directory name).
    pub(crate) active_workspace: String,
    /// Cached list of workspace directory names.
    pub(crate) workspace_list: Vec<String>,
    /// Whether the workspace switcher popup is currently visible.
    pub(crate) show_workspace_picker: bool,
    /// Whether the import action menu popup is currently visible.
    pub(crate) show_import_menu: bool,
    /// Pending import mode: "workspace" or "tmx" while waiting for SAF picker result.
    pub(crate) import_pending: Option<ImportMode>,
}

/// Which kind of import the user initiated.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImportMode {
    Workspace,
    Tmx,
}

impl AppState {
    pub(crate) fn new() -> Self {
        let device_locale_tag = detect_device_locale_tag();
        Self {
            session: None,
            tileset_textures: BTreeMap::new(),
            tile_textures: BTreeMap::new(),
            tile_chip_cache: BTreeMap::new(),
            selected_chip_rt: None,
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
            hidden_layers: BTreeSet::new(),
            property_panel_expanded: true,
            sheet_zoom: 0.0,
            sheet_zoom_key: 0,
            sheet_pinch_dist: None,
            page_transition: None,
            float_controls_alpha: 1.0,
            mobile_screen: read_initial_screen(),
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
            canvas_dirty: true,
            tiles_dirty: true,
            canvas_cached_zoom: 0,
            undo_action_order: Vec::new(),
            redo_action_order: Vec::new(),
            selection_undo_stack: Vec::new(),
            selection_redo_stack: Vec::new(),
            show_grid: true,
            active_tileset: 0,
            icon_cache: {
                let mut cache = IconTintCache::new();
                cache.preload_mode_icons();
                cache
            },
            logo_texture: None,
            debug_info: String::new(),
            perf_info: String::new(),
            last_eye_toggle: None,
            canvas_rebuild_count: 0,
            pending_canvas_center: 0,
            center_debug: String::new(),
            safe_inset_top: 0.0,
            joystick_active: false,
            joystick_offset: (0.0, 0.0),
            zoom_slider_active: false,
            zoom_slider_offset: 0.0,
            zoom_accumulator: 0.0,
            viewfinder_offset: (0.0, 0.0),
            viewfinder_touch_active: false,
            viewfinder_dragging: false,
            viewfinder_drag_start_mouse: (0.0, 0.0),
            viewfinder_drag_start_offset: (0.0, 0.0),
            viewfinder_snap_anim: None,
            viewfinder_zoom_level: 1,
            viewfinder_pinch_dist: None,
            show_language_popup: false,
            active_workspace: crate::workspace::BUILTIN_WORKSPACE.to_string(),
            workspace_list: Vec::new(),
            show_workspace_picker: false,
            show_import_menu: false,
            import_pending: None,
        }
    }

    pub(crate) fn resolved_language(&self) -> SupportedLanguage {
        resolve_language(self.language_preference, &self.device_locale_tag)
    }

    pub(crate) fn navigate(&mut self, screen: MobileScreen) {
        if screen == self.mobile_screen {
            return;
        }
        self.show_language_popup = false;
        self.show_workspace_picker = false;
        self.show_import_menu = false;
        self.page_transition = Some(PageTransition {
            from_screen: self.mobile_screen,
            start_time: get_time(),
            dir: TransitionDir::Forward,
        });
        self.mobile_screen = screen;
    }

    pub(crate) fn navigate_back_to(&mut self, screen: MobileScreen) {
        if screen == self.mobile_screen {
            return;
        }
        self.page_transition = Some(PageTransition {
            from_screen: self.mobile_screen,
            start_time: get_time(),
            dir: TransitionDir::Back,
        });
        self.mobile_screen = screen;
    }

    pub(crate) fn navigate_up(&mut self, screen: MobileScreen) {
        if screen == self.mobile_screen {
            return;
        }
        self.page_transition = Some(PageTransition {
            from_screen: self.mobile_screen,
            start_time: get_time(),
            dir: TransitionDir::Up,
        });
        self.mobile_screen = screen;
    }

    pub(crate) fn navigate_down(&mut self, screen: MobileScreen) {
        if screen == self.mobile_screen {
            return;
        }
        self.page_transition = Some(PageTransition {
            from_screen: self.mobile_screen,
            start_time: get_time(),
            dir: TransitionDir::Down,
        });
        self.mobile_screen = screen;
    }

    /// Switch tab instantly (no slide animation).
    pub(crate) fn navigate_tab(&mut self, screen: MobileScreen) {
        self.page_transition = None;
        self.mobile_screen = screen;
    }

    /// Navigate to the logical parent screen.
    pub(crate) fn navigate_back(&mut self) {
        let target = match self.mobile_screen {
            MobileScreen::About | MobileScreen::Themes => MobileScreen::Settings,
            MobileScreen::Tilesets
            | MobileScreen::Layers
            | MobileScreen::Objects
            | MobileScreen::Properties => MobileScreen::Editor,
            MobileScreen::Settings | MobileScreen::Dashboard | MobileScreen::Editor => {
                MobileScreen::Dashboard
            }
        };
        if target == self.mobile_screen {
            return;
        }
        let dir = if self.mobile_screen.is_editor_subtab() {
            TransitionDir::Down
        } else {
            TransitionDir::Back
        };
        self.page_transition = Some(PageTransition {
            from_screen: self.mobile_screen,
            start_time: get_time(),
            dir,
        });
        self.mobile_screen = target;
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
