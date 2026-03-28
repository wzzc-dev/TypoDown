use gpui::{Application, AppContext, px, size};
use gpui_component_assets::Assets;

use crate::app::{EditorApp, init_language_registry, init_menus, install_keybindings};

pub fn run() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        typodown_core::init(cx);
        init_language_registry();
        init_menus(cx);
        install_keybindings(cx);

        typodown_core::create_new_window_with_size(
            "TypoDown",
            Some(size(px(1200.), px(750.))),
            |window, cx| cx.new(|cx| EditorApp::new(window, cx)),
            cx,
        );
    });
}
