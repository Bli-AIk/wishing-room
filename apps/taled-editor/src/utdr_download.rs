use std::path::{Path, PathBuf};
use std::sync::mpsc;

use crate::app_state::{AppState, MobileScreen};

// ── Types ──────────────────────────────────────────────────────────

pub(crate) enum DownloadMsg {
    Progress(String),
    Downloaded(PathBuf),
    Error(String),
}

#[derive(Clone)]
pub(crate) enum DownloadStatus {
    InProgress(String),
    Error(String),
}

// ── Start download ─────────────────────────────────────────────────

pub(crate) fn start_room_download(
    state: &mut AppState,
    room_path: &str,
    repo: &str,
    branch: &str,
) {
    if state.download_rx.is_some() {
        state.status = "Download already in progress".to_string();
        return;
    }

    let files_dir = match crate::platform::files_dir() {
        Some(d) => d,
        None => {
            state.status = "Cannot determine app storage".to_string();
            return;
        }
    };

    let (tx, rx) = mpsc::channel();
    let path = room_path.to_string();
    let repo = repo.to_string();
    let branch = branch.to_string();

    state.download_rx = Some(rx);
    state.download_status = Some(DownloadStatus::InProgress("Starting...".into()));

    std::thread::spawn(move || {
        if let Err(e) = download_thread(&tx, &path, &repo, &branch, &files_dir) {
            let _ = tx.send(DownloadMsg::Error(e));
        }
    });
}

// ── Poll ───────────────────────────────────────────────────────────

pub(crate) fn poll_download(state: &mut AppState) {
    let msgs: Vec<_> = state
        .download_rx
        .as_ref()
        .map(|rx| rx.try_iter().collect())
        .unwrap_or_default();

    for msg in msgs {
        match msg {
            DownloadMsg::Progress(text) => {
                state.download_status = Some(DownloadStatus::InProgress(text.clone()));
                state.status = text;
            }
            DownloadMsg::Downloaded(temp_dir) => handle_downloaded(state, &temp_dir),
            DownloadMsg::Error(err) => {
                state.download_rx = None;
                state.download_status = Some(DownloadStatus::Error(err.clone()));
                state.status = format!("Download failed: {err}");
            }
        }
    }
}

fn handle_downloaded(state: &mut AppState, temp_dir: &Path) {
    state.download_rx = None;
    let tmx = find_tmx_in_dir(temp_dir);
    let imported = tmx.and_then(|t| {
        crate::workspace::import_tmx_to_workspace(&t, &state.active_workspace)
    });
    let _ = std::fs::remove_dir_all(temp_dir);
    match imported {
        Some(dest) => {
            state.download_status = None;
            let path_str = dest.to_string_lossy().to_string();
            if crate::session_ops::load_filesystem_map(state, &path_str) {
                state.navigate(MobileScreen::Editor);
            }
        }
        None => {
            state.download_status =
                Some(DownloadStatus::Error("Import failed".into()));
        }
    }
}

fn find_tmx_in_dir(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().is_some_and(|e| e == "tmx") {
            return Some(p);
        }
    }
    None
}

// ── Download thread ────────────────────────────────────────────────

fn download_thread(
    tx: &mpsc::Sender<DownloadMsg>,
    tmx_rel_path: &str,
    repo: &str,
    branch: &str,
    files_dir: &str,
) -> Result<(), String> {
    let base_url = format!("https://raw.githubusercontent.com/{repo}/{branch}");

    let _ = tx.send(DownloadMsg::Progress("Downloading map...".into()));
    let tmx_url = format!("{base_url}/{tmx_rel_path}");
    let tmx_content = fetch_text(&tmx_url)?;

    let temp_dir = Path::new(files_dir).join("utdr_import_tmp");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let tmx_filename = Path::new(tmx_rel_path)
        .file_name()
        .unwrap_or_default();
    let tmx_local = temp_dir.join(tmx_filename);
    std::fs::write(&tmx_local, &tmx_content).map_err(|e| e.to_string())?;

    let tmx_parent = Path::new(tmx_rel_path)
        .parent()
        .unwrap_or(Path::new(""));
    let tsx_sources = extract_attr_values(&tmx_content, "tileset", "source");

    for tsx_rel in &tsx_sources {
        let _ = tx.send(DownloadMsg::Progress(format!("Tileset: {tsx_rel}")));
        let tsx_repo_path = tmx_parent.join(tsx_rel);
        let tsx_url = format!("{base_url}/{}", tsx_repo_path.display());
        let tsx_content = match fetch_text(&tsx_url) {
            Ok(c) => c,
            Err(_) => continue,
        };
        save_relative(&temp_dir, tsx_rel, tsx_content.as_bytes());

        let tsx_parent = Path::new(tsx_rel)
            .parent()
            .unwrap_or(Path::new(""));
        let img_sources = extract_attr_values(&tsx_content, "image", "source");

        for img_rel in &img_sources {
            let _ = tx.send(DownloadMsg::Progress(format!("Image: {img_rel}")));
            let full_rel = tsx_parent.join(img_rel);
            let img_repo_path = tmx_parent.join(&full_rel);
            let img_url = format!("{base_url}/{}", img_repo_path.display());
            let img_bytes = match fetch_bytes(&img_url) {
                Ok(b) => b,
                Err(_) => continue,
            };
            save_relative(&temp_dir, &full_rel.to_string_lossy(), &img_bytes);
        }
    }

    let _ = tx.send(DownloadMsg::Downloaded(temp_dir));
    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────

fn save_relative(base: &Path, rel: &str, data: &[u8]) {
    let dest = base.join(rel);
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&dest, data);
}

fn fetch_text(url: &str) -> Result<String, String> {
    ureq::get(url)
        .call()
        .map_err(|e| format!("{e}"))?
        .into_string()
        .map_err(|e| format!("{e}"))
}

fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    ureq::get(url)
        .call()
        .map_err(|e| format!("{e}"))?
        .into_reader()
        .read_to_end(&mut buf)
        .map_err(|e| format!("{e}"))?;
    Ok(buf)
}

fn extract_attr_values(xml: &str, tag: &str, attr: &str) -> Vec<String> {
    let mut results = Vec::new();
    let tag_open = format!("<{tag}");
    let attr_pat = format!("{attr}=\"");
    let mut pos = 0;
    while let Some(tp) = xml[pos..].find(&tag_open) {
        let abs = pos + tp;
        let end = xml[abs..].find('>').map_or(xml.len(), |p| abs + p);
        let region = &xml[abs..end];
        if let Some(ap) = region.find(&attr_pat) {
            let vs = ap + attr_pat.len();
            if let Some(ve) = region[vs..].find('"') {
                results.push(region[vs..vs + ve].to_string());
            }
        }
        pos = end + 1;
    }
    results
}
