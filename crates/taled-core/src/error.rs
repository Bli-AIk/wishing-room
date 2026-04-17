use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportIssue {
    pub scope: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedFeatures(pub Vec<SupportIssue>);

impl fmt::Display for UnsupportedFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, issue) in self.0.iter().enumerate() {
            if index > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{}: {}", issue.scope, issue.reason)?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("tiled parse error: {0}")]
    Tiled(#[from] tiled::Error),
    #[error("xml write error: {0}")]
    XmlWrite(#[from] quick_xml::Error),
    #[error("xml parse error: {0}")]
    XmlParse(String),
    #[error("invalid data: {0}")]
    Invalid(String),
    #[error("unsupported feature(s): {0}")]
    Unsupported(UnsupportedFeatures),
}

pub type Result<T> = std::result::Result<T, EditorError>;

pub fn unsupported(scope: impl Into<String>, reason: impl Into<String>) -> EditorError {
    EditorError::Unsupported(UnsupportedFeatures(vec![SupportIssue {
        scope: scope.into(),
        reason: reason.into(),
    }]))
}
