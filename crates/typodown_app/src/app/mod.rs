pub mod bootstrap;
mod editor_app;
mod menus;

pub use editor_app::{EditorApp, init_language_registry};
pub use menus::{init as init_menus, install_keybindings};
