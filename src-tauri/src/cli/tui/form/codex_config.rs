/// Maximum allowed value for token count fields (100 million).
pub(crate) const MAX_TOKEN_COUNT: u64 = 100_000_000;

/// Parse a string as a positive integer token count.
/// Returns `Some(value)` if the input is a valid positive integer <= MAX_TOKEN_COUNT,
/// or `None` if the input is empty (treat as "not set").
/// Returns `Err(message)` if the input is invalid (non-numeric, zero, negative, or too large).
pub(crate) fn parse_token_count(raw: &str) -> Result<Option<u64>, &'static str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let value: u64 = trimmed.parse().map_err(|_| {
        if is_chinese() {
            "请输入正整数"
        } else {
            "Must be a positive integer"
        }
    })?;
    if value == 0 {
        return Err(if is_chinese() {
            "值必须大于 0"
        } else {
            "Value must be greater than 0"
        });
    }
    if value > MAX_TOKEN_COUNT {
        return Err(if is_chinese() {
            "值不能超过 1 亿"
        } else {
            "Value must not exceed 100 million"
        });
    }
    Ok(Some(value))
}

pub(crate) fn validate_token_count_pair(
    auto_compact_raw: &str,
    context_window_raw: &str,
) -> Option<&'static str> {
    let auto_compact = match parse_token_count(auto_compact_raw) {
        Ok(value) => value,
        Err(_) => return Some(crate::cli::i18n::texts::codex_auto_compact_token_limit_invalid()),
    };
    let context_window = match parse_token_count(context_window_raw) {
        Ok(value) => value,
        Err(_) => return Some(crate::cli::i18n::texts::codex_context_window_invalid()),
    };

    if let (Some(limit), Some(window)) = (auto_compact, context_window) {
        if limit > window {
            return Some(crate::cli::i18n::texts::codex_auto_compact_exceeds_context_window());
        }
    }

    None
}

pub(crate) fn validate_codex_config_token_counts(config: &str) -> Option<&'static str> {
    let table: toml::Table = match toml::from_str(config.trim()) {
        Ok(table) => table,
        Err(_) => return None,
    };
    let Some(section) = selected_provider_section(&table) else {
        return None;
    };

    let auto_compact = match validate_token_count_value(
        section.get("model_auto_compact_token_limit"),
        crate::cli::i18n::texts::codex_auto_compact_token_limit_invalid(),
    ) {
        Ok(value) => value,
        Err(message) => return Some(message),
    };
    let context_window = match validate_token_count_value(
        section.get("model_context_window"),
        crate::cli::i18n::texts::codex_context_window_invalid(),
    ) {
        Ok(value) => value,
        Err(message) => return Some(message),
    };

    if let (Some(limit), Some(window)) = (auto_compact, context_window) {
        if limit > window {
            return Some(crate::cli::i18n::texts::codex_auto_compact_exceeds_context_window());
        }
    }

    None
}

fn validate_token_count_value(
    value: Option<&toml::Value>,
    invalid_message: &'static str,
) -> Result<Option<u64>, &'static str> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(value) = value.as_integer() else {
        return Err(invalid_message);
    };
    if value <= 0 {
        return Err(invalid_message);
    }
    let Ok(value) = u64::try_from(value) else {
        return Err(invalid_message);
    };
    if value > MAX_TOKEN_COUNT {
        return Err(crate::cli::i18n::texts::codex_token_count_exceeds_max());
    }
    Ok(Some(value))
}

fn is_chinese() -> bool {
    crate::cli::i18n::is_chinese()
}

use super::CodexWireApi;

#[derive(Debug, Default)]
pub(crate) struct ParsedCodexConfigSnippet {
    pub(crate) base_url: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) wire_api: Option<CodexWireApi>,
    pub(crate) requires_openai_auth: Option<bool>,
    pub(crate) env_key: Option<String>,
    pub(crate) model_auto_compact_token_limit: Option<u64>,
    pub(crate) model_context_window: Option<u64>,
}

pub(crate) fn parse_codex_config_snippet(cfg: &str) -> ParsedCodexConfigSnippet {
    let mut out = ParsedCodexConfigSnippet::default();
    let table: toml::Table = match toml::from_str(cfg.trim()) {
        Ok(table) => table,
        Err(_) => return out,
    };

    out.model = table
        .get("model")
        .and_then(|value| value.as_str())
        .map(String::from);

    let section = selected_provider_section(&table);

    if let Some(section) = section {
        out.base_url = section
            .get("base_url")
            .and_then(|value| value.as_str())
            .map(String::from);
        out.wire_api = section
            .get("wire_api")
            .and_then(|value| value.as_str())
            .and_then(|value| match value {
                "chat" => Some(CodexWireApi::Chat),
                "responses" => Some(CodexWireApi::Responses),
                _ => None,
            });
        out.requires_openai_auth = section
            .get("requires_openai_auth")
            .and_then(|value| value.as_bool());
        out.env_key = section
            .get("env_key")
            .and_then(|value| value.as_str())
            .map(String::from);
        out.model_auto_compact_token_limit = section
            .get("model_auto_compact_token_limit")
            .and_then(|value| value.as_integer())
            .and_then(|v| u64::try_from(v).ok());
        out.model_context_window = section
            .get("model_context_window")
            .and_then(|value| value.as_integer())
            .and_then(|v| u64::try_from(v).ok());
    }

    out
}

fn selected_provider_section(table: &toml::Table) -> Option<&toml::Table> {
    table
        .get("model_provider")
        .and_then(|value| value.as_str())
        .and_then(|key| {
            table
                .get("model_providers")
                .and_then(|value| value.as_table())
                .and_then(|providers| providers.get(key))
                .and_then(|value| value.as_table())
        })
}

pub(crate) fn update_codex_config_snippet(
    original: &str,
    base_url: &str,
    model: &str,
    wire_api: CodexWireApi,
    requires_openai_auth: bool,
    env_key: &str,
    model_auto_compact_token_limit: u64,
    model_context_window: u64,
) -> String {
    let mut doc = match original.trim().parse::<toml_edit::DocumentMut>() {
        Ok(doc) => doc,
        Err(_) => return original.to_string(),
    };

    if let Some(model) = non_empty(model) {
        doc["model"] = toml_edit::value(model);
    } else {
        doc.remove("model");
    }

    let provider_key = doc
        .get("model_provider")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());

    if let Some(key) = provider_key {
        if doc.get("model_providers").is_none() {
            doc["model_providers"] = toml_edit::Item::Table(toml_edit::Table::new());
        }
        let providers = doc["model_providers"]
            .as_table_like_mut()
            .expect("model_providers should be a table");
        if providers.get(&key).is_none() {
            providers.insert(&key, toml_edit::Item::Table(toml_edit::Table::new()));
        }

        if let Some(section) = providers
            .get_mut(&key)
            .and_then(|value| value.as_table_like_mut())
        {
            if let Some(base_url) = non_empty(base_url) {
                section.insert("base_url", toml_edit::value(base_url));
            } else {
                section.remove("base_url");
            }

            section.insert("wire_api", toml_edit::value(wire_api.as_str()));
            section.insert(
                "requires_openai_auth",
                toml_edit::value(requires_openai_auth),
            );

            if requires_openai_auth {
                section.remove("env_key");
            } else {
                let env_key = non_empty(env_key).unwrap_or("OPENAI_API_KEY");
                section.insert("env_key", toml_edit::value(env_key));
            }

            match parse_token_count(&model_auto_compact_token_limit.to_string()) {
                Ok(Some(v)) => {
                    section.insert(
                        "model_auto_compact_token_limit",
                        toml_edit::value(i64::try_from(v).unwrap_or(i64::MAX)),
                    );
                }
                _ => {
                    section.remove("model_auto_compact_token_limit");
                }
            }

            match parse_token_count(&model_context_window.to_string()) {
                Ok(Some(v)) => {
                    section.insert(
                        "model_context_window",
                        toml_edit::value(i64::try_from(v).unwrap_or(i64::MAX)),
                    );
                }
                _ => {
                    section.remove("model_context_window");
                }
            }
        }
    }

    let result = doc.to_string();
    let trimmed = result.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn clean_codex_provider_key(provider_id: &str, provider_name: &str) -> String {
    let raw = if provider_id.trim().is_empty() {
        provider_name.trim()
    } else {
        provider_id.trim()
    };
    crate::codex_config::clean_codex_provider_key(raw)
}

pub(crate) fn build_codex_provider_config_toml(
    provider_key: &str,
    base_url: &str,
    model: &str,
    wire_api: CodexWireApi,
) -> String {
    let provider_key = escape_toml_string(provider_key);
    let model = escape_toml_string(model);
    let base_url = escape_toml_string(base_url);

    [
        format!("model_provider = \"{}\"", provider_key),
        format!("model = \"{}\"", model),
        "model_reasoning_effort = \"high\"".to_string(),
        "disable_response_storage = true".to_string(),
        String::new(),
        format!("[model_providers.{}]", provider_key),
        format!("name = \"{}\"", provider_key),
        format!("base_url = \"{}\"", base_url),
        format!("wire_api = \"{}\"", wire_api.as_str()),
        "requires_openai_auth = true".to_string(),
        String::new(),
    ]
    .join("\n")
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn escape_toml_string(value: &str) -> String {
    value.replace('"', "\\\"")
}
