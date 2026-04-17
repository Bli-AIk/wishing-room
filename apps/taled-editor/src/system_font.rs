/// System font discovery for CJK text on Android.
///
/// Uses `fontdb` to scan `/system/fonts/` and find a suitable CJK font,
/// avoiding the need to bundle a 19 MB font file in the APK.
use ply_engine::prelude::FontAsset;

/// CJK font family names in priority order.
const CJK_FAMILIES: &[&str] = &[
    "Noto Sans CJK SC",
    "Noto Sans SC",
    "Noto Sans CJK",
    "Noto Sans CJK TC",
    "Noto Sans CJK JP",
    "Noto Sans CJK KR",
    "Droid Sans Fallback",
    "Source Han Sans SC",
    "Source Han Sans",
];

/// Attempts to find and load a CJK font from the system.
///
/// On Android, scans `/system/fonts/`. On desktop Linux, scans standard
/// system font directories. Returns `None` if no CJK font is found.
///
/// The returned reference is intentionally leaked for the `'static` lifetime
/// that [`FontAsset`] requires — the font data stays loaded for the entire
/// application lifetime anyway.
pub fn find_system_cjk_font() -> Option<&'static FontAsset> {
    let mut db = fontdb::Database::new();

    #[cfg(target_os = "android")]
    db.load_fonts_dir("/system/fonts");

    #[cfg(not(target_os = "android"))]
    db.load_system_fonts();

    let families: Vec<fontdb::Family<'_>> = CJK_FAMILIES
        .iter()
        .map(|name| fontdb::Family::Name(name))
        .collect();

    let query = fontdb::Query {
        families: &families,
        ..Default::default()
    };

    let id = db.query(&query)?;

    let font_data = db.with_face_data(id, |data, _index| data.to_vec())?;

    let leaked_data: &'static [u8] = Box::leak(font_data.into_boxed_slice());

    let asset = Box::new(FontAsset::Bytes {
        file_name: "system-cjk-font.ttc",
        data: leaked_data,
    });

    Some(Box::leak(asset))
}
