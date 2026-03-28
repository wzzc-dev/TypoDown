use std::{
    ops::Range,
    rc::Rc,
    sync::{Arc, RwLock},
};

use gpui::{App, Context, Entity, Result, SharedString, Task, Window};
use gpui_component::{
    highlighter::Diagnostic,
    input::{
        CodeActionProvider, CompletionProvider, DefinitionProvider, DocumentColorProvider,
        HoverProvider, InputState, Rope,
    },
};
use lsp_types::{
    CodeAction, CompletionContext, CompletionResponse, InlineCompletionContext,
    InlineCompletionResponse, WorkspaceEdit,
};

#[derive(Debug, Clone, Default)]
pub struct DiagnosticsState {
    diagnostics: Vec<Diagnostic>,
    code_actions: Vec<(Range<usize>, CodeAction)>,
    dirty: bool,
}

#[derive(Clone, Default)]
pub struct EditorAssistService {
    state: Arc<RwLock<DiagnosticsState>>,
    #[cfg(feature = "dev-demo")]
    completions: Arc<Vec<lsp_types::CompletionItem>>,
}

impl EditorAssistService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(DiagnosticsState::default())),
            #[cfg(feature = "dev-demo")]
            completions: Arc::new(crate::services::mock::load_completion_items()),
        }
    }

    pub fn code_action_providers(&self) -> Vec<Rc<dyn CodeActionProvider>> {
        let service: Rc<dyn CodeActionProvider> = Rc::new(self.clone());
        let providers = vec![service];
        #[cfg(feature = "dev-demo")]
        let mut providers = {
            let mut providers = providers;
            providers.push(Rc::new(crate::services::MockTextConvertor));
            providers
        };
        providers
    }

    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.state.read().unwrap().diagnostics.clone()
    }

    pub fn update_diagnostics(&self, diagnostics: Vec<Diagnostic>) {
        let mut state = self.state.write().unwrap();
        state.diagnostics = diagnostics;
        state.dirty = true;
    }

    pub fn code_actions_for_range(&self, range: Range<usize>) -> Vec<CodeAction> {
        self.state
            .read()
            .unwrap()
            .code_actions
            .iter()
            .filter(|(node_range, _)| range.start >= node_range.start && range.end <= node_range.end)
            .map(|(_, code_action)| code_action.clone())
            .collect()
    }

    pub fn update_code_actions(&self, code_actions: Vec<(Range<usize>, CodeAction)>) {
        let mut state = self.state.write().unwrap();
        state.code_actions = code_actions;
        state.dirty = true;
    }

    pub fn clear_diagnostics(&self) {
        let mut state = self.state.write().unwrap();
        state.diagnostics.clear();
        state.code_actions.clear();
        state.dirty = true;
    }

    pub fn take_dirty(&self) -> bool {
        let mut state = self.state.write().unwrap();
        let dirty = state.dirty;
        state.dirty = false;
        dirty
    }
}

impl CompletionProvider for EditorAssistService {
    fn completions(
        &self,
        rope: &Rope,
        offset: usize,
        trigger: CompletionContext,
        _window: &mut Window,
        cx: &mut Context<InputState>,
    ) -> Task<Result<CompletionResponse>> {
        #[cfg(feature = "dev-demo")]
        {
            return crate::services::mock::completions(
                self.completions.clone(),
                rope,
                offset,
                trigger,
                cx,
            );
        }

        #[cfg(not(feature = "dev-demo"))]
        {
            let _ = rope;
            let _ = offset;
            let _ = trigger;
            let _ = cx;
            Task::ready(Ok(CompletionResponse::Array(vec![])))
        }
    }

    fn inline_completion(
        &self,
        rope: &Rope,
        offset: usize,
        trigger: InlineCompletionContext,
        _window: &mut Window,
        cx: &mut Context<InputState>,
    ) -> Task<Result<InlineCompletionResponse>> {
        #[cfg(feature = "dev-demo")]
        {
            return crate::services::mock::inline_completion(rope, offset, trigger, cx);
        }

        #[cfg(not(feature = "dev-demo"))]
        {
            let _ = rope;
            let _ = offset;
            let _ = trigger;
            let _ = cx;
            Task::ready(Ok(InlineCompletionResponse::Array(vec![])))
        }
    }

    fn is_completion_trigger(
        &self,
        _offset: usize,
        _new_text: &str,
        _cx: &mut Context<InputState>,
    ) -> bool {
        #[cfg(feature = "dev-demo")]
        {
            true
        }

        #[cfg(not(feature = "dev-demo"))]
        {
            false
        }
    }
}

impl CodeActionProvider for EditorAssistService {
    fn id(&self) -> SharedString {
        "EditorAssistService".into()
    }

    fn code_actions(
        &self,
        _state: Entity<InputState>,
        range: Range<usize>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<Result<Vec<CodeAction>>> {
        Task::ready(Ok(self.code_actions_for_range(range)))
    }

    fn perform_code_action(
        &self,
        state: Entity<InputState>,
        action: CodeAction,
        _push_to_history: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<Result<()>> {
        let Some(WorkspaceEdit {
            changes: Some(changes),
            ..
        }) = action.edit
        else {
            return Task::ready(Ok(()));
        };

        let Some((_, text_edits)) = changes.into_iter().next() else {
            return Task::ready(Ok(()));
        };

        let state = state.downgrade();
        window.spawn(cx, async move |cx| {
            state.update_in(cx, |state, window, cx| {
                state.apply_lsp_edits(&text_edits, window, cx);
            })
        })
    }
}

impl HoverProvider for EditorAssistService {
    fn hover(
        &self,
        text: &Rope,
        offset: usize,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<Result<Option<lsp_types::Hover>>> {
        #[cfg(feature = "dev-demo")]
        {
            return crate::services::mock::hover(self.completions.clone(), text, offset);
        }

        #[cfg(not(feature = "dev-demo"))]
        {
            let _ = text;
            let _ = offset;
            Task::ready(Ok(None))
        }
    }
}

impl DefinitionProvider for EditorAssistService {
    fn definitions(
        &self,
        text: &Rope,
        offset: usize,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<Result<Vec<lsp_types::LocationLink>>> {
        #[cfg(feature = "dev-demo")]
        {
            return crate::services::mock::definitions(text, offset);
        }

        #[cfg(not(feature = "dev-demo"))]
        {
            let _ = text;
            let _ = offset;
            Task::ready(Ok(vec![]))
        }
    }
}

impl DocumentColorProvider for EditorAssistService {
    fn document_colors(
        &self,
        text: &Rope,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<Result<Vec<lsp_types::ColorInformation>>> {
        let nodes = color_lsp::parse(&text.to_string());
        let colors = nodes
            .into_iter()
            .map(|node| {
                let start = lsp_types::Position::new(node.position.line, node.position.character);
                let end = lsp_types::Position::new(
                    node.position.line,
                    node.position.character + node.matched.chars().count() as u32,
                );

                lsp_types::ColorInformation {
                    range: lsp_types::Range { start, end },
                    color: lsp_types::Color {
                        red: node.color.r,
                        green: node.color.g,
                        blue: node.color.b,
                        alpha: node.color.a,
                    },
                }
            })
            .collect::<Vec<_>>();

        Task::ready(Ok(colors))
    }
}
