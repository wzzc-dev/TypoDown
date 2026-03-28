use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use gpui::{Action, App, SharedString};
use gpui_component::{ActiveTheme, Theme, ThemeMode, ThemeRegistry, scroll::ScrollbarShow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct State {
    theme: SharedString,
    scrollbar_show: Option<ScrollbarShow>,
}

impl State {
    fn with_defaults() -> Self {
        Self {
            theme: "Default Light".into(),
            scrollbar_show: None,
        }
    }
}

pub fn init(cx: &mut App) {
    tracing::info!("Load themes...");
    let state = load_state().unwrap_or_else(State::with_defaults);
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
        if let Some(theme) = ThemeRegistry::global(cx)
            .themes()
            .get(&state.theme)
            .cloned()
        {
            Theme::global_mut(cx).apply_config(&theme);
        }
    }) {
        tracing::error!("Failed to watch themes directory: {}", err);
    }

    if let Some(scrollbar_show) = state.scrollbar_show {
        Theme::global_mut(cx).scrollbar_show = scrollbar_show;
    }
    cx.refresh_windows();

    cx.observe_global::<Theme>(|cx| {
        let state = State {
            theme: cx.theme().theme_name().clone(),
            scrollbar_show: Some(cx.theme().scrollbar_show),
        };

        let _ = save_state(&state);
    })
    .detach();

    cx.on_action(|switch: &SwitchTheme, cx| {
        let theme_name = switch.0.clone();
        if let Some(theme_config) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
            Theme::global_mut(cx).apply_config(&theme_config);
        }
        cx.refresh_windows();
    });
    cx.on_action(|switch: &SwitchThemeMode, cx| {
        let mode = switch.0;
        Theme::change(mode, None, cx);
        cx.refresh_windows();
    });
}

#[derive(Action, Clone, PartialEq)]
#[action(namespace = themes, no_json)]
pub(crate) struct SwitchTheme(pub(crate) SharedString);

#[derive(Action, Clone, PartialEq)]
#[action(namespace = themes, no_json)]
pub(crate) struct SwitchThemeMode(pub(crate) ThemeMode);

fn state_file_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "TypoDown").map(|dirs| dirs.config_dir().join("gallery-theme-state.json"))
}

fn load_state() -> Option<State> {
    let path = state_file_path()?;
    read_state_from_path(&path).ok()
}

fn save_state(state: &State) -> std::io::Result<()> {
    let Some(path) = state_file_path() else {
        return Ok(());
    };

    write_state_to_path(&path, state)
}

fn read_state_from_path(path: &Path) -> std::io::Result<State> {
    let json = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str::<State>(&json).unwrap_or_else(|_| State::with_defaults()))
}

fn write_state_to_path(path: &Path, state: &State) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(state)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    std::fs::write(path, json)
}
