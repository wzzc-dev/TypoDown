use std::{
    fs,
    path::{Path, PathBuf},
};

use autocorrect::ignorer::Ignorer;
use directories::UserDirs;
use gpui_component::tree::TreeItem;

use crate::document::DEFAULT_DOCUMENT_TEXT;

#[derive(Debug, Clone)]
pub struct WorkspaceRoot {
    path: PathBuf,
}

impl WorkspaceRoot {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default() -> Self {
        Self::new(default_workspace_path())
    }

    pub fn from_document(path: &Path) -> Self {
        let root = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(default_workspace_path);
        Self::new(root)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }

    pub fn ensure_fallback_document(&self) -> std::io::Result<PathBuf> {
        fs::create_dir_all(&self.path)?;
        let path = self.path.join("Untitled.md");
        if !path.exists() {
            fs::write(&path, DEFAULT_DOCUMENT_TEXT)?;
        }
        Ok(path)
    }

    pub fn tree_items(&self) -> Vec<TreeItem> {
        let ignorer = Ignorer::new(&self.path.to_string_lossy());
        build_file_items(&ignorer, &self.path, &self.path)
    }
}

pub fn default_workspace_path() -> PathBuf {
    if let Some(user_dirs) = UserDirs::new()
        && let Some(documents_dir) = user_dirs.document_dir()
    {
        return documents_dir.join("TypoDown");
    }

    PathBuf::from("./")
}

fn build_file_items(ignorer: &Ignorer, root: &Path, path: &Path) -> Vec<TreeItem> {
    let mut items = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        let mut entries = entries.flatten().collect::<Vec<_>>();
        entries.sort_by(|a, b| {
            let a_path = a.path();
            let b_path = b.path();
            match (a_path.is_dir(), b_path.is_dir()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in entries {
            let path = entry.path();
            let relative_path = path.strip_prefix(root).unwrap_or(&path);
            if ignorer.is_ignored(&relative_path.to_string_lossy())
                || relative_path.ends_with(".git")
            {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown")
                .to_string();
            let id = path.to_string_lossy().to_string();

            if path.is_dir() {
                let children = build_file_items(ignorer, root, &path);
                items.push(TreeItem::new(id, file_name).children(children));
            } else {
                items.push(TreeItem::new(id, file_name));
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use autocorrect::ignorer::Ignorer;

    use super::{WorkspaceRoot, build_file_items};

    fn temp_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("typodown-workspace-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn builds_sorted_tree_and_ignores_git_directory() {
        let root = temp_dir("tree");
        fs::create_dir_all(root.join("notes")).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join("b.md"), "b").unwrap();
        fs::write(root.join("a.md"), "a").unwrap();
        fs::write(root.join(".git").join("config"), "git").unwrap();

        let ignorer = Ignorer::new(&root.to_string_lossy());
        let items = build_file_items(&ignorer, &root, &root);

        assert_eq!(items[0].label.as_ref(), "notes");
        assert_eq!(items[1].label.as_ref(), "a.md");
        assert_eq!(items[2].label.as_ref(), "b.md");
        assert!(items.iter().all(|item| item.label.as_ref() != ".git"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn creates_fallback_document_in_workspace() {
        let root = temp_dir("fallback");
        let workspace = WorkspaceRoot::new(root.clone());

        let path = workspace.ensure_fallback_document().unwrap();

        assert!(path.exists());
        assert_eq!(path.file_name().and_then(|name| name.to_str()), Some("Untitled.md"));

        let _ = fs::remove_dir_all(root);
    }
}
