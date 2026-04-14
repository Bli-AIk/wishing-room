use crate::error::{EditorError, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// TMX flip flag constants (stored in the high bits of tile GIDs).
const FLIP_FLAGS_MASK: u32 = 0xE000_0000;
const FLIP_H_FLAG: u32 = 0x8000_0000;
const FLIP_V_FLAG: u32 = 0x4000_0000;
const FLIP_D_FLAG: u32 = 0x2000_0000;

/// Strip flip/rotation flags from a raw tile GID to get the base tile ID.
pub fn strip_flip_flags(gid: u32) -> u32 {
    gid & !FLIP_FLAGS_MASK
}

/// Extract flip flags from a raw tile GID as (flip_h, flip_v, flip_d).
pub fn tile_flip_flags(gid: u32) -> (bool, bool, bool) {
    (
        gid & FLIP_H_FLAG != 0,
        gid & FLIP_V_FLAG != 0,
        gid & FLIP_D_FLAG != 0,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Orientation {
    Orthogonal,
}

impl Orientation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Orthogonal => "orthogonal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderOrder {
    RightDown,
    RightUp,
    LeftDown,
    LeftUp,
}

impl RenderOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RightDown => "right-down",
            Self::RightUp => "right-up",
            Self::LeftDown => "left-down",
            Self::LeftUp => "left-up",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl PropertyValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
        }
    }

    pub fn as_editor_string(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::Bool(value) => value.to_string(),
        }
    }

    pub fn parse_like(&self, raw: &str) -> Result<Self> {
        match self {
            Self::String(_) => Ok(Self::String(raw.to_string())),
            Self::Int(_) => raw
                .parse()
                .map(Self::Int)
                .map_err(|_| EditorError::Invalid(format!("cannot parse '{raw}' as int"))),
            Self::Float(_) => raw
                .parse()
                .map(Self::Float)
                .map_err(|_| EditorError::Invalid(format!("cannot parse '{raw}' as float"))),
            Self::Bool(_) => raw
                .parse()
                .map(Self::Bool)
                .map_err(|_| EditorError::Invalid(format!("cannot parse '{raw}' as bool"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationFrame {
    pub tile_id: u32,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TilesetImage {
    pub source: PathBuf,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tileset {
    pub version: Option<String>,
    pub tiled_version: Option<String>,
    pub name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_count: u32,
    pub columns: u32,
    pub image: TilesetImage,
    /// Per-tile images for collection-of-images tilesets (local tile ID → image).
    pub tile_images: BTreeMap<u32, TilesetImage>,
    /// Tile animations keyed by local tile ID.
    pub animations: BTreeMap<u32, Vec<AnimationFrame>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TilesetReference {
    pub first_gid: u32,
    pub source: PathBuf,
    pub tileset: Tileset,
}

impl TilesetReference {
    pub fn resolved_source_path(&self, map_path: &Path) -> PathBuf {
        map_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(&self.source)
    }

    pub fn resolved_image_path(&self, map_path: &Path) -> PathBuf {
        self.resolved_source_path(map_path)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(&self.tileset.image.source)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileLayer {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<u32>,
    pub properties: Vec<Property>,
}

impl TileLayer {
    pub fn index_of(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some((y * self.width + x) as usize)
        }
    }

    pub fn tile_at(&self, x: u32, y: u32) -> Option<u32> {
        self.index_of(x, y).map(|index| self.tiles[index])
    }

    pub fn set_tile(&mut self, x: u32, y: u32, gid: u32) -> Result<()> {
        let index = self.index_of(x, y).ok_or_else(|| {
            EditorError::Invalid(format!("tile coordinate out of bounds: {x},{y}"))
        })?;
        self.tiles[index] = gid;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectShape {
    Rectangle,
    Point,
    Ellipse,
    Polygon { points: Vec<(f32, f32)> },
    Text { text: String, wrap: bool },
    Capsule,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapObject {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub shape: ObjectShape,
    pub gid: Option<u32>,
    pub properties: Vec<Property>,
}

impl MapObject {
    pub fn is_point(&self) -> bool {
        matches!(self.shape, ObjectShape::Point)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectLayer {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub objects: Vec<MapObject>,
    pub properties: Vec<Property>,
}

impl ObjectLayer {
    pub fn object_mut(&mut self, object_id: u32) -> Option<&mut MapObject> {
        self.objects
            .iter_mut()
            .find(|object| object.id == object_id)
    }

    pub fn object(&self, object_id: u32) -> Option<&MapObject> {
        self.objects.iter().find(|object| object.id == object_id)
    }

    pub fn remove_object(&mut self, object_id: u32) -> Option<MapObject> {
        let index = self
            .objects
            .iter()
            .position(|object| object.id == object_id)?;
        Some(self.objects.remove(index))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Layer {
    Tile(TileLayer),
    Object(ObjectLayer),
}

impl Layer {
    pub fn id(&self) -> u32 {
        match self {
            Self::Tile(layer) => layer.id,
            Self::Object(layer) => layer.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Tile(layer) => &layer.name,
            Self::Object(layer) => &layer.name,
        }
    }

    pub fn name_mut(&mut self) -> &mut String {
        match self {
            Self::Tile(layer) => &mut layer.name,
            Self::Object(layer) => &mut layer.name,
        }
    }

    pub fn visible(&self) -> bool {
        match self {
            Self::Tile(layer) => layer.visible,
            Self::Object(layer) => layer.visible,
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        match self {
            Self::Tile(layer) => layer.visible = visible,
            Self::Object(layer) => layer.visible = visible,
        }
    }

    pub fn locked(&self) -> bool {
        match self {
            Self::Tile(layer) => layer.locked,
            Self::Object(layer) => layer.locked,
        }
    }

    pub fn set_locked(&mut self, locked: bool) {
        match self {
            Self::Tile(layer) => layer.locked = locked,
            Self::Object(layer) => layer.locked = locked,
        }
    }

    pub fn opacity(&self) -> f32 {
        match self {
            Self::Tile(layer) => layer.opacity,
            Self::Object(layer) => layer.opacity,
        }
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        match self {
            Self::Tile(layer) => layer.opacity = opacity,
            Self::Object(layer) => layer.opacity = opacity,
        }
    }

    pub fn properties(&self) -> &Vec<Property> {
        match self {
            Self::Tile(layer) => &layer.properties,
            Self::Object(layer) => &layer.properties,
        }
    }

    pub fn properties_mut(&mut self) -> &mut Vec<Property> {
        match self {
            Self::Tile(layer) => &mut layer.properties,
            Self::Object(layer) => &mut layer.properties,
        }
    }

    pub fn as_tile(&self) -> Option<&TileLayer> {
        match self {
            Self::Tile(layer) => Some(layer),
            Self::Object(_) => None,
        }
    }

    pub fn as_tile_mut(&mut self) -> Option<&mut TileLayer> {
        match self {
            Self::Tile(layer) => Some(layer),
            Self::Object(_) => None,
        }
    }

    pub fn as_object(&self) -> Option<&ObjectLayer> {
        match self {
            Self::Tile(_) => None,
            Self::Object(layer) => Some(layer),
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut ObjectLayer> {
        match self {
            Self::Tile(_) => None,
            Self::Object(layer) => Some(layer),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    pub version: Option<String>,
    pub tiled_version: Option<String>,
    pub orientation: Orientation,
    pub render_order: RenderOrder,
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub next_layer_id: u32,
    pub next_object_id: u32,
    pub properties: Vec<Property>,
    pub tilesets: Vec<TilesetReference>,
    pub layers: Vec<Layer>,
}

impl Map {
    pub fn total_pixel_width(&self) -> u32 {
        self.width * self.tile_width
    }

    pub fn total_pixel_height(&self) -> u32 {
        self.height * self.tile_height
    }

    pub fn has_animations(&self) -> bool {
        self.tilesets
            .iter()
            .any(|ts| !ts.tileset.animations.is_empty())
    }

    pub fn layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }

    pub fn layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }

    pub fn tile_layer_indices(&self) -> Vec<usize> {
        self.layers
            .iter()
            .enumerate()
            .filter_map(|(index, layer)| matches!(layer, Layer::Tile(_)).then_some(index))
            .collect()
    }

    pub fn object_layer_indices(&self) -> Vec<usize> {
        self.layers
            .iter()
            .enumerate()
            .filter_map(|(index, layer)| matches!(layer, Layer::Object(_)).then_some(index))
            .collect()
    }

    pub fn tileset_for_gid(&self, gid: u32) -> Option<(usize, &TilesetReference)> {
        let base_gid = strip_flip_flags(gid);
        self.tilesets
            .iter()
            .enumerate()
            .rev()
            .find(|(_, tileset)| base_gid >= tileset.first_gid)
    }

    pub fn tile_reference_for_gid(&self, gid: u32) -> Option<TileReference<'_>> {
        let base_gid = strip_flip_flags(gid);
        if base_gid == 0 {
            return None;
        }

        let (tileset_index, tileset) = self.tileset_for_gid(base_gid)?;
        let local_id = base_gid - tileset.first_gid;
        // Collection-of-images tilesets may have non-contiguous tile IDs that
        // exceed tile_count.  Accept them when an explicit tile image exists.
        if local_id >= tileset.tileset.tile_count
            && !tileset.tileset.tile_images.contains_key(&local_id)
        {
            return None;
        }

        Some(TileReference {
            tileset_index,
            tileset,
            local_id,
        })
    }

    /// Add a new empty tile layer and return its index.
    pub fn add_tile_layer(&mut self, name: &str) -> usize {
        let id = self.next_layer_id;
        self.next_layer_id += 1;
        let layer = TileLayer {
            id,
            name: name.to_string(),
            visible: true,
            locked: false,
            opacity: 1.0,
            width: self.width,
            height: self.height,
            tiles: vec![0; (self.width * self.height) as usize],
            properties: Vec::new(),
        };
        self.layers.push(Layer::Tile(layer));
        self.layers.len() - 1
    }

    /// Add a new empty object layer and return its index.
    pub fn add_object_layer(&mut self, name: &str) -> usize {
        let id = self.next_layer_id;
        self.next_layer_id += 1;
        let layer = ObjectLayer {
            id,
            name: name.to_string(),
            visible: true,
            locked: false,
            opacity: 1.0,
            objects: Vec::new(),
            properties: Vec::new(),
        };
        self.layers.push(Layer::Object(layer));
        self.layers.len() - 1
    }

    /// Remove a layer by index. Returns the removed layer or `None`.
    pub fn remove_layer(&mut self, index: usize) -> Option<Layer> {
        if index < self.layers.len() {
            Some(self.layers.remove(index))
        } else {
            None
        }
    }

    /// Swap two layers for reordering.
    pub fn swap_layers(&mut self, a: usize, b: usize) {
        if a < self.layers.len() && b < self.layers.len() {
            self.layers.swap(a, b);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileReference<'a> {
    pub tileset_index: usize,
    pub tileset: &'a TilesetReference,
    pub local_id: u32,
}

impl TileReference<'_> {
    /// Returns the local tile ID to render at the given elapsed time (seconds).
    /// If this tile has an animation, cycles through frames; otherwise returns `local_id`.
    pub fn animated_local_id(&self, elapsed_secs: f64) -> u32 {
        let Some(frames) = self.tileset.tileset.animations.get(&self.local_id) else {
            return self.local_id;
        };
        if frames.is_empty() {
            return self.local_id;
        }
        let total_ms: u64 = frames.iter().map(|f| u64::from(f.duration_ms)).sum();
        if total_ms == 0 {
            return self.local_id;
        }
        let elapsed_ms = (elapsed_secs * 1000.0) as u64 % total_ms;
        let mut acc = 0u64;
        for frame in frames {
            acc += u64::from(frame.duration_ms);
            if elapsed_ms < acc {
                return frame.tile_id;
            }
        }
        frames.last().map_or(self.local_id, |f| f.tile_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorDocument {
    pub file_path: PathBuf,
    pub map: Map,
}
