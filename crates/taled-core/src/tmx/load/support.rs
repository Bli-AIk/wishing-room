use crate::error::{EditorError, Result, unsupported};
use crate::model::RenderOrder;
use roxmltree::{Document, Node};
use std::io;
use std::path::{Component, Path, PathBuf};

pub(super) fn parse_document(xml: &str) -> Result<Document<'_>> {
    Document::parse(xml).map_err(|err| EditorError::XmlParse(err.to_string()))
}

pub(super) fn fallback_layer_name(name: &str, fallback: &str) -> String {
    if name.is_empty() {
        fallback.to_string()
    } else {
        name.to_string()
    }
}

pub(super) fn relativize_child_path(parent_file: &Path, child_path: &Path) -> PathBuf {
    parent_file
        .parent()
        .and_then(|parent| child_path.strip_prefix(parent).ok())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| child_path.to_path_buf())
}

pub(super) fn to_io_error(err: EditorError) -> io::Error {
    io::Error::other(err.to_string())
}

pub(super) fn required_attr<'a>(node: Node<'a, '_>, name: &str) -> Result<&'a str> {
    node.attribute(name)
        .ok_or_else(|| EditorError::Invalid(format!("missing attribute '{name}'")))
}

pub(super) fn parse_required_u32(node: Node<'_, '_>, name: &str) -> Result<u32> {
    required_attr(node, name)?
        .parse()
        .map_err(|_| EditorError::Invalid(format!("cannot parse '{name}' as u32")))
}

pub(super) fn parse_optional_u32(node: Node<'_, '_>, name: &str) -> Result<Option<u32>> {
    match node.attribute(name) {
        Some(value) => value
            .parse()
            .map(Some)
            .map_err(|_| EditorError::Invalid(format!("cannot parse '{name}' as u32"))),
        None => Ok(None),
    }
}

pub(super) fn parse_bool_attr(node: Node<'_, '_>, name: &str, default: bool) -> Result<bool> {
    match node.attribute(name) {
        None => Ok(default),
        Some("0") | Some("false") => Ok(false),
        Some("1") | Some("true") => Ok(true),
        Some(value) => Err(EditorError::Invalid(format!(
            "cannot parse '{name}' boolean value '{value}'"
        ))),
    }
}

pub(super) fn parse_render_order(value: &str) -> Result<RenderOrder> {
    match value {
        "right-down" => Ok(RenderOrder::RightDown),
        "right-up" => Ok(RenderOrder::RightUp),
        "left-down" => Ok(RenderOrder::LeftDown),
        "left-up" => Ok(RenderOrder::LeftUp),
        value => Err(EditorError::Invalid(format!(
            "unsupported renderorder '{value}'"
        ))),
    }
}

pub(super) fn reject_attr_if_present(node: Node<'_, '_>, attr: &str, scope: &str) -> Result<()> {
    if node.attribute(attr).is_some() {
        return Err(unsupported(
            scope,
            format!("attribute '{attr}' is out of stage-1 scope"),
        ));
    }
    Ok(())
}

pub(super) fn reject_non_default_f32(
    node: Node<'_, '_>,
    attr: &str,
    default: f32,
    scope: &str,
) -> Result<()> {
    let Some(value) = node.attribute(attr) else {
        return Ok(());
    };
    let parsed: f32 = value
        .parse()
        .map_err(|_| EditorError::Invalid(format!("cannot parse '{attr}' as f32")))?;
    if (parsed - default).abs() > f32::EPSILON {
        return Err(unsupported(
            scope,
            format!("attribute '{attr}' is out of stage-1 scope"),
        ));
    }
    Ok(())
}

pub(super) fn inject_root_attributes(xml: &str, attributes: &[(&str, String)]) -> Result<String> {
    let start = xml.find("<tileset").ok_or_else(|| {
        EditorError::Invalid("could not find <tileset> start tag while patching xml".to_string())
    })?;
    let end = find_tag_end(xml, start).ok_or_else(|| {
        EditorError::Invalid("could not find <tileset> end tag while patching xml".to_string())
    })?;

    let mut patched = String::with_capacity(xml.len() + attributes.len() * 24);
    patched.push_str(&xml[..end]);
    for (name, value) in attributes {
        patched.push(' ');
        patched.push_str(name);
        patched.push_str("=\"");
        patched.push_str(value);
        patched.push('"');
    }
    patched.push_str(&xml[end..]);
    Ok(patched)
}

fn find_tag_end(xml: &str, start: usize) -> Option<usize> {
    let bytes = xml.as_bytes();
    let mut quote = None;
    let mut index = start;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' | b'"' => match quote {
                Some(current) if current == bytes[index] => quote = None,
                None => quote = Some(bytes[index]),
                _ => {}
            },
            b'>' if quote.is_none() => return Some(index),
            _ => {}
        }
        index += 1;
    }
    None
}

pub(super) fn normalize_path(path: &Path) -> PathBuf {
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
