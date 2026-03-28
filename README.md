# TypoDown

TypoDown is a Rust desktop Markdown editor built on GPUI.

## Workspace

- `crates/typodown_app`: product application with editor, split preview, experimental WYSIWYG mode, document lifecycle commands, and workspace browser
- `crates/typodown_core`: shared window shell, title bar, theme loading, and runtime setup
- `crates/typodown_gallery`: development gallery for shared UI components and stories
- `crates/reqwest_client`: isolated HTTP/TLS client used by shared runtime code

## Current Product Capabilities

- Open a workspace folder and browse files from the sidebar
- Open, create, save, save as, and reload Markdown or source files
- Switch between source, split preview, and experimental line-based WYSIWYG modes
- Toggle AutoCorrect diagnostics without running linting on every keystroke
- Persist theme preferences in the user config directory instead of `target/`

## Build And Run

```bash
cargo run -p typodown_app
```

```bash
cargo run -p typodown_gallery
```

## Development Checks

```bash
cargo check --workspace
cargo test --workspace
```

## Notes

- The experimental WYSIWYG mode is intentionally limited. It edits the active line directly and renders the remaining lines as Markdown.
- The default cold-start workspace is created under the user's `Documents/TypoDown` directory when available.
- Theme JSON files are loaded from the repository `themes/` directory.
