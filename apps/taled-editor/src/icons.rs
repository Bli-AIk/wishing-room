use ply_engine::prelude::*;

// ── Icon IDs (used as cache keys) ───────────────────────────────────

#[derive(Clone, Copy)]
#[repr(u16)]
pub(crate) enum IconId {
    NavProjects,
    NavAssets,
    NavTilesets,
    NavLayers,
    NavObjects,
    NavProperties,
    NavSettings,
    ToolHand,
    ToolPaint,
    ToolFill,
    ToolEraser,
    ToolSelect,
    ToolShape,
    ToolMagicWand,
    ToolSameTile,
    ToolTerrain,
    Undo,
    Redo,
    Plus,
    ChevronRight,
    EyeOn,
    EyeOff,
    Lock,
    Unlock,
    ModeRectangle,
    ModeEllipse,
    ModeSelReplace,
    ModeSelAdd,
    ModeSelSubtract,
    ModeSelIntersect,
}

fn icon_bytes(id: IconId) -> &'static [u8] {
    match id {
        IconId::NavProjects => include_bytes!("../../../assets/icons/nav-projects.png"),
        IconId::NavAssets => include_bytes!("../../../assets/icons/nav-assets.png"),
        IconId::NavTilesets => include_bytes!("../../../assets/icons/nav-tilesets.png"),
        IconId::NavLayers => include_bytes!("../../../assets/icons/nav-layers.png"),
        IconId::NavObjects => include_bytes!("../../../assets/icons/nav-objects.png"),
        IconId::NavProperties => include_bytes!("../../../assets/icons/nav-properties.png"),
        IconId::NavSettings => include_bytes!("../../../assets/icons/nav-settings.png"),
        IconId::ToolHand => include_bytes!("../../../assets/icons/tool-hand.png"),
        IconId::ToolPaint => include_bytes!("../../../assets/icons/tool-paint.png"),
        IconId::ToolFill => include_bytes!("../../../assets/icons/tool-fill.png"),
        IconId::ToolEraser => include_bytes!("../../../assets/icons/tool-eraser.png"),
        IconId::ToolSelect => include_bytes!("../../../assets/icons/tool-select.png"),
        IconId::ToolShape => include_bytes!("../../../assets/icons/tool-shape.png"),
        IconId::ToolMagicWand => include_bytes!("../../../assets/icons/tool-magic-wand.png"),
        IconId::ToolSameTile => include_bytes!("../../../assets/icons/tool-same-tile.png"),
        IconId::ToolTerrain => include_bytes!("../../../assets/icons/tool-terrain.png"),
        IconId::Undo => include_bytes!("../../../assets/icons/undo.png"),
        IconId::Redo => include_bytes!("../../../assets/icons/redo.png"),
        IconId::Plus => include_bytes!("../../../assets/icons/plus.png"),
        IconId::ChevronRight => include_bytes!("../../../assets/icons/chevron-right.png"),
        IconId::EyeOn => include_bytes!("../../../assets/icons/eye-on.png"),
        IconId::EyeOff => include_bytes!("../../../assets/icons/eye-off.png"),
        IconId::Lock => include_bytes!("../../../assets/icons/lock.png"),
        IconId::Unlock => include_bytes!("../../../assets/icons/unlock.png"),
        IconId::ModeRectangle => include_bytes!("../../../assets/icons/mode-rectangle.png"),
        IconId::ModeEllipse => include_bytes!("../../../assets/icons/mode-ellipse.png"),
        IconId::ModeSelReplace => include_bytes!("../../../assets/icons/mode-sel-replace.png"),
        IconId::ModeSelAdd => include_bytes!("../../../assets/icons/mode-sel-add.png"),
        IconId::ModeSelSubtract => include_bytes!("../../../assets/icons/mode-sel-subtract.png"),
        IconId::ModeSelIntersect => include_bytes!("../../../assets/icons/mode-sel-intersect.png"),
    }
}

// ── Public label-key lookups ────────────────────────────────────────

pub(crate) fn nav_icon_id(label_key: &str) -> IconId {
    match label_key {
        "nav-projects" => IconId::NavProjects,
        "nav-assets" => IconId::NavAssets,
        "nav-tilesets" => IconId::NavTilesets,
        "nav-layers" => IconId::NavLayers,
        "nav-objects" => IconId::NavObjects,
        "nav-properties" => IconId::NavProperties,
        "nav-settings" => IconId::NavSettings,
        _ => IconId::NavProjects,
    }
}

pub(crate) fn tool_icon_id(label_key: &str) -> IconId {
    match label_key {
        "tool-hand" => IconId::ToolHand,
        "tool-stamp" => IconId::ToolPaint,
        "tool-fill" => IconId::ToolFill,
        "tool-eraser" => IconId::ToolEraser,
        "tool-rect-select" => IconId::ToolSelect,
        "tool-shape-fill" => IconId::ToolShape,
        "tool-magic-wand" => IconId::ToolMagicWand,
        "tool-same-tile" => IconId::ToolSameTile,
        "tool-terrain-brush" => IconId::ToolTerrain,
        _ => IconId::ToolHand,
    }
}

// ── Icon cache ───────────────────────────────────────────────────────

/// Caches base icon GPU textures loaded from embedded PNGs.
pub(crate) struct IconTintCache {
    bases: Vec<(u16, Texture2D)>,
}

impl IconTintCache {
    pub(crate) fn new() -> Self {
        Self {
            bases: Vec::with_capacity(20),
        }
    }

    /// Return the base icon texture. Caller uses ply `background_color` for tinting.
    pub(crate) fn get(&mut self, id: IconId) -> Texture2D {
        let id_u16 = id as u16;
        if let Some(entry) = self.bases.iter().find(|(k, _)| *k == id_u16) {
            return entry.1.clone();
        }
        let tex = Texture2D::from_file_with_format(icon_bytes(id), None);
        tex.set_filter(FilterMode::Linear);
        self.bases.push((id_u16, tex.clone()));
        tex
    }

    /// Preload all mode icon textures to avoid lazy-load GPU issues on Android.
    pub(crate) fn preload_mode_icons(&mut self) {
        let ids = [
            IconId::ModeRectangle,
            IconId::ModeEllipse,
            IconId::ModeSelReplace,
            IconId::ModeSelAdd,
            IconId::ModeSelSubtract,
            IconId::ModeSelIntersect,
        ];
        for id in ids {
            self.get(id);
        }
    }
}
