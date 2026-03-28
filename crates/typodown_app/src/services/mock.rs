use std::{ops::Range, rc::Rc, str::FromStr, sync::Arc, time::Duration};

use gpui::{App, Context, Entity, Result, SharedString, Task, Window};
use gpui_component::input::{
    CodeActionProvider, CompletionContext, InputState, Position, Rope, RopeExt,
};
use lsp_types::{
    CodeAction, CodeActionKind, CompletionItem, CompletionResponse, CompletionTextEdit, Hover,
    HoverContents, InlineCompletionContext, InlineCompletionItem, InlineCompletionResponse,
    InsertReplaceEdit, InsertTextFormat, LocationLink, MarkedString, TextEdit, WorkspaceEdit,
};

const RUST_DOC_URLS: &[(&str, &str)] = &[
    ("String", "string/struct.String"),
    ("Debug", "fmt/trait.Debug"),
    ("Clone", "clone/trait.Clone"),
    ("Option", "option/enum.Option"),
    ("Result", "result/enum.Result"),
    ("Vec", "vec/struct.Vec"),
    ("HashMap", "collections/hash_map/struct.HashMap"),
    ("HashSet", "collections/hash_set/struct.HashSet"),
    ("Arc", "sync/struct.Arc"),
    ("RwLock", "sync/struct.RwLock"),
    ("Duration", "time/struct.Duration"),
];

pub struct MockTextConvertor;

pub fn load_completion_items() -> Vec<CompletionItem> {
    serde_json::from_slice::<Vec<CompletionItem>>(include_bytes!("../fixtures/completion_items.json"))
        .unwrap()
}

pub fn completions(
    items: Arc<Vec<CompletionItem>>,
    rope: &Rope,
    offset: usize,
    trigger: CompletionContext,
    cx: &mut Context<InputState>,
) -> Task<Result<CompletionResponse>> {
    let trigger_character = trigger.trigger_character.unwrap_or_default();
    if trigger_character.is_empty() {
        return Task::ready(Ok(CompletionResponse::Array(vec![])));
    }

    let rope = rope.clone();
    cx.background_spawn(async move {
        smol::Timer::after(Duration::from_millis(20)).await;

        if trigger_character.starts_with('/') {
            let start = offset.saturating_sub(trigger_character.len());
            let start_pos = rope.offset_to_position(start);
            let end_pos = rope.offset_to_position(offset);
            let replace_range = lsp_types::Range::new(start_pos, end_pos);

            let items = vec![
                completion_item(
                    &replace_range,
                    "/date",
                    format!("{}", chrono::Local::now().date_naive()).as_str(),
                    "Insert current date",
                ),
                completion_item(&replace_range, "/thanks", "Thank you!", "Insert Thank you!"),
                completion_item(&replace_range, "/+1", "👍", "Insert thumbs up"),
                completion_item(&replace_range, "/-1", "👎", "Insert thumbs down"),
                completion_item(&replace_range, "/smile", "😉", "Insert wink"),
                completion_item(&replace_range, "/sad", "😩", "Insert tired face"),
                completion_item(&replace_range, "/launch", "🚀", "Insert rocket"),
            ];
            return Ok(CompletionResponse::Array(items));
        }

        let items = items
            .iter()
            .filter(|item| item.label.starts_with(&trigger_character))
            .take(10)
            .map(|item| {
                let mut item = item.clone();
                item.insert_text = Some(item.label.replace(&trigger_character, ""));
                item
            })
            .collect::<Vec<_>>();

        Ok(CompletionResponse::Array(items))
    })
}

pub fn inline_completion(
    rope: &Rope,
    offset: usize,
    _trigger: InlineCompletionContext,
    cx: &mut Context<InputState>,
) -> Task<Result<InlineCompletionResponse>> {
    let rope = rope.clone();
    cx.background_spawn(async move {
        let point = rope.offset_to_point(offset);
        let line_start = rope.line_start_offset(point.row);
        let current_line = rope.slice(line_start..offset).to_string();

        let suggestion = if current_line.trim_start().starts_with("fn ")
            && !current_line.contains('{')
        {
            Some("() {\n    // Write your code here..\n}".into())
        } else {
            None
        };

        if let Some(insert_text) = suggestion {
            Ok(InlineCompletionResponse::Array(vec![InlineCompletionItem {
                insert_text,
                filter_text: None,
                range: None,
                command: None,
                insert_text_format: Some(InsertTextFormat::SNIPPET),
            }]))
        } else {
            Ok(InlineCompletionResponse::Array(vec![]))
        }
    })
}

pub fn hover(
    items: Arc<Vec<CompletionItem>>,
    text: &Rope,
    offset: usize,
) -> Task<Result<Option<Hover>>> {
    let word = text.word_at(offset);
    if word.is_empty() {
        return Task::ready(Ok(None));
    }

    let Some(item) = items.iter().find(|item| item.label == word) else {
        return Task::ready(Ok(None));
    };

    let contents = if let Some(doc) = &item.documentation {
        match doc {
            lsp_types::Documentation::String(value) => value.clone(),
            lsp_types::Documentation::MarkupContent(value) => value.value.clone(),
        }
    } else {
        "No documentation available.".to_string()
    };

    Task::ready(Ok(Some(Hover {
        contents: HoverContents::Scalar(MarkedString::String(contents)),
        range: None,
    })))
}

pub fn definitions(text: &Rope, offset: usize) -> Task<Result<Vec<LocationLink>>> {
    let Some(word_range) = text.word_range(offset) else {
        return Task::ready(Ok(vec![]));
    };
    let word = text.slice(word_range.clone()).to_string();

    let document_uri = lsp_types::Uri::from_str("file://example").unwrap();
    let start = text.offset_to_position(word_range.start);
    let end = text.offset_to_position(word_range.end);
    let symbol_range = lsp_types::Range { start, end };

    if word == "Duration" {
        let target_range = lsp_types::Range {
            start: lsp_types::Position {
                line: 2,
                character: 4,
            },
            end: lsp_types::Position {
                line: 2,
                character: 23,
            },
        };
        return Task::ready(Ok(vec![LocationLink {
            target_uri: document_uri,
            target_range,
            target_selection_range: target_range,
            origin_selection_range: Some(symbol_range),
        }]));
    }

    for (name, url) in RUST_DOC_URLS {
        if *name == word {
            return Task::ready(Ok(vec![LocationLink {
                target_uri: lsp_types::Uri::from_str(&format!(
                    "https://doc.rust-lang.org/std/{}.html",
                    url
                ))
                .unwrap(),
                target_selection_range: lsp_types::Range::default(),
                target_range: lsp_types::Range::default(),
                origin_selection_range: Some(symbol_range),
            }]));
        }
    }

    Task::ready(Ok(vec![]))
}

impl CodeActionProvider for MockTextConvertor {
    fn id(&self) -> SharedString {
        "MockTextConvertor".into()
    }

    fn code_actions(
        &self,
        state: Entity<InputState>,
        range: Range<usize>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Task<Result<Vec<CodeAction>>> {
        if range.is_empty() {
            return Task::ready(Ok(vec![]));
        }

        let state = state.read(cx);
        let document_uri = lsp_types::Uri::from_str("file://example").unwrap();
        let old_text = state.text().slice(range.clone()).to_string();
        let start = state.text().offset_to_position(range.start);
        let end = state.text().offset_to_position(range.end);
        let range = lsp_types::Range { start, end };

        Task::ready(Ok(vec![
            replacement_action(
                "Convert to Uppercase",
                document_uri.clone(),
                range,
                old_text.to_uppercase(),
            ),
            replacement_action(
                "Convert to Lowercase",
                document_uri.clone(),
                range,
                old_text.to_lowercase(),
            ),
            replacement_action(
                "Titleize",
                document_uri.clone(),
                range,
                old_text
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        chars
                            .next()
                            .map(|c| c.to_uppercase().collect::<String>())
                            .unwrap_or_default()
                            + chars.as_str()
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            replacement_action(
                "Capitalize",
                document_uri.clone(),
                range,
                old_text
                    .chars()
                    .enumerate()
                    .map(|(index, ch)| {
                        if index == 0 {
                            ch.to_uppercase().to_string()
                        } else {
                            ch.to_string()
                        }
                    })
                    .collect(),
            ),
            replacement_action(
                "Convert to snake_case",
                document_uri,
                range,
                old_text
                    .chars()
                    .enumerate()
                    .map(|(index, ch)| {
                        if ch.is_uppercase() {
                            if index != 0 {
                                format!("_{}", ch.to_lowercase())
                            } else {
                                ch.to_lowercase().to_string()
                            }
                        } else {
                            ch.to_string()
                        }
                    })
                    .collect(),
            ),
        ]))
    }

    fn perform_code_action(
        &self,
        state: Entity<InputState>,
        action: CodeAction,
        _push_to_history: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<Result<()>> {
        let Some(edit) = action.edit else {
            return Task::ready(Ok(()));
        };

        let Some(changes) = edit.changes else {
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

fn completion_item(
    replace_range: &lsp_types::Range,
    label: &str,
    replace_text: &str,
    documentation: &str,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(lsp_types::CompletionItemKind::FUNCTION),
        text_edit: Some(CompletionTextEdit::InsertAndReplace(InsertReplaceEdit {
            new_text: replace_text.to_string(),
            insert: replace_range.clone(),
            replace: replace_range.clone(),
        })),
        documentation: Some(lsp_types::Documentation::String(documentation.to_string())),
        insert_text: None,
        ..Default::default()
    }
}

fn replacement_action(
    title: &str,
    document_uri: lsp_types::Uri,
    range: lsp_types::Range,
    new_text: String,
) -> CodeAction {
    CodeAction {
        title: title.into(),
        kind: Some(CodeActionKind::REFACTOR),
        edit: Some(WorkspaceEdit {
            changes: Some(
                std::iter::once((
                    document_uri,
                    vec![TextEdit {
                        range,
                        new_text,
                        ..Default::default()
                    }],
                ))
                .collect(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    }
}

