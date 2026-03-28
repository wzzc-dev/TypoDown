use gpui::{App, KeyBinding, Menu, MenuItem, SharedString, actions};
use gpui_component::{ThemeMode, ThemeRegistry};

actions!(
    typodown_app,
    [
        NewFile,
        OpenFile,
        SaveFile,
        SaveFileAs,
        ReloadFile,
        SelectWorkspace,
        RefreshWorkspace,
        ToggleAutocorrectDiagnostics
    ]
);

pub fn init(cx: &mut App) {
    cx.set_menus(vec![
        Menu {
            name: SharedString::from("TypoDown"),
            items: vec![
                MenuItem::action("About TypoDown", typodown_core::About),
                MenuItem::separator(),
                MenuItem::action("New", NewFile),
                MenuItem::action("Open...", OpenFile),
                MenuItem::action("Open Workspace...", SelectWorkspace),
                MenuItem::separator(),
                MenuItem::action("Save", SaveFile),
                MenuItem::action("Save As...", SaveFileAs),
                MenuItem::action("Reload From Disk", ReloadFile),
                MenuItem::separator(),
                MenuItem::action("Quit", typodown_core::Quit),
            ],
        },
        Menu {
            name: SharedString::from("Edit"),
            items: vec![
                MenuItem::action("Undo", gpui_component::input::Undo),
                MenuItem::action("Redo", gpui_component::input::Redo),
                MenuItem::separator(),
                MenuItem::action("Cut", gpui_component::input::Cut),
                MenuItem::action("Copy", gpui_component::input::Copy),
                MenuItem::action("Paste", gpui_component::input::Paste),
                MenuItem::separator(),
                MenuItem::action("Delete", gpui_component::input::Delete),
                MenuItem::action("Find", gpui_component::input::Search),
                MenuItem::separator(),
                MenuItem::action("Select All", gpui_component::input::SelectAll),
            ],
        },
        Menu {
            name: SharedString::from("View"),
            items: vec![
                MenuItem::Submenu(Menu {
                    name: "Appearance".into(),
                    items: vec![
                        MenuItem::action("Light", typodown_core::SwitchThemeMode(ThemeMode::Light)),
                        MenuItem::action("Dark", typodown_core::SwitchThemeMode(ThemeMode::Dark)),
                    ],
                }),
                theme_menu(cx),
                MenuItem::separator(),
                MenuItem::action("Refresh Workspace", RefreshWorkspace),
                MenuItem::action("Toggle AutoCorrect Diagnostics", ToggleAutocorrectDiagnostics),
            ],
        },
        Menu {
            name: SharedString::from("Window"),
            items: vec![MenuItem::action("Close Window", typodown_core::CloseWindow)],
        },
    ]);
}

pub fn install_keybindings(cx: &mut App) {
    cx.bind_keys([
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-n", NewFile, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-n", NewFile, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-o", OpenFile, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-o", OpenFile, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-shift-o", SelectWorkspace, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-shift-o", SelectWorkspace, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-s", SaveFile, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-s", SaveFile, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-shift-s", SaveFileAs, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-shift-s", SaveFileAs, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-r", ReloadFile, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-r", ReloadFile, None),
        KeyBinding::new("f5", RefreshWorkspace, None),
        KeyBinding::new("ctrl-l", ToggleAutocorrectDiagnostics, None),
    ]);
}

fn theme_menu(cx: &App) -> MenuItem {
    let themes = ThemeRegistry::global(cx).sorted_themes();
    MenuItem::Submenu(Menu {
        name: "Theme".into(),
        items: themes
            .iter()
            .map(|theme| {
                MenuItem::action(
                    theme.name.clone(),
                    typodown_core::SwitchTheme(theme.name.clone()),
                )
            })
            .collect(),
    })
}

