# Hermes Provider Rename Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe Hermes custom-provider rename operation while showing the provider ID as read-only in the edit form.

**Architecture:** A configuration-layer primitive performs one atomic YAML rewrite for `custom_providers` and an active `model.provider` reference. `ProviderService` wraps local key migration and the live write in the existing snapshot/rollback transaction. The TUI exposes the operation only for writable Hermes custom rows through a text-input action and keeps the friendly display name independent.

**Tech Stack:** Rust 2021, serde/serde_yaml/serde_json, IndexMap-backed provider state, Ratatui TUI, existing `AppState` transaction and localized-text infrastructure.

---

## File Map

- `src-tauri/src/hermes_config.rs`: validate and atomically rename the live YAML identity.
- `src-tauri/src/services/provider/mod.rs`: migrate the saved provider key and coordinate rollback.
- `src-tauri/src/services/provider/tests.rs`: service-level state, ordering, collision, and rollback coverage.
- `src-tauri/src/cli/tui/app/types.rs`: add the rename text-submit payload.
- `src-tauri/src/cli/tui/app/app_state.rs`: add the provider rename runtime action.
- `src-tauri/src/cli/tui/app/content_entities.rs`: expose `r` for writable Hermes custom rows.
- `src-tauri/src/cli/tui/app/overlay_handlers/dialogs.rs`: validate and submit rename input.
- `src-tauri/src/cli/tui/runtime_actions/mod.rs`: dispatch the rename action.
- `src-tauri/src/cli/tui/runtime_actions/providers.rs`: invoke the service, redirect detail routes, reload data, and toast.
- `src-tauri/src/cli/tui/form/provider_state.rs`: include provider ID in Hermes edit fields only.
- `src-tauri/src/cli/tui/app/form_handlers/provider.rs`: make that Hermes edit ID field non-editable.
- `src-tauri/src/cli/tui/ui/providers.rs`: render rename key hints only where supported.
- `src-tauri/src/cli/i18n.rs` and `src-tauri/src/cli/i18n/texts/providers.rs`: mirrored English/Chinese rename labels, prompts, errors, and success text.
- `src-tauri/src/cli/tui/app/tests.rs`, `src-tauri/src/cli/tui/ui/tests.rs`, and `src-tauri/src/cli/tui/form/tests.rs`: interaction, rendering, and form-field regression tests.

### Task 1: Atomic Hermes YAML Rename

**Files:**
- Modify: `src-tauri/src/hermes_config.rs`

- [ ] **Step 1: Write failing configuration tests**

Add tests beside the existing provider CRUD tests that exercise the public API:

```rust
#[test]
fn rename_provider_updates_custom_name_and_active_model_reference_once() {
    with_temp_hermes_config(
        r#"model:
  provider: custom:old-provider
  default: model-a
custom_providers:
  - name: old-provider
    base_url: https://old.example/v1
    models:
      model-a:
        context_length: 128000
agent:
  max_turns: 42
"#,
        || {
            rename_provider("old-provider", "new-provider").expect("rename provider");
            let config = read_hermes_config().expect("read renamed config");
            assert_eq!(config["custom_providers"][0]["name"], "new-provider");
            assert_eq!(config["model"]["provider"], "new-provider");
            assert_eq!(config["agent"]["max_turns"], 42);
            assert_eq!(
                config["custom_providers"][0]["models"]["model-a"]["context_length"],
                128000
            );
        },
    );
}
```

Add separate tests proving that an unrelated `model.provider` remains unchanged, blank/new-name collisions fail without changing the file, missing and dict-only old IDs fail, and normalized collisions such as `New Provider` versus `new-provider` fail.

- [ ] **Step 2: Run the tests and verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml rename_provider_ -- --test-threads=1
```

Expected: compilation fails because `hermes_config::rename_provider` does not exist.

- [ ] **Step 3: Implement the atomic rename primitive**

Add a public API and a private comparison helper:

```rust
fn normalized_provider_identity(value: &str) -> String {
    normalize_hermes_runtime_provider_name(value.trim().strip_prefix("custom:").unwrap_or(value))
}

pub fn rename_provider(old_name: &str, new_name: &str) -> Result<HermesWriteOutcome, AppError> {
    let old_name = old_name.trim();
    let new_name = new_name.trim();
    if new_name.is_empty() {
        return Err(AppError::Config("Hermes provider ID cannot be empty".to_string()));
    }
    if old_name == new_name {
        return Ok(HermesWriteOutcome::default());
    }

    let _guard = hermes_write_lock().lock()?;
    let config_path = get_hermes_config_path();
    let raw = fs::read_to_string(&config_path).map_err(|e| AppError::io(&config_path, e))?;
    let mut config: serde_yaml::Value = serde_yaml::from_str(&raw)
        .map_err(|e| AppError::Config(format!("Failed to parse Hermes config as YAML: {e}")))?;
    ensure_provider_writable(&config, old_name, "rename")?;

    let providers = config
        .get_mut("custom_providers")
        .and_then(serde_yaml::Value::as_sequence_mut)
        .ok_or_else(|| AppError::Config(format!("Provider '{old_name}' does not exist")))?;
    let provider = providers
        .iter_mut()
        .find(|provider| provider.get("name").and_then(serde_yaml::Value::as_str) == Some(old_name))
        .ok_or_else(|| AppError::Config(format!("Provider '{old_name}' does not exist")))?;
    provider["name"] = serde_yaml::Value::String(new_name.to_string());

    let mut new_raw = replace_yaml_section(
        &raw,
        "custom_providers",
        config.get("custom_providers").expect("section exists"),
    )?;
    if let Some(model) = config.get_mut("model").and_then(serde_yaml::Value::as_mapping_mut) {
        let provider_key = serde_yaml::Value::String("provider".to_string());
        let points_to_old = model
            .get(&provider_key)
            .and_then(serde_yaml::Value::as_str)
            .is_some_and(|value| normalized_provider_identity(value) == normalized_provider_identity(old_name));
        if points_to_old {
            model.insert(provider_key, serde_yaml::Value::String(new_name.to_string()));
            new_raw = replace_yaml_section(
                &new_raw,
                "model",
                config.get("model").expect("section exists"),
            )?;
        }
    }

    let backup_path = Some(create_hermes_backup(&raw)?);
    atomic_write(&config_path, new_raw.as_bytes())?;
    Ok(HermesWriteOutcome {
        backup_path: backup_path.map(|path| path.display().to_string()),
    })
}
```

Before locating the mutable list entry, add exact and normalized collision checks against `get_providers()` and `configured_builtin_providers()`. Exclude `old_name` itself from those checks. Preserve all provider fields and unrelated top-level YAML text.

- [ ] **Step 4: Run configuration tests and verify GREEN**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml rename_provider_ -- --test-threads=1
```

Expected: all rename configuration tests pass with zero failures.

- [ ] **Step 5: Commit the configuration primitive**

```bash
git add src-tauri/src/hermes_config.rs
git commit -m "feat(hermes): rename live custom provider identities"
```

### Task 2: Transactional Saved-State Migration

**Files:**
- Modify: `src-tauri/src/services/provider/mod.rs`
- Modify: `src-tauri/src/services/provider/tests.rs`

- [ ] **Step 1: Write failing service tests**

Add tests proving that `ProviderService::rename_hermes_provider`:

```rust
let renamed = ProviderService::rename_hermes_provider(&state, "old-provider", "new-provider")?;
assert!(renamed);
let manager = state.config.read().unwrap().get_manager(&AppType::Hermes).unwrap();
assert!(!manager.providers.contains_key("old-provider"));
let provider = &manager.providers["new-provider"];
assert_eq!(provider.id, "new-provider");
assert_eq!(provider.name, "Friendly Name");
assert_eq!(manager.providers.get_index(0).unwrap().0, "new-provider");
```

Also cover saved-ID collision, a configured built-in collision, a dict-only source, and rollback when the live write fails. The rollback assertion must verify both the old local key and the original live YAML are restored.

- [ ] **Step 2: Run the service test and verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml rename_hermes_provider -- --test-threads=1
```

Expected: compilation fails because the service method does not exist.

- [ ] **Step 3: Extend the post-commit action**

Add an optional payload:

```rust
#[derive(Clone)]
struct HermesProviderRename {
    old_id: String,
    new_id: String,
}

struct PostCommitAction {
    // existing fields
    hermes_provider_rename: Option<HermesProviderRename>,
}
```

Initialize it to `None` at every existing construction site. In `apply_post_commit`, execute `hermes_config::rename_provider` when the payload exists; otherwise keep the existing snapshot-write behavior.

- [ ] **Step 4: Implement the service method**

Add:

```rust
pub fn rename_hermes_provider(
    state: &AppState,
    old_id: &str,
    new_id: &str,
) -> Result<bool, AppError>
```

Validate Hermes-only writable source metadata and all saved/live/built-in collisions before mutation. Inside `run_transaction`, remove the old IndexMap entry, update `Provider.id`, and reinsert it at the original index. Capture a Hermes live snapshot and emit a post-commit rename payload. Do not change `Provider.name` or model settings.

- [ ] **Step 5: Run service tests and verify GREEN**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml rename_hermes_provider -- --test-threads=1
```

Expected: all service rename and rollback tests pass.

- [ ] **Step 6: Commit the service transaction**

```bash
git add src-tauri/src/services/provider/mod.rs src-tauri/src/services/provider/tests.rs
git commit -m "feat(hermes): migrate saved state during provider rename"
```

### Task 3: TUI Rename Interaction and Read-Only ID

**Files:**
- Modify: `src-tauri/src/cli/tui/app/types.rs`
- Modify: `src-tauri/src/cli/tui/app/app_state.rs`
- Modify: `src-tauri/src/cli/tui/app/content_entities.rs`
- Modify: `src-tauri/src/cli/tui/app/overlay_handlers/dialogs.rs`
- Modify: `src-tauri/src/cli/tui/runtime_actions/mod.rs`
- Modify: `src-tauri/src/cli/tui/runtime_actions/providers.rs`
- Modify: `src-tauri/src/cli/tui/form/provider_state.rs`
- Modify: `src-tauri/src/cli/tui/app/form_handlers/provider.rs`
- Modify: `src-tauri/src/cli/tui/ui/providers.rs`
- Modify: `src-tauri/src/cli/i18n.rs`
- Modify: `src-tauri/src/cli/i18n/texts/providers.rs`
- Modify: `src-tauri/src/cli/tui/app/tests.rs`
- Modify: `src-tauri/src/cli/tui/ui/tests.rs`
- Modify: `src-tauri/src/cli/tui/form/tests.rs`

- [ ] **Step 1: Write failing form and interaction tests**

Add tests for these observable behaviors:

```rust
assert_eq!(
    ProviderAddFormState::from_provider(AppType::Hermes, &provider).fields()[0],
    ProviderAddField::Id
);
```

Verify that Enter/typing on the Hermes edit ID row does not change `form.id`, while the same field behavior for OpenClaw remains unchanged.

For a selected writable Hermes custom row, assert that `r` opens:

```rust
Overlay::TextInput(TextInputState {
    submit: TextSubmit::HermesProviderRename { old_id },
    input,
    ..
})
```

with `old_id` and `input.value` equal to the current provider ID. Assert that built-in and `providers_dict` rows do not open the overlay. Assert that blank submission keeps the input open, and valid submission returns `Action::HermesProviderRename { old_id, new_id }`.

- [ ] **Step 2: Write failing rendering tests**

Assert that Hermes edit mode renders the ID before Name, the ID is marked read-only, and provider list/detail key bars show `r rename` only for writable custom rows. Built-in rows with quota support must continue to show `r refresh` instead.

- [ ] **Step 3: Run TUI tests and verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml hermes_provider_rename -- --test-threads=1
cargo test --manifest-path src-tauri/Cargo.toml hermes_edit_form_shows_read_only_id -- --test-threads=1
```

Expected: tests fail because the new submit/action variants and key handling do not exist.

- [ ] **Step 4: Add localized text and action types**

Add mirrored functions for rename title, prompt, key label, blank/collision/read-only errors, and success toast in both i18n sources. Add:

```rust
TextSubmit::HermesProviderRename { old_id: String }
Action::HermesProviderRename { old_id: String, new_id: String }
```

- [ ] **Step 5: Implement scoped input and dispatch**

Add a helper that returns true only when:

```rust
app_type == AppType::Hermes
    && !data::is_hermes_builtin_row(row)
    && row.provider.settings_config[PROVIDER_SOURCE_FIELD] == PROVIDER_SOURCE_CUSTOM_LIST
```

Use it in list/detail `r` handling and key-bar rendering. Submit trimmed values through the new action. Runtime dispatch calls `ProviderService::rename_hermes_provider`, redirects `Route::ProviderDetail { id: old_id }` to the new ID, clears filters if necessary, reloads provider data, and shows the localized success toast.

- [ ] **Step 6: Implement the read-only edit ID row**

In `ProviderAddFormState::fields`, insert `ProviderAddField::Id` at index zero only for Hermes edit mode. In field activation/edit handling, ignore text-edit keys for that specific combination while keeping navigation active. Render a localized read-only hint in the value or field hint without modifying the stored ID.

- [ ] **Step 7: Run TUI tests and verify GREEN**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml hermes_provider_rename -- --test-threads=1
cargo test --manifest-path src-tauri/Cargo.toml hermes_edit_form_shows_read_only_id -- --test-threads=1
cargo test --manifest-path src-tauri/Cargo.toml hermes_provider_list_key_bar -- --test-threads=1
```

Expected: all targeted interaction and rendering tests pass.

- [ ] **Step 8: Commit the TUI feature**

```bash
git add src-tauri/src/cli/i18n.rs src-tauri/src/cli/i18n/texts/providers.rs \
  src-tauri/src/cli/tui/app src-tauri/src/cli/tui/form/provider_state.rs \
  src-tauri/src/cli/tui/runtime_actions src-tauri/src/cli/tui/ui/providers.rs \
  src-tauri/src/cli/tui/ui/tests.rs src-tauri/src/cli/tui/form/tests.rs
git commit -m "feat(hermes): add safe provider rename interaction"
```

### Task 4: Final Verification and Review

**Files:**
- Verify all files changed by Tasks 1-3.

- [ ] **Step 1: Run focused Hermes tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml hermes_provider -- --test-threads=1
```

Expected: zero failures.

- [ ] **Step 2: Run the complete serial suite**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1
```

Expected: all unit, integration, and documentation tests pass.

- [ ] **Step 3: Run compiler and formatting checks**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
rustfmt --edition 2021 --check <each changed Rust file>
git diff --check
```

Expected: commands exit zero; existing unrelated `dead_code` warnings may remain.

- [ ] **Step 4: Review the complete branch diff**

Check that no secrets, unrelated formatting, general provider rename behavior, or model-name behavior entered the implementation. Obtain an independent review of the complete implementation diff and resolve blocking findings.

- [ ] **Step 5: Create the final verification commit if review fixes were needed**

```bash
git add <review-fix-files>
git commit -m "fix(hermes): address provider rename review findings"
```

Skip this commit when the review requires no changes.
