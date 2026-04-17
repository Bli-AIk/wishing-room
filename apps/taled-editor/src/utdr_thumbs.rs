use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use ply_engine::prelude::{FilterMode, Texture2D};

/// Max concurrent thumbnail downloads at once.
const MAX_CONCURRENT: usize = 6;

/// Completed downloads waiting to be turned into textures on the main thread.
static MAILBOX: LazyLock<Mutex<Vec<(String, Vec<u8>)>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// Number of active download threads.
static ACTIVE: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(0));

#[derive(Clone)]
enum Entry {
    Pending,
    Ready(Texture2D),
    Failed,
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Entry>> = RefCell::new(HashMap::new());
}

/// Build the raw.githubusercontent URL for a room thumbnail.
fn thumb_url(repo: &str, branch: &str, game_key: &str, room_name: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/{repo}/{branch}/thumbnails/{game_key}/{room_name}.jpg",
    )
}

/// Request a thumbnail, spawning a download if needed. Returns texture if ready.
pub(crate) fn get(game_key: &str, room_name: &str, repo: &str, branch: &str) -> Option<Texture2D> {
    let key = format!("{game_key}/{room_name}");
    CACHE.with(|c| {
        let cache = c.borrow();
        match cache.get(&key) {
            Some(Entry::Ready(tex)) => return Some(tex.clone()),
            Some(Entry::Pending | Entry::Failed) => return None,
            None => {}
        }
        drop(cache);

        // Not in cache — mark as pending and start download if under limit.
        c.borrow_mut().insert(key.clone(), Entry::Pending);
        let url = thumb_url(repo, branch, game_key, room_name);
        let mut active = ACTIVE.lock().unwrap();
        if *active < MAX_CONCURRENT {
            *active += 1;
            drop(active);
            spawn_download(key, url);
        }
        // If at limit, it stays Pending and will be retried later.
        None
    })
}

/// Poll mailbox and convert completed downloads to textures.
pub(crate) fn poll() {
    let ready: Vec<(String, Vec<u8>)> = {
        let mut mbox = MAILBOX.lock().unwrap();
        std::mem::take(&mut *mbox)
    };
    if ready.is_empty() {
        return;
    }
    CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        for (key, bytes) in ready {
            if bytes.is_empty() {
                cache.insert(key, Entry::Failed);
            } else {
                let tex = Texture2D::from_file_with_format(&bytes, None);
                tex.set_filter(FilterMode::Linear);
                cache.insert(key, Entry::Ready(tex));
            }
        }
    });
}

fn spawn_download(key: String, url: String) {
    std::thread::spawn(move || {
        let bytes = fetch_bytes_quiet(&url);
        if let Ok(mut mbox) = MAILBOX.lock() {
            mbox.push((key, bytes));
        }
        if let Ok(mut active) = ACTIVE.lock() {
            *active = active.saturating_sub(1);
        }
    });
}

fn fetch_bytes_quiet(url: &str) -> Vec<u8> {
    let resp = match ureq::get(url).call() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut buf = Vec::new();
    if resp.into_reader().read_to_end(&mut buf).is_err() {
        return Vec::new();
    }
    buf
}
