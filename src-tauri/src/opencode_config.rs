use crate::config::{home_dir, write_json_file};
use crate::error::AppError;
use crate::provider::OpenCodeProviderConfig;
use crate::settings::get_opencode_override_dir;
use indexmap::IndexMap;
use serde_json::{json, Map, Value};
use std::path::PathBuf;

/// 获取 OpenCode data 目录（~/.local/share/opencode）
pub fn get_opencode_data_dir() -> PathBuf {
    home_dir()
        .map(|home| home.join(".local").join("share").join("opencode"))
        .unwrap_or_else(|| PathBuf::from(".local").join("share").join("opencode"))
}

/// 获取 OpenCode auth.json 路径
pub fn get_opencode_auth_path() -> PathBuf {
    get_opencode_data_dir().join("auth.json")
}

/// 读取 auth.json，返回 { "provider-id": { "type": "api", "key": "..." } }
pub fn read_opencode_auth() -> Result<Map<String, Value>, AppError> {
    let path = get_opencode_auth_path();
    if !path.exists() {
        return Ok(Map::new());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
    let value: Value = serde_json::from_str(&content).map_err(|e| AppError::json(&path, e))?;
    Ok(value.as_object().cloned().unwrap_or_default())
}

/// 原子写入 auth.json
pub fn write_opencode_auth(auth: &Value) -> Result<(), AppError> {
    let path = get_opencode_auth_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
    }
    write_json_file(&path, auth)
}

/// 将 auth.json 中的 API key 合并到 provider 配置中
fn merge_auth_into_providers(
    providers: Map<String, Value>,
) -> Result<Map<String, Value>, AppError> {
    let auth = read_opencode_auth()?;
    if auth.is_empty() {
        return Ok(providers);
    }

    let mut merged = providers;
    for (id, auth_entry) in &auth {
        let key = auth_entry.get("key").and_then(Value::as_str);
        let Some(key) = key else { continue };

        if let Some(provider) = merged.get_mut(id) {
            // provider 已存在于 opencode.json，注入 apiKey（若未设置）
            if let Some(options) = provider.get_mut("options").and_then(Value::as_object_mut) {
                if !options.contains_key("apiKey") || options["apiKey"].as_str() == Some("") {
                    options.insert("apiKey".to_string(), Value::String(key.to_string()));
                }
            }
        } else {
            // provider 仅在 auth.json 中存在，构造最小条目
            let mut options = Map::new();
            options.insert("apiKey".to_string(), Value::String(key.to_string()));
            let mut provider = json!({
                "npm": "@ai-sdk/openai-compatible",
                "options": options,
                "models": {}
            });
            if let Some(type_val) = auth_entry.get("type") {
                provider["authType"] = type_val.clone();
            }
            merged.insert(id.clone(), provider);
        }
    }
    Ok(merged)
}

pub fn get_opencode_dir() -> PathBuf {
    if let Some(override_dir) = get_opencode_override_dir() {
        return override_dir;
    }

    home_dir()
        .map(|home| home.join(".config").join("opencode"))
        .unwrap_or_else(|| PathBuf::from(".config").join("opencode"))
}

pub fn get_opencode_config_path() -> PathBuf {
    get_opencode_dir().join("opencode.json")
}

pub fn read_opencode_config() -> Result<Value, AppError> {
    let path = get_opencode_config_path();
    if !path.exists() {
        return Ok(json!({ "$schema": "https://opencode.ai/config.json" }));
    }

    let content = std::fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
    serde_json::from_str(&content).map_err(|e| AppError::json(&path, e))
}

pub fn write_opencode_config(config: &Value) -> Result<(), AppError> {
    let path = get_opencode_config_path();
    write_json_file(&path, config)
}

pub fn get_providers() -> Result<Map<String, Value>, AppError> {
    let config = read_opencode_config()?;
    let providers = config
        .get("provider")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    merge_auth_into_providers(providers)
}

pub fn set_provider(id: &str, provider: Value) -> Result<(), AppError> {
    let mut full_config = read_opencode_config()?;

    if full_config.get("provider").is_none() {
        full_config["provider"] = json!({});
    }

    if let Some(providers) = full_config
        .get_mut("provider")
        .and_then(Value::as_object_mut)
    {
        providers.insert(id.to_string(), provider);
    }

    write_opencode_config(&full_config)
}

pub fn remove_provider(id: &str) -> Result<(), AppError> {
    let mut full_config = read_opencode_config()?;
    if let Some(providers) = full_config
        .get_mut("provider")
        .and_then(Value::as_object_mut)
    {
        providers.remove(id);
    }
    write_opencode_config(&full_config)?;

    // 同步清理 auth.json
    let mut auth = read_opencode_auth()?;
    if auth.remove(id).is_some() {
        write_opencode_auth(&Value::Object(auth))?;
    }
    Ok(())
}

pub fn get_typed_providers() -> Result<IndexMap<String, OpenCodeProviderConfig>, AppError> {
    let mut result = IndexMap::new();
    for (id, value) in get_providers()? {
        match serde_json::from_value::<OpenCodeProviderConfig>(value) {
            Ok(config) => {
                result.insert(id, config);
            }
            Err(err) => {
                log::warn!("Failed to parse OpenCode provider '{id}': {err}");
            }
        }
    }
    Ok(result)
}

pub fn set_typed_provider(id: &str, config: &OpenCodeProviderConfig) -> Result<(), AppError> {
    // 将 apiKey 同步到 auth.json
    if let Some(api_key) = &config.options.api_key {
        if !api_key.is_empty() {
            let mut auth = read_opencode_auth()?;
            let mut entry = Map::new();
            entry.insert("type".to_string(), Value::String("api".to_string()));
            entry.insert("key".to_string(), Value::String(api_key.clone()));
            auth.insert(id.to_string(), Value::Object(entry));
            write_opencode_auth(&Value::Object(auth))?;
        }
    }

    let value =
        serde_json::to_value(config).map_err(|source| AppError::JsonSerialize { source })?;
    set_provider(id, value)
}

/// 将单个 provider 的 apiKey 同步到 auth.json
pub fn sync_provider_auth(id: &str, api_key: &str) -> Result<(), AppError> {
    let mut auth = read_opencode_auth()?;
    if api_key.is_empty() {
        auth.remove(id);
    } else {
        let mut entry = Map::new();
        entry.insert("type".to_string(), Value::String("api".to_string()));
        entry.insert("key".to_string(), Value::String(api_key.to_string()));
        auth.insert(id.to_string(), Value::Object(entry));
    }
    write_opencode_auth(&Value::Object(auth))
}

pub fn get_mcp_servers() -> Result<Map<String, Value>, AppError> {
    let config = read_opencode_config()?;
    Ok(config
        .get("mcp")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default())
}

pub fn set_mcp_server(id: &str, server: Value) -> Result<(), AppError> {
    let mut full_config = read_opencode_config()?;

    if full_config.get("mcp").is_none() {
        full_config["mcp"] = json!({});
    }

    if let Some(mcp) = full_config.get_mut("mcp").and_then(Value::as_object_mut) {
        mcp.insert(id.to_string(), server);
    }

    write_opencode_config(&full_config)
}

pub fn remove_mcp_server(id: &str) -> Result<(), AppError> {
    let mut config = read_opencode_config()?;

    if let Some(mcp) = config.get_mut("mcp").and_then(Value::as_object_mut) {
        mcp.remove(id);
    }

    write_opencode_config(&config)
}
