use std::{path::Path, path::PathBuf, str::FromStr};

use anyhow::Result;
use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, IconName, Selectable, Sizable, WindowExt,
    button::{Button, ButtonVariants as _},
    clipboard::Clipboard,
    h_flex,
    highlighter::{Diagnostic, DiagnosticSeverity, Language, LanguageConfig, LanguageRegistry},
    input::{self, Input, InputEvent, InputState, Position, RopeExt, TabSize},
    list::ListItem,
    resizable::{h_resizable, resizable_panel},
    text::TextView,
    tree::{TreeState, tree},
    v_flex,
};
use lsp_types::{CodeAction, CodeActionKind, TextEdit, WorkspaceEdit};

use crate::{
    document::{DEFAULT_DOCUMENT_TEXT, EditorPreferences, OpenDocument},
    services::EditorAssistService,
    ui::ExperimentalWysiwygEditor,
    workspace::WorkspaceRoot,
};

use super::menus::{
    NewFile, OpenFile, RefreshWorkspace, ReloadFile, SaveFile, SaveFileAs, SelectWorkspace,
    ToggleAutocorrectDiagnostics,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Source,
    Split,
    Experimental,
}

pub fn init_language_registry() {
    LanguageRegistry::singleton().register(
        "navi",
        &LanguageConfig::new(
            "navi",
            tree_sitter_navi::LANGUAGE.into(),
            vec![],
            tree_sitter_navi::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
    );
}

pub struct EditorApp {
    editor: Entity<InputState>,
    experimental_editor: Entity<ExperimentalWysiwygEditor>,
    tree_state: Entity<TreeState>,
    go_to_line_state: Entity<InputState>,
    language: Language,
    view_mode: ViewMode,
    document: OpenDocument,
    workspace: WorkspaceRoot,
    preferences: EditorPreferences,
    assist_service: EditorAssistService,
    subscriptions: Vec<Subscription>,
    lint_task: Task<()>,
}

impl EditorApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let workspace = WorkspaceRoot::default();
        let fallback_path = workspace.ensure_fallback_document().ok();
        let (document, initial_text, language) = initial_document_state(fallback_path.as_deref());

        let preferences = EditorPreferences::default();
        let assist_service = EditorAssistService::new();
        let editor = cx.new(|cx| {
            let mut state = InputState::new(window, cx)
                .code_editor(language.name())
                .line_number(preferences.line_number)
                .indent_guides(preferences.indent_guides)
                .tab_size(TabSize {
                    tab_size: 4,
                    hard_tabs: false,
                })
                .soft_wrap(preferences.soft_wrap)
                .default_value(initial_text.clone())
                .placeholder("Start writing in Markdown...");

            let completion_provider: std::rc::Rc<dyn gpui_component::input::CompletionProvider> =
                std::rc::Rc::new(assist_service.clone());
            let hover_provider: std::rc::Rc<dyn gpui_component::input::HoverProvider> =
                std::rc::Rc::new(assist_service.clone());
            let definition_provider: std::rc::Rc<dyn gpui_component::input::DefinitionProvider> =
                std::rc::Rc::new(assist_service.clone());
            let document_color_provider: std::rc::Rc<
                dyn gpui_component::input::DocumentColorProvider,
            > = std::rc::Rc::new(assist_service.clone());

            state.lsp.completion_provider = Some(completion_provider);
            state.lsp.code_action_providers = assist_service.code_action_providers();
            state.lsp.hover_provider = Some(hover_provider);
            state.lsp.definition_provider = Some(definition_provider);
            state.lsp.document_color_provider = Some(document_color_provider);
            state
        });

        let go_to_line_state = cx.new(|cx| InputState::new(window, cx));
        let tree_state = cx.new(|cx| TreeState::new(cx));
        let experimental_editor =
            cx.new(|cx| ExperimentalWysiwygEditor::new(editor.clone(), window, cx));

        let mut this = Self {
            editor,
            experimental_editor,
            tree_state,
            go_to_line_state,
            language,
            view_mode: ViewMode::Split,
            document,
            workspace,
            preferences,
            assist_service,
            subscriptions: Vec::new(),
            lint_task: Task::ready(()),
        };

        this.refresh_workspace_tree(cx);
        if this.preferences.autocorrect_enabled {
            this.lint_document(cx);
        }

        this.subscriptions.push(cx.subscribe(&this.editor, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        }));

        this
    }

    fn current_text(&self, cx: &App) -> String {
        self.editor.read(cx).value().to_string()
    }

    fn is_dirty(&self, cx: &App) -> bool {
        self.document.is_dirty(&self.current_text(cx))
    }

    fn refresh_workspace_tree(&mut self, cx: &mut Context<Self>) {
        let items = self.workspace.tree_items();
        self.tree_state.update(cx, |state, cx| {
            state.set_items(items, cx);
        });
    }

    fn apply_document_text(
        &mut self,
        content: String,
        language: Language,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.language = language;
        self.editor.update(cx, |editor, cx| {
            editor.set_highlighter(language.name(), cx);
            editor.set_value(content, window, cx);
        });

        if self.preferences.autocorrect_enabled {
            self.lint_document(cx);
        } else {
            self.assist_service.clear_diagnostics();
        }

        cx.notify();
    }

    fn reset_to_new_document(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.document = OpenDocument::unsaved(DEFAULT_DOCUMENT_TEXT);
        self.apply_document_text(
            DEFAULT_DOCUMENT_TEXT.to_string(),
            Language::from_str("markdown"),
            window,
            cx,
        );
    }

    fn open_document_path(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let content = std::fs::read_to_string(&path)?;
        let language = language_for_path(&path);
        self.document = OpenDocument::from_disk(path.clone(), content.clone());
        self.workspace = WorkspaceRoot::from_document(&path);
        self.refresh_workspace_tree(cx);
        self.apply_document_text(content, language, window, cx);
        Ok(())
    }

    fn save_to_path(
        &mut self,
        path: PathBuf,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let text = self.current_text(cx);
        std::fs::write(&path, &text)?;
        self.document.mark_saved(path.clone(), text);
        self.workspace = WorkspaceRoot::from_document(&path);
        self.refresh_workspace_tree(cx);
        if self.preferences.autocorrect_enabled {
            self.lint_document(cx);
        }
        cx.notify();
        Ok(())
    }

    fn reload_current_document(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = self.document.path().map(Path::to_path_buf) else {
            self.reset_to_new_document(window, cx);
            return;
        };

        if let Err(err) = self.open_document_path(path, window, cx) {
            window.push_notification(format!("Failed to reload document: {err}"), cx);
        }
    }

    fn lint_document(&mut self, cx: &mut Context<Self>) {
        if !self.preferences.autocorrect_enabled {
            self.assist_service.clear_diagnostics();
            return;
        }

        let language = self.language.name().to_string();
        let assist_service = self.assist_service.clone();
        let text = self.editor.read(cx).text().clone();

        self.lint_task = cx.background_spawn(async move {
            let value = text.to_string();
            let result = autocorrect::lint_for(value.as_str(), &language);

            let mut code_actions = vec![];
            let mut diagnostics = vec![];

            for item in result.lines.iter() {
                let severity = match item.severity {
                    autocorrect::Severity::Error => DiagnosticSeverity::Warning,
                    autocorrect::Severity::Warning => DiagnosticSeverity::Hint,
                    autocorrect::Severity::Pass => DiagnosticSeverity::Info,
                };

                let line = item.line.saturating_sub(1);
                let col = item.col.saturating_sub(1);
                let start = Position::new(line as u32, col as u32);
                let end = Position::new(line as u32, (col + item.old.chars().count()) as u32);
                diagnostics.push(
                    Diagnostic::new(start..end, format!("AutoCorrect: {}", item.new))
                        .with_severity(severity),
                );

                let range = text.position_to_offset(&start)..text.position_to_offset(&end);
                let edit = WorkspaceEdit {
                    changes: Some(
                        std::iter::once((
                            lsp_types::Uri::from_str("file://example").unwrap(),
                            vec![TextEdit {
                                range: lsp_types::Range { start, end },
                                new_text: item.new.clone(),
                                ..Default::default()
                            }],
                        ))
                        .collect(),
                    ),
                    ..Default::default()
                };

                code_actions.push((
                    range,
                    CodeAction {
                        title: format!("Change to '{}'", item.new),
                        kind: Some(CodeActionKind::QUICKFIX),
                        edit: Some(edit),
                        ..Default::default()
                    },
                ));
            }

            assist_service.update_code_actions(code_actions);
            assist_service.update_diagnostics(diagnostics);
        });
    }

    fn go_to_line(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let editor = self.editor.clone();
        let input_state = self.go_to_line_state.clone();

        window.open_dialog(cx, move |dialog, window, cx| {
            input_state.update(cx, |state, cx| {
                let cursor_pos = editor.read(cx).cursor_position();
                state.set_placeholder(
                    format!("{}:{}", cursor_pos.line + 1, cursor_pos.character + 1),
                    window,
                    cx,
                );
                state.focus(window, cx);
            });

            dialog
                .title("Go to line")
                .child(Input::new(&input_state))
                .confirm()
                .on_ok({
                    let editor = editor.clone();
                    let input_state = input_state.clone();
                    move |_, window, cx| {
                        let query = input_state.read(cx).value();
                        let mut parts = query
                            .split(':')
                            .map(|part| part.trim().parse::<usize>().ok())
                            .collect::<Vec<_>>()
                            .into_iter();
                        let Some(line) = parts.next().and_then(|value| value) else {
                            return false;
                        };
                        let column = parts.next().and_then(|value| value).unwrap_or(1);
                        editor.update(cx, |state, cx| {
                            state.set_cursor_position(
                                input::Position::new(
                                    line.saturating_sub(1) as u32,
                                    column.saturating_sub(1) as u32,
                                ),
                                window,
                                cx,
                            );
                        });
                        true
                    }
                })
        });
    }

    fn on_action_new_file(&mut self, _: &NewFile, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_to_new_document(window, cx);
    }

    fn on_action_open_file(&mut self, _: &OpenFile, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open a Markdown or source file".into()),
        });

        let view = cx.entity();
        cx.spawn_in(window, async move |_, window| {
            let path = prompt.await.ok()?.ok()??.into_iter().next()?;
            view.update_in(window, |this, window, cx| {
                if let Err(err) = this.open_document_path(path, window, cx) {
                    window.push_notification(format!("Failed to open document: {err}"), cx);
                }
            })
            .ok()
        })
        .detach();
    }

    fn on_action_select_workspace(
        &mut self,
        _: &SelectWorkspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Choose a workspace folder".into()),
        });

        let view = cx.entity();
        cx.spawn_in(window, async move |_, window| {
            let path = prompt.await.ok()?.ok()??.into_iter().next()?;
            view.update_in(window, |this, window, cx| {
                this.workspace.set_path(path.clone());
                this.refresh_workspace_tree(cx);
                if this.document.path().is_none()
                    && let Ok(fallback) = this.workspace.ensure_fallback_document()
                {
                    let _ = this.open_document_path(fallback, window, cx);
                }
                cx.notify();
            })
            .ok()
        })
        .detach();
    }

    fn on_action_refresh_workspace(
        &mut self,
        _: &RefreshWorkspace,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.refresh_workspace_tree(cx);
        cx.notify();
    }

    fn on_action_save_file(&mut self, _: &SaveFile, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = self.document.path().map(Path::to_path_buf) {
            if let Err(err) = self.save_to_path(path, window, cx) {
                window.push_notification(format!("Failed to save document: {err}"), cx);
            }
        } else {
            self.on_action_save_as_file(&SaveFileAs, window, cx);
        }
    }

    fn on_action_save_as_file(
        &mut self,
        _: &SaveFileAs,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let directory = self
            .document
            .path()
            .and_then(Path::parent)
            .unwrap_or(self.workspace.path());
        let suggested_name = self.document.suggested_name();
        let prompt = cx.prompt_for_new_path(directory, Some(&suggested_name));
        let view = cx.entity();

        cx.spawn_in(window, async move |_, window| {
            let path = prompt.await.ok()?.ok()??;
            view.update_in(window, |this, window, cx| {
                if let Err(err) = this.save_to_path(path, window, cx) {
                    window.push_notification(format!("Failed to save document: {err}"), cx);
                }
            })
            .ok()
        })
        .detach();
    }

    fn on_action_reload_file(
        &mut self,
        _: &ReloadFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.reload_current_document(window, cx);
    }

    fn on_action_toggle_autocorrect(
        &mut self,
        _: &ToggleAutocorrectDiagnostics,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.preferences.autocorrect_enabled = !self.preferences.autocorrect_enabled;
        if self.preferences.autocorrect_enabled {
            self.lint_document(cx);
        } else {
            self.assist_service.clear_diagnostics();
        }
        cx.notify();
    }

    fn render_file_tree(&self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        tree(
            &self.tree_state,
            move |ix, entry, _selected, _window, cx| {
                view.update(cx, |_, cx| {
                    let item = entry.item();
                    let icon = if !entry.is_folder() {
                        IconName::File
                    } else if entry.is_expanded() {
                        IconName::FolderOpen
                    } else {
                        IconName::Folder
                    };

                    ListItem::new(ix)
                        .w_full()
                        .rounded(cx.theme().radius)
                        .py_0p5()
                        .px_2()
                        .pl(px(16.) * entry.depth() + px(8.))
                        .child(h_flex().gap_2().child(icon).child(item.label.clone()))
                        .on_click(cx.listener({
                            let item = item.clone();
                            move |this, _, window, cx| {
                                if item.is_folder() {
                                    return;
                                }

                                if let Err(err) =
                                    this.open_document_path(PathBuf::from(item.id.as_ref()), window, cx)
                                {
                                    window.push_notification(
                                        format!("Failed to open document: {err}"),
                                        cx,
                                    );
                                }
                                cx.notify();
                            }
                        }))
                })
            },
        )
        .text_sm()
        .p_1()
        .bg(cx.theme().sidebar)
        .text_color(cx.theme().sidebar_foreground)
        .h_full()
    }
}

impl Render for EditorApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.assist_service.take_dirty() {
            let diagnostics = self.assist_service.diagnostics();
            self.editor.update(cx, |state, cx| {
                state.diagnostics_mut().map(|set| {
                    set.clear();
                    set.extend(diagnostics);
                });
                cx.notify();
            });
        }

        let current_text = self.editor.read(cx).value().to_string();
        let document_label = format!(
            "{}{}",
            self.document.title(),
            if self.is_dirty(cx) { " • modified" } else { "" }
        );

        v_flex()
            .id("typodown")
            .size_full()
            .on_action(cx.listener(Self::on_action_new_file))
            .on_action(cx.listener(Self::on_action_open_file))
            .on_action(cx.listener(Self::on_action_save_file))
            .on_action(cx.listener(Self::on_action_save_as_file))
            .on_action(cx.listener(Self::on_action_reload_file))
            .on_action(cx.listener(Self::on_action_select_workspace))
            .on_action(cx.listener(Self::on_action_refresh_workspace))
            .on_action(cx.listener(Self::on_action_toggle_autocorrect))
            .child(
                v_flex()
                    .flex_1()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .py_2()
                            .px_4()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                v_flex()
                                    .gap_0p5()
                                    .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child(document_label))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(self.workspace.path().display().to_string()),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(Button::new("new").ghost().xsmall().label("New").on_click(
                                        cx.listener(|this, _, window, cx| this.on_action_new_file(&NewFile, window, cx)),
                                    ))
                                    .child(Button::new("open").ghost().xsmall().label("Open").on_click(
                                        cx.listener(|this, _, window, cx| this.on_action_open_file(&OpenFile, window, cx)),
                                    ))
                                    .child(Button::new("save").ghost().xsmall().label("Save").on_click(
                                        cx.listener(|this, _, window, cx| this.on_action_save_file(&SaveFile, window, cx)),
                                    ))
                                    .child(Button::new("save-as").ghost().xsmall().label("Save As").on_click(
                                        cx.listener(|this, _, window, cx| this.on_action_save_as_file(&SaveFileAs, window, cx)),
                                    ))
                                    .child(Button::new("reload").ghost().xsmall().label("Reload").on_click(
                                        cx.listener(|this, _, window, cx| this.on_action_reload_file(&ReloadFile, window, cx)),
                                    ))
                                    .child(Button::new("workspace").ghost().xsmall().label("Workspace").on_click(
                                        cx.listener(|this, _, window, cx| this.on_action_select_workspace(&SelectWorkspace, window, cx)),
                                    ))
                                    .child(Button::new("refresh-workspace").ghost().xsmall().label("Refresh").on_click(
                                        cx.listener(|this, _, window, cx| this.on_action_refresh_workspace(&RefreshWorkspace, window, cx)),
                                    )),
                            ),
                    )
                    .child(
                        h_resizable("editor-container")
                            .child(
                                resizable_panel()
                                    .size(px(260.))
                                    .child(self.render_file_tree(window, cx)),
                            )
                            .child(match self.view_mode {
                                ViewMode::Source => div()
                                    .size_full()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_size(cx.theme().mono_font_size)
                                    .child(
                                        Input::new(&self.editor)
                                            .h_full()
                                            .p_0()
                                            .border_0()
                                            .focus_bordered(false),
                                    )
                                    .into_any_element(),
                                ViewMode::Split => h_resizable("split-container")
                                    .child(
                                        resizable_panel().child(
                                            div()
                                                .size_full()
                                                .font_family(cx.theme().mono_font_family.clone())
                                                .text_size(cx.theme().mono_font_size)
                                                .child(
                                                    Input::new(&self.editor)
                                                        .h_full()
                                                        .p_0()
                                                        .border_0()
                                                        .focus_bordered(false),
                                                ),
                                        ),
                                    )
                                    .child(
                                        resizable_panel().child(
                                            TextView::markdown("preview", current_text.clone(), window, cx)
                                                .code_block_actions(|code_block, _window, _cx| {
                                                    h_flex()
                                                        .gap_1()
                                                        .child(
                                                            Clipboard::new("copy")
                                                                .value(code_block.code().clone()),
                                                        )
                                                })
                                                .flex_none()
                                                .p_5()
                                                .scrollable(true)
                                                .selectable(true),
                                        ),
                                    )
                                    .into_any_element(),
                                ViewMode::Experimental => div()
                                    .size_full()
                                    .child(self.experimental_editor.clone())
                                    .into_any_element(),
                            }),
                    ),
            )
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .text_sm()
                    .bg(cx.theme().background)
                    .py_1p5()
                    .px_4()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        h_flex()
                            .gap_3()
                            .child(
                                Button::new("line-number")
                                    .ghost()
                                    .xsmall()
                                    .when(self.preferences.line_number, |button| button.icon(IconName::Check))
                                    .label("Line Number")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.preferences.line_number = !this.preferences.line_number;
                                        this.editor.update(cx, |state, cx| {
                                            state.set_line_number(this.preferences.line_number, window, cx);
                                        });
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("soft-wrap")
                                    .ghost()
                                    .xsmall()
                                    .when(self.preferences.soft_wrap, |button| button.icon(IconName::Check))
                                    .label("Soft Wrap")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.preferences.soft_wrap = !this.preferences.soft_wrap;
                                        this.editor.update(cx, |state, cx| {
                                            state.set_soft_wrap(this.preferences.soft_wrap, window, cx);
                                        });
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("indent-guides")
                                    .ghost()
                                    .xsmall()
                                    .when(self.preferences.indent_guides, |button| button.icon(IconName::Check))
                                    .label("Indent Guides")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.preferences.indent_guides = !this.preferences.indent_guides;
                                        this.editor.update(cx, |state, cx| {
                                            state.set_indent_guides(this.preferences.indent_guides, window, cx);
                                        });
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("toggle-autocorrect")
                                    .ghost()
                                    .xsmall()
                                    .when(self.preferences.autocorrect_enabled, |button| button.icon(IconName::Check))
                                    .label(if self.preferences.autocorrect_enabled {
                                        "AutoCorrect On"
                                    } else {
                                        "AutoCorrect Off"
                                    })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.on_action_toggle_autocorrect(&ToggleAutocorrectDiagnostics, window, cx);
                                    })),
                            ),
                    )
                    .child(
                        Button::new("line-column")
                            .ghost()
                            .xsmall()
                            .label({
                                let position = self.editor.read(cx).cursor_position();
                                let cursor = self.editor.read(cx).cursor();
                                format!("{}:{} ({} byte)", position.line + 1, position.character + 1, cursor)
                            })
                            .on_click(cx.listener(Self::go_to_line)),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new("mode-source")
                                    .ghost()
                                    .xsmall()
                                    .label("Source")
                                    .selected(self.view_mode == ViewMode::Source)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.view_mode = ViewMode::Source;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("mode-split")
                                    .ghost()
                                    .xsmall()
                                    .label("Split")
                                    .selected(self.view_mode == ViewMode::Split)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.view_mode = ViewMode::Split;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("mode-experimental")
                                    .ghost()
                                    .xsmall()
                                    .label("Experimental")
                                    .selected(self.view_mode == ViewMode::Experimental)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.view_mode = ViewMode::Experimental;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
    }
}

fn initial_document_state(path: Option<&Path>) -> (OpenDocument, String, Language) {
    let Some(path) = path else {
        return (
            OpenDocument::unsaved(DEFAULT_DOCUMENT_TEXT),
            DEFAULT_DOCUMENT_TEXT.to_string(),
            Language::from_str("markdown"),
        );
    };

    let content =
        std::fs::read_to_string(path).unwrap_or_else(|_| DEFAULT_DOCUMENT_TEXT.to_string());
    let language = language_for_path(path);
    (
        OpenDocument::from_disk(path.to_path_buf(), content.clone()),
        content,
        language,
    )
}

fn language_for_path(path: &Path) -> Language {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("md");
    Language::from_str(extension)
}
