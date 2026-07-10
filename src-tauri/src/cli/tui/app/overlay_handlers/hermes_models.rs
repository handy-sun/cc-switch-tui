use super::super::types::{HermesModelEditorField, HermesModelEntryEditorState};
use super::*;
use crate::cli::tui::form::TextInput;
use serde_json::Value;

impl App {
    pub(super) fn handle_hermes_models_overlay_key(&mut self, key: KeyEvent) -> Option<Action> {
        if let Some(action) = self.handle_hermes_models_picker_key(key) {
            return Some(action);
        }
        self.handle_hermes_model_entry_editor_key(key)
    }

    fn handle_hermes_models_picker_key(&mut self, key: KeyEvent) -> Option<Action> {
        let Overlay::HermesModelsPicker { selected } = &mut self.overlay else {
            return None;
        };
        let Some(FormState::ProviderAdd(provider)) = self.form.as_mut() else {
            self.overlay = Overlay::None;
            return Some(Action::None);
        };
        if provider.app_type != AppType::Hermes {
            self.overlay = Overlay::None;
            return Some(Action::None);
        }

        Some(match key.code {
            KeyCode::Esc => {
                self.overlay = Overlay::None;
                Action::None
            }
            KeyCode::Up => {
                *selected = selected.saturating_sub(1);
                Action::None
            }
            KeyCode::Down => {
                if !provider.openclaw_models.is_empty() {
                    *selected = selected
                        .saturating_add(1)
                        .min(provider.openclaw_models.len() - 1);
                }
                Action::None
            }
            KeyCode::Char('a') => {
                self.overlay = Overlay::HermesModelEntryEditor(HermesModelEntryEditorState {
                    row: None,
                    return_selected: *selected,
                    field: HermesModelEditorField::Id,
                    id: TextInput::new(""),
                    context_length: TextInput::new(""),
                    max_tokens: TextInput::new(""),
                });
                Action::None
            }
            KeyCode::Enter => {
                let Some(model) = provider.openclaw_models.get(*selected) else {
                    return Some(Action::None);
                };
                let id = model.get("id").and_then(Value::as_str).unwrap_or_default();
                self.overlay = Overlay::HermesModelEntryEditor(HermesModelEntryEditorState {
                    row: Some(*selected),
                    return_selected: *selected,
                    field: HermesModelEditorField::Id,
                    id: TextInput::new(id),
                    context_length: TextInput::new(model_number_text(model, "context_length")),
                    max_tokens: TextInput::new(model_number_text(model, "max_tokens")),
                });
                Action::None
            }
            KeyCode::Backspace | KeyCode::Delete => {
                provider.remove_hermes_model(*selected);
                *selected = (*selected).min(provider.openclaw_models.len().saturating_sub(1));
                Action::None
            }
            _ => Action::None,
        })
    }

    fn handle_hermes_model_entry_editor_key(&mut self, key: KeyEvent) -> Option<Action> {
        if !matches!(self.overlay, Overlay::HermesModelEntryEditor(_)) {
            return None;
        }

        match key.code {
            KeyCode::Esc => {
                let selected = match (&self.overlay, &self.form) {
                    (
                        Overlay::HermesModelEntryEditor(editor),
                        Some(FormState::ProviderAdd(provider)),
                    ) => editor
                        .return_selected
                        .min(provider.openclaw_models.len().saturating_sub(1)),
                    _ => 0,
                };
                self.overlay = Overlay::HermesModelsPicker { selected };
                Some(Action::None)
            }
            KeyCode::Tab => {
                if let Overlay::HermesModelEntryEditor(editor) = &mut self.overlay {
                    editor.field = match editor.field {
                        HermesModelEditorField::Id => HermesModelEditorField::ContextLength,
                        HermesModelEditorField::ContextLength => HermesModelEditorField::MaxTokens,
                        HermesModelEditorField::MaxTokens => HermesModelEditorField::Id,
                    };
                }
                Some(Action::None)
            }
            KeyCode::Enter => {
                let (row, id, context_length, max_tokens) = match &self.overlay {
                    Overlay::HermesModelEntryEditor(editor) => (
                        editor.row,
                        editor.id.value.clone(),
                        editor.context_length.value.clone(),
                        editor.max_tokens.value.clone(),
                    ),
                    _ => return Some(Action::None),
                };
                let result = match self.form.as_mut() {
                    Some(FormState::ProviderAdd(provider)) => {
                        provider.upsert_hermes_model(row, id, context_length, max_tokens)
                    }
                    _ => return Some(Action::None),
                };
                match result {
                    Ok(selected) => self.overlay = Overlay::HermesModelsPicker { selected },
                    Err(err) => self.push_toast(err, ToastKind::Warning),
                }
                Some(Action::None)
            }
            _ => {
                if let Overlay::HermesModelEntryEditor(editor) = &mut self.overlay {
                    let input = match editor.field {
                        HermesModelEditorField::Id => &mut editor.id,
                        HermesModelEditorField::ContextLength => &mut editor.context_length,
                        HermesModelEditorField::MaxTokens => &mut editor.max_tokens,
                    };
                    let _ = input.apply_key(key);
                }
                Some(Action::None)
            }
        }
    }
}

fn model_number_text(model: &Value, key: &str) -> String {
    model
        .get(key)
        .and_then(Value::as_u64)
        .map(|value| value.to_string())
        .unwrap_or_default()
}
