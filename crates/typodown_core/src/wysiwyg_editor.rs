use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme,
    input::{InputState, Position},
    text::TextView,
    v_flex,
};

#[derive(IntoElement)]
pub struct WysiwygEditor {
    input_state: Entity<InputState>,
}

impl WysiwygEditor {
    pub fn new(input_state: Entity<InputState>) -> Self {
        Self { input_state }
    }
}

impl RenderOnce for WysiwygEditor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let input = self.input_state.read(cx);
        let cursor_line = input.cursor_position().line;
        let value = input.value().clone();
        let input_state = self.input_state.clone();

        v_flex()
            .id("wysiwyg-editor")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .children(value.lines().enumerate().map(move |(i, line_text)| {
                let is_active = i as u32 == cursor_line;
                let input_state = input_state.clone();

                div()
                    .id(("line", i))
                    .w_full()
                    .child(if is_active {
                        // Render raw text for the active line
                        div()
                            .font_family(cx.theme().mono_font_family.clone())
                            .text_size(cx.theme().mono_font_size)
                            .bg(cx.theme().accent.opacity(0.1))
                            .child(line_text.to_string())
                            .into_any_element()
                    } else {
                        // Render styled Markdown for inactive lines
                        TextView::markdown(
                            SharedString::from(format!("line-preview-{}", i)),
                            line_text.to_string(),
                            window,
                            cx,
                        )
                        .into_any_element()
                    })
                    .on_click(move |_, window, cx| {
                        input_state.update(cx, |state: &mut InputState, cx| {
                            state.set_cursor_position(Position::new(i as u32, 0), window, cx);
                        });
                    })
            }))
    }
}
