# Hermes Provider Identity and Built-in Rows

## Goal

Make Hermes provider selection use the provider identifiers stored in Hermes configuration, while also showing configured Hermes built-in API-key providers as selectable, read-only rows at the end of the provider table.

## Identity contract

Hermes exposes two different namespaces:

- User-defined providers are stored in `custom_providers[].name` or under a `providers:` key.
- Hermes may refer to a user-defined provider at runtime as `custom:<normalized-name>`.
- Built-in providers use Hermes catalog slugs such as `deepseek`, `xiaomi`, and `copilot`.

CC Switch will keep the configuration identifier as the canonical identity for a user-defined provider. A runtime reference such as `custom:sensen` will resolve to the existing `sensen` row instead of creating or importing a second provider. Matching will account for Hermes' lowercase and space-to-hyphen normalization and for a `providers:` dictionary key that differs from its display name.

When a user-defined provider is selected, CC Switch will write its original identifier to `model.provider`. It will not persist the `custom:` runtime prefix. Existing synthetic `custom:*` rows will be hidden or removed only when their `_cc_source: model_section` metadata and a matching canonical provider make the migration unambiguous. Unknown references remain visible and are not destructively rewritten.

## Built-in provider catalog

CC Switch will maintain a small Rust descriptor catalog mirroring Hermes' API-key provider registry. Each descriptor contains only non-secret metadata:

- Hermes slug;
- display name;
- accepted credential environment-variable names;
- optional public base URL.

Availability is determined only from non-empty values in `${HERMES_HOME}/.env`. Shell variables, OAuth state, `auth.json`, external secret managers, and tool-only tokens are outside this change. Values are used only as booleans and are never returned in a `Provider`, logged, written to the database, synchronized through WebDAV, or rendered in the TUI.

The dotenv reader will accept ordinary `KEY=value`, optional `export`, whitespace, and quoted values. Invalid lines are ignored. Tests use placeholder values and verify that values never enter the generated row.

## Table representation

Configured built-ins are synthesized while loading the Hermes `ProvidersSnapshot`; they are not persisted.

- Internal row ID: `builtin:<slug>`, for example `builtin:deepseek`.
- Display name: `[Built-in] DeepSeek` / `[内置] DeepSeek`.
- Category: `builtin`.
- Ordering: all saved/user-defined rows first using their existing order, then built-ins in catalog order.
- Capabilities: switch and filter only. Edit, delete, import, reorder, provider tests, and WebDAV persistence are disabled.

The namespaced row ID prevents a built-in `deepseek` row from colliding inside the TUI with a user-defined row named `deepseek`. The namespace is presentation-only and is never written to Hermes configuration.

Hermes itself gives a raw canonical built-in slug precedence over a same-named custom provider. Under the required no-`custom:` write contract, such a custom row cannot be selected safely; CC Switch will leave it visible but reject switching with a rename instruction instead of silently routing it to the built-in.

## Switching

Selecting `builtin:<slug>` validates that the descriptor is still configured in `${HERMES_HOME}/.env`, strips the presentation namespace, and writes `<slug>` to `model.provider`. Because the synthesized row does not contain a model catalog, the existing `model.default` is preserved. Provider-specific top-level `base_url` and `api_key` values are cleared so the built-in resolves credentials through Hermes' `.env` behavior rather than inheriting a previous custom provider's secrets.

After reload, a raw built-in slug marks the matching built-in row current. A resolved custom runtime reference marks only the canonical custom row current.

## Error handling

- If `.env` is absent, no built-in rows are added.
- If `.env` cannot be read, provider loading continues with user-defined rows and returns a non-secret warning through the existing error/logging conventions.
- If a token is removed between table load and selection, switching fails without changing `config.yaml`.
- Shared tool credentials such as a generic `GITHUB_TOKEN` do not enable a built-in row; only provider-owned credential variables such as `COPILOT_GITHUB_TOKEN` count.
- Unknown `custom:*` references remain visible as live-only model-section entries so the user does not lose access to an unexplained configuration.

## Tests

Regression tests will cover:

1. `custom:sensen` resolving to `sensen` without a duplicate row.
2. Normalized case/space matching and `providers:` key versus display-name matching.
3. Unknown runtime references remaining intact.
4. Only non-empty configured built-in credentials producing rows.
5. Built-in rows appearing after saved rows with the `[内置]`/`[Built-in]` marker.
6. Built-in rows refusing edit/delete actions while allowing Space switching.
7. Built-in switching writing the raw slug, preserving `model.default`, and clearing stale custom credentials.
8. Token values never appearing in synthesized provider data.

## Non-goals

- Reading OAuth or `auth.json` account state.
- Fetching built-in model catalogs or probing provider APIs.
- Persisting built-in rows or their credentials in the CC Switch database.
- Changing Hermes model selection beyond preserving the current default.
- Changing provider behavior for Claude, Codex, Gemini, OpenCode, or OpenClaw.
