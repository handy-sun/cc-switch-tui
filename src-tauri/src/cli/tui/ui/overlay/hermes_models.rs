use super::super::theme;
use super::super::*;
use crate::cli::tui::app::HermesModelEditorField;
use serde_json::Value;

pub(super) fn render_hermes_models_picker_overlay(
    frame: &mut Frame<'_>,
    app: &App,
    content_area: Rect,
    theme: &theme::Theme,
    selected: usize,
) {
    let area = centered_rect_fixed(76, 18, content_area);
    frame.render_widget(Clear, area);
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(overlay_border_style(theme, false))
        .title(texts::tui_hermes_models_title());
    frame.render_widget(outer.clone(), area);
    let inner = outer.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    render_key_bar_center(
        frame,
        chunks[0],
        theme,
        &[
            ("↑↓", texts::tui_key_select()),
            ("a", texts::tui_key_add()),
            ("Enter", texts::tui_key_edit()),
            ("Del/Backspace", texts::tui_key_delete()),
            ("Esc", texts::tui_key_close()),
        ],
    );

    let Some(FormState::ProviderAdd(provider)) = app.form.as_ref() else {
        return;
    };
    if provider.openclaw_models.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::raw(texts::tui_hermes_models_empty_state()))
                .alignment(Alignment::Center),
            inset_top(chunks[1], 1),
        );
        return;
    }

    let items = provider.openclaw_models.iter().map(|model| {
        let id = model.get("id").and_then(Value::as_str).unwrap_or("?");
        let context = model
            .get("context_length")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        let output = model
            .get("max_tokens")
            .and_then(Value::as_u64)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        ListItem::new(Line::raw(format!(
            "{id}  |  context {context}  |  output {output}"
        )))
    });
    let list = List::new(items)
        .highlight_style(selection_style(theme))
        .highlight_symbol(highlight_symbol(theme));
    let mut state = ListState::default();
    state.select(Some(
        selected.min(provider.openclaw_models.len().saturating_sub(1)),
    ));
    frame.render_stateful_widget(list, inset_top(chunks[1], 1), &mut state);
}

pub(super) fn render_hermes_model_entry_editor_overlay(
    frame: &mut Frame<'_>,
    content_area: Rect,
    theme: &theme::Theme,
    overlay: &Overlay,
) {
    let Overlay::HermesModelEntryEditor(editor) = overlay else {
        return;
    };
    let area = centered_rect_fixed(68, 16, content_area);
    frame.render_widget(Clear, area);
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(overlay_border_style(theme, false))
        .title(if editor.row.is_some() {
            texts::tui_hermes_model_edit_title()
        } else {
            texts::tui_hermes_model_add_title()
        });
    frame.render_widget(outer.clone(), area);
    let inner = outer.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(inner);
    render_key_bar_center(
        frame,
        chunks[0],
        theme,
        &[
            ("Tab", texts::tui_key_select()),
            ("Enter", texts::tui_key_apply()),
            ("Esc", texts::tui_key_cancel()),
        ],
    );

    let fields = [
        (
            texts::tui_hermes_model_id_label(),
            &editor.id,
            editor.field == HermesModelEditorField::Id,
        ),
        (
            texts::tui_label_context_limit(),
            &editor.context_length,
            editor.field == HermesModelEditorField::ContextLength,
        ),
        (
            texts::tui_label_output_limit(),
            &editor.max_tokens,
            editor.field == HermesModelEditorField::MaxTokens,
        ),
    ];
    for (index, (label, input, active)) in fields.into_iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(if active {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.dim)
            })
            .title(label);
        let input_area = chunks[index + 1];
        let input_inner = block.inner(input_area);
        frame.render_widget(block, input_area);
        let (visible, cursor_x) =
            visible_text_window(&input.value, input.cursor, input_inner.width as usize);
        frame.render_widget(
            Paragraph::new(Line::raw(visible)).wrap(Wrap { trim: false }),
            input_inner,
        );
        if active {
            frame.set_cursor_position((
                input_inner.x + cursor_x.min(input_inner.width.saturating_sub(1)),
                input_inner.y,
            ));
        }
    }
}
