use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme,
    input::{Input, InputEvent, InputState, Position},
    text::TextView,
    v_flex,
};

pub struct ExperimentalWysiwygEditor {
    main_editor: Entity<InputState>,
    line_editor: Entity<InputState>,
    active_line: Option<u32>,
    _subscriptions: Vec<Subscription>,
}

impl ExperimentalWysiwygEditor {
    pub fn new(
        main_editor: Entity<InputState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let line_editor = cx.new(|cx| InputState::new(window, cx));
        let mut this = Self {
            main_editor: main_editor.clone(),
            line_editor,
            active_line: None,
            _subscriptions: vec![],
        };

        this._subscriptions.push(cx.subscribe_in(
            &this.line_editor,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event {
                    this.sync_to_main(window, cx);
                }
            },
        ));

        this
    }

    fn update_active_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let main_state = self.main_editor.read(cx);
        let cursor_line = main_state.cursor_position().line;
        let line_text = main_state
            .value()
            .lines()
            .nth(cursor_line as usize)
            .unwrap_or_default()
            .to_string();

        if self.active_line != Some(cursor_line) {
            self.active_line = Some(cursor_line);
            self.line_editor.update(cx, |state, cx| {
                state.set_value(line_text, window, cx);
                state.focus(window, cx);
            });
        }
    }

    fn sync_to_main(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let line_text = self.line_editor.read(cx).value().to_string();
        let active_line = if let Some(l) = self.active_line {
            l
        } else {
            return;
        };

        let old_text = self
            .main_editor
            .read(cx)
            .value()
            .lines()
            .nth(active_line as usize)
            .unwrap_or_default()
            .to_string();
        if old_text == line_text {
            return;
        }

        self.main_editor.update(cx, |main_state, cx| {
            let text = main_state.value();
            let lines = text.lines().collect::<Vec<_>>();
            if (active_line as usize) < lines.len() {
                let old_line = lines[active_line as usize];
                let range = lsp_types::Range {
                    start: Position::new(active_line, 0),
                    end: Position::new(active_line, old_line.chars().count() as u32),
                };

                let edits = vec![lsp_types::TextEdit {
                    range,
                    new_text: line_text,
                }];

                main_state.apply_lsp_edits(&edits, window, cx);
            }
        });
    }
}

impl Render for ExperimentalWysiwygEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cursor_line = {
            let main_editor = self.main_editor.read(cx);
            main_editor.cursor_position().line
        };

        // Ensure we are synced if cursor moved
        if self.active_line != Some(cursor_line) {
            self.update_active_line(window, cx);
        }

        let main_editor = self.main_editor.read(cx);
        let value = main_editor.value().clone();

        v_flex()
            .id("experimental-wysiwyg-editor")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .children(value.lines().enumerate().map(|(i, line_text)| {
                let is_active = i as u32 == cursor_line;

                div()
                    .id(("line", i))
                    .w_full()
                    .child(if is_active {
                        div()
                            .bg(cx.theme().accent.opacity(0.1))
                            .child(
                                Input::new(&self.line_editor)
                                    .appearance(false)
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_size(cx.theme().mono_font_size),
                            )
                            .into_any_element()
                    } else {
                        TextView::markdown(
                            SharedString::from(format!("line-preview-{}", i)),
                            line_text.to_string(),
                            window,
                            cx,
                        )
                        .into_any_element()
                    })
                    .on_click(cx.listener(move |this, _event: &ClickEvent, window, cx| {
                        this.main_editor.update(cx, |state, cx| {
                            state.set_cursor_position(Position::new(i as u32, 0), window, cx);
                        });
                        this.update_active_line(window, cx);
                        cx.notify();
                    }))
            }))
    }
}
