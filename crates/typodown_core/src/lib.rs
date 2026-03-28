mod app_menus;
mod themes;
mod title_bar;

use gpui::{
    Action, AnyView, App, AppContext, Bounds, Context, Entity, IntoElement, KeyBinding, ParentElement as _, Pixels,
    Render, SharedString, Size, Styled as _, Window, WindowBounds, WindowKind, WindowOptions,
    actions, div, px, size,
};
use gpui_component::{Root, TitleBar};
use serde::Deserialize;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

pub use themes::{SwitchTheme, SwitchThemeMode};
pub use title_bar::AppTitleBar;

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = core, no_json)]
pub struct SelectScrollbarShow(pub gpui_component::scroll::ScrollbarShow);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = core, no_json)]
pub struct SelectLocale(pub SharedString);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = core, no_json)]
pub struct SelectFont(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = core, no_json)]
pub struct SelectRadius(pub usize);

actions!(
    core,
    [
        About,
        Open,
        Quit,
        CloseWindow,
        ToggleSearch
    ]
);

pub fn init(cx: &mut App) {
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("gpui_component=trace".parse().unwrap()),
        )
        .try_init();

    gpui_component::init(cx);
    themes::init(cx);

    let http_client = std::sync::Arc::new(
        reqwest_client::ReqwestClient::user_agent("typodown/core").unwrap(),
    );
    cx.set_http_client(http_client);

    cx.bind_keys([
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-q", Quit, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("alt-f4", Quit, None),
    ]);

    cx.on_action(|_: &Quit, cx: &mut App| {
        cx.quit();
    });

    cx.activate(true);
}

pub fn init_app_menus(title: impl Into<SharedString>, cx: &mut App) {
    app_menus::init(title, cx);
}

pub fn create_new_window<F, E>(title: &str, create_view: F, cx: &mut App)
where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + Send + 'static,
{
    create_new_window_with_size(title, None, create_view, cx);
}

pub fn create_new_window_with_size<F, E>(
    title: &str,
    window_size: Option<Size<Pixels>>,
    create_view: F,
    cx: &mut App,
) where
    E: Into<AnyView>,
    F: FnOnce(&mut Window, &mut App) -> E + Send + 'static,
{
    let mut window_size = window_size.unwrap_or(size(px(1600.0), px(1200.0)));
    if let Some(display) = cx.primary_display() {
        let display_size = display.bounds().size;
        window_size.width = window_size.width.min(display_size.width * 0.85);
        window_size.height = window_size.height.min(display_size.height * 0.85);
    }

    let window_bounds = Bounds::centered(None, window_size, cx);
    let title = SharedString::from(title.to_string());

    cx.spawn(async move |cx| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            window_min_size: Some(gpui::Size {
                width: px(480.),
                height: px(320.),
            }),
            kind: WindowKind::Normal,
            #[cfg(target_os = "linux")]
            window_background: gpui::WindowBackgroundAppearance::Transparent,
            #[cfg(target_os = "linux")]
            window_decorations: Some(gpui::WindowDecorations::Client),
            ..Default::default()
        };

        let window = cx
            .open_window(options, |window, cx| {
                let view = create_view(window, cx);
                let root = cx.new(|cx| AppRoot::new(title.clone(), view, window, cx));
                cx.new(|cx| Root::new(root, window, cx))
            })
            .expect("failed to open window");

        window
            .update(cx, |_, window, _| {
                window.activate_window();
                window.set_window_title(&title);
            })
            .expect("failed to update window");

        Ok::<_, anyhow::Error>(())
    })
    .detach();
}

struct AppRoot {
    title_bar: Entity<AppTitleBar>,
    view: AnyView,
}

impl AppRoot {
    fn new(
        title: impl Into<SharedString>,
        view: impl Into<AnyView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            title_bar: cx.new(|cx| AppTitleBar::new(title, window, cx)),
            view: view.into(),
        }
    }
}

impl Render for AppRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .size_full()
            .child(
                gpui_component::v_flex()
                    .size_full()
                    .child(self.title_bar.clone())
                    .child(div().flex_1().overflow_hidden().child(self.view.clone())),
            )
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}
