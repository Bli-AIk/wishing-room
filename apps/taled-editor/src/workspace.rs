use std::fs;
use std::path::{Path, PathBuf};

use crate::embedded_samples::embedded_sample_assets;
use crate::platform;

pub(crate) const BUILTIN_WORKSPACE: &str = "builtin";

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceInfo {
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) path: PathBuf,
    #[allow(dead_code)]
    pub(crate) map_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct MapFileInfo {
    pub(crate) path: PathBuf,
    pub(crate) file_name: String,
    pub(crate) size_bytes: u64,
}

/// Root directory for all workspaces: `<files_dir>/workspaces/`.
pub(crate) fn workspaces_root() -> Option<PathBuf> {
    platform::files_dir().map(|d| Path::new(&d).join("workspaces"))
}

/// Ensure the builtin workspace exists and contains embedded sample files.
/// Only writes files that don't already exist (idempotent).
pub(crate) fn ensure_builtin_workspace() -> Option<PathBuf> {
    let root = workspaces_root()?;
    let builtin = root.join(BUILTIN_WORKSPACE);
    if fs::create_dir_all(&builtin).is_err() {
        return None;
    }

    for (rel_path, data) in embedded_sample_assets() {
        let dest = builtin.join(rel_path);
        if dest.exists() {
            continue;
        }
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&dest, data);
    }

    Some(builtin)
}

/// List all workspaces (subdirectories of the workspaces root).
pub(crate) fn list_workspaces() -> Vec<WorkspaceInfo> {
    let Some(root) = workspaces_root() else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(&root) else {
        return Vec::new();
    };

    let mut workspaces = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let map_count = count_tmx_files(&path);
        workspaces.push(WorkspaceInfo {
            name,
            path,
            map_count,
        });
    }

    // Put builtin first, then sort the rest alphabetically.
    workspaces.sort_by(|a, b| {
        let a_builtin = a.name == BUILTIN_WORKSPACE;
        let b_builtin = b.name == BUILTIN_WORKSPACE;
        b_builtin.cmp(&a_builtin).then_with(|| a.name.cmp(&b.name))
    });
    workspaces
}

/// List all .tmx files inside a workspace directory (recursive).
pub(crate) fn list_maps(workspace_name: &str) -> Vec<MapFileInfo> {
    let Some(root) = workspaces_root() else {
        return Vec::new();
    };
    let ws_path = root.join(workspace_name);
    let mut maps = Vec::new();
    collect_tmx_files(&ws_path, &mut maps);
    maps.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    maps
}

/// Create a new empty workspace directory. Returns the path on success.
pub(crate) fn create_workspace(name: &str) -> Option<PathBuf> {
    let root = workspaces_root()?;
    let ws_path = root.join(name);
    if ws_path.exists() {
        return None;
    }
    fs::create_dir_all(&ws_path).ok()?;
    Some(ws_path)
}

fn count_tmx_files(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_tmx_files(&path);
            } else if matches!(path.extension(), Some(ext) if ext == "tmx") {
                count += 1;
            }
        }
    }
    count
}

fn collect_tmx_files(dir: &Path, out: &mut Vec<MapFileInfo>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_tmx_files(&path, out);
        } else if matches!(path.extension(), Some(ext) if ext == "tmx") {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
            out.push(MapFileInfo {
                path,
                file_name,
                size_bytes,
            });
        }
    }
}

// ── Import functions ────────────────────────────────────────────────

/// Import a directory as a new workspace. Copies everything recursively.
/// Returns the workspace name on success.
pub(crate) fn import_directory_as_workspace(source_dir: &Path) -> Option<String> {
    crate::logging::append(&format!(
        "import_directory_as_workspace: source={source_dir:?} exists={} is_dir={}",
        source_dir.exists(),
        source_dir.is_dir(),
    ));
    if !source_dir.is_dir() {
        crate::logging::append("import_directory_as_workspace: not a directory, aborting");
        return None;
    }
    let ws_name = source_dir
        .file_name()?
        .to_string_lossy()
        .into_owned();

    let root = workspaces_root()?;
    let mut dest = root.join(&ws_name);
    let mut final_name = ws_name.clone();

    // If the workspace already exists, append a number suffix.
    if dest.exists() {
        let mut n = 2u32;
        loop {
            final_name = format!("{ws_name}-{n}");
            dest = root.join(&final_name);
            if !dest.exists() {
                break;
            }
            n += 1;
        }
    }

    crate::logging::append(&format!(
        "import_directory_as_workspace: copying to {dest:?} as '{final_name}'",
    ));
    match copy_dir_recursive(source_dir, &dest) {
        Ok(()) => {
            crate::logging::append("import_directory_as_workspace: copy success");
            Some(final_name)
        }
        Err(e) => {
            crate::logging::append(&format!("import_directory_as_workspace: copy failed: {e}"));
            None
        }
    }
}

/// Import a single TMX file and its referenced assets into a workspace.
/// Scans the TMX for `<tileset source="...">` and each TSX for
/// `<image source="...">`, then copies all referenced files.
pub(crate) fn import_tmx_to_workspace(
    tmx_path: &Path,
    workspace_name: &str,
) -> Option<PathBuf> {
    let root = workspaces_root()?;
    let ws_dir = root.join(workspace_name);
    fs::create_dir_all(&ws_dir).ok()?;

    let tmx_parent = tmx_path.parent().unwrap_or_else(|| Path::new("."));
    let tmx_content = fs::read_to_string(tmx_path).ok()?;

    // Copy the TMX file itself.
    let tmx_name = tmx_path.file_name()?;
    let dest_tmx = ws_dir.join(tmx_name);
    fs::copy(tmx_path, &dest_tmx).ok()?;

    // Find and copy TSX files + their image assets.
    let tsx_sources = extract_attr_values(&tmx_content, "tileset", "source");
    for tsx_rel in &tsx_sources {
        let tsx_abs = tmx_parent.join(tsx_rel);
        if !tsx_abs.is_file() {
            continue;
        }
        copy_file_preserving_rel(&tsx_abs, tsx_rel, &ws_dir);
        import_tsx_images(&tsx_abs, tsx_rel, &ws_dir);
    }

    Some(dest_tmx)
}

/// Copy images referenced by a TSX file into the workspace.
fn import_tsx_images(tsx_abs: &Path, tsx_rel: &str, ws_dir: &Path) {
    let Ok(tsx_content) = fs::read_to_string(tsx_abs) else {
        return;
    };
    let tsx_parent = tsx_abs.parent().unwrap_or_else(|| Path::new("."));
    let tsx_rel_parent = Path::new(tsx_rel)
        .parent()
        .unwrap_or_else(|| Path::new("."));

    for img_rel in extract_attr_values(&tsx_content, "image", "source") {
        let img_abs = tsx_parent.join(&img_rel);
        if !img_abs.is_file() {
            continue;
        }
        let img_dest_rel = tsx_rel_parent.join(&img_rel);
        copy_file_preserving_rel(&img_abs, &img_dest_rel.to_string_lossy(), ws_dir);
    }
}

/// Copy a single file into `ws_dir` at the given relative path, creating
/// parent directories as needed.
fn copy_file_preserving_rel(src: &Path, rel_path: &str, ws_dir: &Path) {
    let dest = ws_dir.join(rel_path);
    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::copy(src, &dest);
}

// ── Helpers ─────────────────────────────────────────────────────────

/// Recursively copy a directory tree.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Extract attribute values from XML-like text.
/// e.g. `extract_attr_values(xml, "tileset", "source")` finds all
/// `<tileset ... source="VALUE" ...>` and returns the VALUEs.
fn extract_attr_values(xml: &str, tag: &str, attr: &str) -> Vec<String> {
    let mut results = Vec::new();
    let tag_open = format!("<{tag}");
    let attr_pat = format!("{attr}=\"");

    let mut search_from = 0;
    while let Some(tag_pos) = xml[search_from..].find(&tag_open) {
        let abs_tag = search_from + tag_pos;
        let tag_end = xml[abs_tag..].find('>').map(|p| abs_tag + p);
        let region_end = tag_end.unwrap_or(xml.len());
        let tag_region = &xml[abs_tag..region_end];

        if let Some(attr_pos) = tag_region.find(&attr_pat) {
            let val_start = attr_pos + attr_pat.len();
            if let Some(val_end) = tag_region[val_start..].find('"') {
                results.push(tag_region[val_start..val_start + val_end].to_string());
            }
        }

        search_from = region_end + 1;
    }
    results
}

/// Process a completed SAF directory picker result.
/// The result string is `"mode:path"` where mode is "workspace" or "tmx".
/// The Java side has already copied the tree to `<files_dir>/import/<name>`.
pub(crate) fn handle_import_result(
    state: &mut crate::app_state::AppState,
    result: &str,
) {
    use crate::app_state::ImportMode;

    // Parse "mode:path" format from Java side.
    let (mode, import_path) = if let Some(rest) = result.strip_prefix("workspace:") {
        (ImportMode::Workspace, rest)
    } else if let Some(rest) = result.strip_prefix("tmx:") {
        (ImportMode::Tmx, rest)
    } else {
        // Fallback: treat as workspace import.
        (ImportMode::Workspace, result)
    };

    // Clear any stale pending state.
    state.import_pending = None;

    let lang = state.resolved_language();
    let path = Path::new(import_path);

    crate::logging::append(&format!(
        "handle_import_result: mode={} path={import_path} exists={} is_dir={}",
        match mode {
            ImportMode::Workspace => "Workspace",
            ImportMode::Tmx => "Tmx",
        },
        path.exists(),
        path.is_dir(),
    ));

    match mode {
        ImportMode::Workspace => {
            if let Some(name) = import_directory_as_workspace(path) {
                state.workspace_list = list_workspaces()
                    .into_iter()
                    .map(|w| w.name)
                    .collect();
                state.active_workspace = name.clone();
                state.status = format!(
                    "{} '{name}'",
                    crate::l10n::text(lang, "import-workspace-done"),
                );
                crate::logging::append(&format!(
                    "handle_import_result: workspace imported as '{name}'"
                ));
            } else {
                state.status = "Import failed".to_string();
                crate::logging::append("handle_import_result: import_directory_as_workspace failed");
            }
        }
        ImportMode::Tmx => {
            let mut count = 0u32;
            let mut tmx_files = Vec::new();
            collect_tmx_files(path, &mut tmx_files);
            crate::logging::append(&format!(
                "handle_import_result: found {} TMX files",
                tmx_files.len()
            ));
            for map in &tmx_files {
                if import_tmx_to_workspace(&map.path, &state.active_workspace).is_some()
                {
                    count += 1;
                }
            }
            state.status = format!(
                "{count} {}",
                crate::l10n::text(lang, "import-tmx-done"),
            );
            crate::logging::append(&format!(
                "handle_import_result: imported {count} TMX files"
            ));
        }
    }

    // Clean up the staging copy.
    crate::logging::append(&format!("handle_import_result: cleaning up {import_path}"));
    let _ = fs::remove_dir_all(path);
}
