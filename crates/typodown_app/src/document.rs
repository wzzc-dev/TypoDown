use std::path::{Path, PathBuf};

const DEFAULT_DOCUMENT_NAME: &str = "Untitled.md";
pub const DEFAULT_DOCUMENT_TEXT: &str = "# Untitled\n\nStart writing here...";

#[derive(Debug, Clone)]
pub struct OpenDocument {
    path: Option<PathBuf>,
    last_saved_text: String,
}

impl OpenDocument {
    pub fn unsaved(initial_text: impl Into<String>) -> Self {
        Self {
            path: None,
            last_saved_text: initial_text.into(),
        }
    }

    pub fn from_disk(path: PathBuf, text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            path: Some(path),
            last_saved_text: text,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn title(&self) -> String {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or(DEFAULT_DOCUMENT_NAME)
            .to_string()
    }

    pub fn suggested_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or(DEFAULT_DOCUMENT_NAME)
            .to_string()
    }

    pub fn is_dirty(&self, current_text: &str) -> bool {
        self.last_saved_text != current_text
    }

    pub fn mark_saved(&mut self, path: PathBuf, text: impl Into<String>) {
        self.path = Some(path);
        self.last_saved_text = text.into();
    }

}

#[derive(Debug, Clone)]
pub struct EditorPreferences {
    pub line_number: bool,
    pub indent_guides: bool,
    pub soft_wrap: bool,
    pub autocorrect_enabled: bool,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            line_number: true,
            indent_guides: true,
            soft_wrap: false,
            autocorrect_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{DEFAULT_DOCUMENT_TEXT, OpenDocument};

    #[test]
    fn dirty_state_changes_after_edit_and_save() {
        let mut document = OpenDocument::unsaved(DEFAULT_DOCUMENT_TEXT);
        assert!(!document.is_dirty(DEFAULT_DOCUMENT_TEXT));
        assert!(document.is_dirty("# Changed"));

        document.mark_saved(PathBuf::from("notes.md"), "# Changed");
        assert!(!document.is_dirty("# Changed"));
        assert_eq!(document.title(), "notes.md");
    }
}
