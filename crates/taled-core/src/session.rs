use crate::error::Result;
use crate::model::EditorDocument;
use crate::tmx;
use base64::Engine;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
enum AssetSource {
    FileSystem,
    Embedded(EmbeddedAssets),
}

#[derive(Debug, Clone, Default)]
struct EmbeddedAssets {
    files: BTreeMap<PathBuf, Vec<u8>>,
}

impl EmbeddedAssets {
    fn from_files<I, K, V>(files: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<PathBuf>,
        V: Into<Vec<u8>>,
    {
        Self {
            files: files
                .into_iter()
                .map(|(path, bytes)| (normalize_virtual_path(&path.into()), bytes.into()))
                .collect(),
        }
    }

    fn read_bytes(&self, path: &Path) -> Result<&[u8]> {
        let normalized = normalize_virtual_path(path);
        self.files
            .get(&normalized)
            .map(Vec::as_slice)
            .ok_or_else(|| {
                crate::error::EditorError::Invalid(format!(
                    "missing embedded asset '{}'",
                    normalized.display()
                ))
            })
    }

    fn read_text(&self, path: &Path) -> Result<String> {
        let bytes = self.read_bytes(path)?;
        String::from_utf8(bytes.to_vec()).map_err(|err| {
            crate::error::EditorError::Invalid(format!(
                "embedded asset '{}' is not valid utf-8: {err}",
                normalize_virtual_path(path).display()
            ))
        })
    }
}

#[derive(Debug, Clone)]
pub struct EditorSession {
    document: EditorDocument,
    undo_stack: Vec<EditorDocument>,
    redo_stack: Vec<EditorDocument>,
    history_batch: Option<EditorDocument>,
    asset_source: AssetSource,
    dirty: bool,
}

impl EditorSession {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let file_path = path.as_ref().to_path_buf();
        let map = tmx::load_map(&file_path)?;
        Ok(Self {
            document: EditorDocument { file_path, map },
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            history_batch: None,
            asset_source: AssetSource::FileSystem,
            dirty: false,
        })
    }

    pub fn load_embedded<P, I, K, V>(path: P, files: I) -> Result<Self>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = (K, V)>,
        K: Into<PathBuf>,
        V: Into<Vec<u8>>,
    {
        let file_path = normalize_virtual_path(path.as_ref());
        let assets = EmbeddedAssets::from_files(files);
        let xml = assets.read_text(&file_path)?;
        let map = tmx::load_map_from_str(&file_path, &xml, &|source_path| {
            assets.read_text(source_path)
        })?;

        Ok(Self {
            document: EditorDocument { file_path, map },
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            history_batch: None,
            asset_source: AssetSource::Embedded(assets),
            dirty: false,
        })
    }

    pub fn document(&self) -> &EditorDocument {
        &self.document
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn save(&mut self) -> Result<()> {
        if !matches!(self.asset_source, AssetSource::FileSystem) {
            return Err(crate::error::unsupported(
                "io.save",
                "embedded demo maps cannot be saved in place",
            ));
        }
        tmx::save_map(&self.document.file_path, &self.document.map)?;
        self.dirty = false;
        Ok(())
    }

    pub fn save_as(&mut self, path: impl AsRef<Path>) -> Result<()> {
        if !matches!(self.asset_source, AssetSource::FileSystem) {
            return Err(crate::error::unsupported(
                "io.save_as",
                "embedded demo maps cannot be written from the web preview yet",
            ));
        }
        let path = path.as_ref().to_path_buf();
        tmx::save_map(&path, &self.document.map)?;
        self.document.file_path = path;
        self.dirty = false;
        Ok(())
    }

    pub fn edit<F>(&mut self, edit: F) -> Result<()>
    where
        F: FnOnce(&mut EditorDocument) -> Result<()>,
    {
        let snapshot = self.document.clone();
        edit(&mut self.document)?;
        if self.history_batch.is_some() {
            if self.document != snapshot {
                self.dirty = true;
            }
            return Ok(());
        }

        if self.document != snapshot {
            self.undo_stack.push(snapshot);
            self.redo_stack.clear();
            self.dirty = true;
        }
        Ok(())
    }

    pub fn begin_history_batch(&mut self) {
        if self.history_batch.is_none() {
            self.history_batch = Some(self.document.clone());
        }
    }

    pub fn finish_history_batch(&mut self) -> bool {
        let Some(snapshot) = self.history_batch.take() else {
            return false;
        };

        if snapshot == self.document {
            return false;
        }

        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
        self.dirty = true;
        true
    }

    pub fn abort_history_batch(&mut self) {
        self.history_batch = None;
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo_stack.pop() else {
            return false;
        };

        let snapshot = self.document.clone();
        self.redo_stack.push(snapshot);
        self.document = previous;
        self.dirty = true;
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo_stack.pop() else {
            return false;
        };

        let snapshot = self.document.clone();
        self.undo_stack.push(snapshot);
        self.document = next;
        self.dirty = true;
        true
    }

    pub fn tileset_image_data_uri(&self, tileset_index: usize) -> Result<String> {
        let tileset = self
            .document
            .map
            .tilesets
            .get(tileset_index)
            .ok_or_else(|| {
                crate::error::EditorError::Invalid(format!(
                    "unknown tileset index: {tileset_index}"
                ))
            })?;
        let image_path = tileset.resolved_image_path(&self.document.file_path);
        let bytes = match &self.asset_source {
            AssetSource::FileSystem => fs::read(&image_path)?,
            AssetSource::Embedded(assets) => assets.read_bytes(&image_path)?.to_vec(),
        };
        let mime = match image_path.extension().and_then(|ext| ext.to_str()) {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            _ => "application/octet-stream",
        };

        Ok(format!(
            "data:{mime};base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ))
    }

    pub fn sample_path_from_root(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref()
            .join("assets")
            .join("samples")
            .join("stage1-basic")
            .join("map.tmx")
    }
}

fn normalize_virtual_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
