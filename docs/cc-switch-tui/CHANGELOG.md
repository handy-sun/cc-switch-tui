# Changelog

## [0.2.2] - 2026-07-10

### Added

- Structured Hermes multi-model editor for model IDs, context lengths, and output token limits.

### Fixed

- Persist Hermes model parameters with native `context_length` and `max_tokens` field names.
- Migrate historical Hermes model dictionaries and legacy parameter aliases into the structured editor.
- Preserve provider Headers and unknown provider/model fields while applying explicit field and model deletions.
- Keep live Hermes top-level model defaults scoped to edits of the active provider.
- Reconcile Codex provider catalog identity, renamed provider keys, active URLs, and stored snapshots.
- Commit the highlighted fetched model when Enter is pressed with an empty picker input.
- Preserve OpenClaw and Hermes MCP enablement flags when saving MCP forms.

## [0.2.1] - 2026-06-11

### Added

- OpenCode provider discovery from `auth.json` alongside `opencode.json`.
- Configurable save shortcut with UI hint display.

### Fixed

- Respect Claude config dir for MCP sync.
- Show built-in Hermes providers from model config in provider list.
- Ensure Hermes provider edits are always persisted to `config.yaml`.
- Respect `HERMES_HOME` env var for Hermes config directory resolution.
- Normalize Ctrl+Shift+letter shortcuts for crossterm compatibility.

## [0.1.4] - 2026-05-21

### Added

- Codex MCP live drift detection: surface config.toml changes made outside cc-switch directly in the TUI.
- Hermes provider catalog import: press `i` on the Providers page to import live providers from the Hermes config, merge by stable key, and create new saved providers for unrecognized entries.
- Codex merge forward-compatibility regression guard: unknown root-level preference keys from live config are preserved by default on provider switch, so new Codex preferences are never silently discarded.

### Fixed

- Preserve user comments in Codex `config.toml` across provider switches; root-level, inline, and commented-out subtable comments survive in-place merge.
- Preserve Codex MCP drift during implicit syncs so external edits to `[mcp_servers]` are not overwritten.
- Reconcile live current provider drift: detect when `config.toml` points at a different provider than cc-switch's stored one and surface the actual live choice.
- Sync Hermes credentials on provider switch: update `model.base_url` and `model.api_key` alongside provider and model defaults; clear stale credentials when the target provider does not supply them.
- Delete live Hermes providers without panic on missing or malformed entries.
- Limit failover provider switch guard to active-proxy state: allow normal provider switching when automatic failover is enabled but the local proxy is not routing traffic.
- Absorb upstream WebDAV upload readback fix: remove probe write/read/delete round trip and post-PUT manifest GET check.

### Changed

- Refactor Codex merge from preference-key whitelist to provider-scoped blacklist: only `model_provider`, `model`, `model_providers`, and `projects` are hard-synced from snapshot; all other root keys follow the user-preference preservation rule.
- Absorb upstream TUI footer shortcut refinements and prompt list order stabilization.

## [0.1.3] - 2026-05-18

### Added

- Codex provider catalog import: press `i` on the Providers page to read live providers from `~/.codex/config.toml`, merge by stable catalog key, and create new saved providers for unrecognized entries.
- Auto-repair conflicting custom provider keys: detect duplicate `custom` keys in `[model_providers]` before sync and rewrite them to unique keys derived from provider id/name.
- Provider key rewrite primitives for config snapshots: rename a provider table key, rewrite profile references, and update root model_provider.
- Skill sync method setting exposed in the TUI.

### Fixed

- Honor `CODEX_HOME` for MCP live sync instead of assuming the default path.
- Preserve migrated user settings during config migration.
- Keep tests off real config directories and isolate `cargo test` home by default.

### Changed

- Optimize Codex provider catalog import and sync: keep TUI-managed custom providers mirrored into the live `config.toml` `[model_providers.*]` table; tolerate broken legacy snapshots instead of aborting unrelated operations.
- Update Rust toolchain baseline.

### Removed

- Remove unused TUI actions, provider proxy code, and config helpers.

## [0.1.2] - 2026-05-13

### Added

- Add OpenClaw MCP management support across the CLI/TUI app model.
- Show the installed OpenClaw CLI version in the TUI home local environment check.
- Add visual selection mode for skills management.
- Add OpenClaw skill support and align agent app columns.

### Fixed

- Keep OpenClaw and Hermes app switches persisted in TUI state.
- Prune stale OpenClaw agent model catalog entries when providers are removed.
- Align the OpenClaw current provider marker and default provider keyboard handling.
- Reconcile live app skill enablement and skip managed or bundled skills during agent import.
- Adapt upstream sync changes for cc-switch-tui.

## [0.1.1] — 2026-05-11

### Added

- Publish the Rust crate to crates.io during tagged release workflows.

### Fixed

- Fix OpenClaw provider switching and default model writes when valid upstream config uses flexible default model shapes or empty object values.
- Keep TUI app switching responsive during startup and accept localized app switch hotkey labels.
- Run legacy config directory migration before startup database initialization.

## [0.1.0] — 2026-05-10

Initial release of the renamed cc-switch-tui fork.

### Added

- CC_SWITCH_TUI_CONFIG_DIR env var to override config directory (with `~` expansion)
- Auto-migration from legacy `~/.cc-switch/` to `~/.cc-switch-tui/`
- Hermes support: provider management, MCP, skills, prompts, proxy
- OpenClaw support: provider management, MCP, prompts, proxy
- Interactive prompt for legacy config directory migration

### Changed

- Rename project from cc-switch-cli to cc-switch-tui (package, binaries, config paths)
- Repository URL updated to github.com/handy-sun/cc-switch-tui
- Description updated to include Hermes and OpenClaw

### Fixed

- Embedded line numbers in flake.nix and generate_latest_json.py
- MCP table rendering for Hermes column
- TUI picker navigation bounds for 6-app layout

### Removed

- Sponsor section from README files and partner assets

[0.2.2]: https://github.com/handy-sun/cc-switch-tui/releases/tag/v0.2.2
[0.2.1]: https://github.com/handy-sun/cc-switch-tui/releases/tag/v0.2.1
[0.1.4]: https://github.com/handy-sun/cc-switch-tui/releases/tag/v0.1.4
[0.1.3]: https://github.com/handy-sun/cc-switch-tui/releases/tag/v0.1.3
[0.1.2]: https://github.com/handy-sun/cc-switch-tui/releases/tag/v0.1.2
[0.1.1]: https://github.com/handy-sun/cc-switch-tui/releases/tag/v0.1.1
[0.1.0]: https://github.com/handy-sun/cc-switch-tui/releases/tag/v0.1.0
