# Hermes Provider Rename Design

## Goal

Make the identity behind `custom_providers[].name` visible without treating it
like an ordinary display-name field, and provide a safe rename operation that
keeps CC Switch state and Hermes routing references consistent.

## Chosen UX

- Hermes provider edit forms show `Provider ID` as a read-only field before
  the friendly `Name` field.
- On provider list and detail routes, `r` opens a rename text input only for
  writable Hermes `custom_providers` rows.
- Built-in rows keep their current behavior and cannot be renamed.
- Rows sourced only from Hermes' `providers:` dictionary remain read-only and
  cannot be renamed.
- The friendly `Name` remains independently editable and continues to control
  the label shown by CC Switch.
- Adding a provider keeps the existing generated-ID behavior; the new read-only
  ID row is only needed while editing an existing provider.

This separates a stable routing identity from a presentation label. A regular
editable ID field was rejected because partially applying an edit could leave
the database, `custom_providers`, and `model.provider` out of sync. Requiring
users to delete and recreate providers was also rejected because it is
needlessly disruptive and loses the existing identity migration opportunity.

## Rename Flow

1. The TUI pre-fills the selected provider ID in a text-input overlay.
2. Submission trims the new ID and dispatches a dedicated Hermes rename action.
3. The provider service validates that:
   - the old provider exists and is sourced from `custom_providers`;
   - the new ID is non-empty and differs from the old ID;
   - no saved provider, live custom provider, `providers:` dictionary entry, or
     configured built-in provider conflicts with the normalized new ID.
4. The local provider manager replaces the old key with the new key at the
   same list position and updates the stored provider object's `id`. Its
   friendly display `name` is preserved.
5. A Hermes configuration rename operation changes the matching
   `custom_providers[].name`. If the top-level `model.provider` resolves to the
   old identity, it is changed to the raw new ID as well.
6. The YAML update is written once from one in-memory source string, producing
   one backup and preventing an intermediate state where only one section has
   changed.
7. If either local persistence or the live YAML write fails, the existing
   provider transaction/backup mechanism restores both sides.
8. After success, provider data is reloaded. A detail route for the old ID is
   redirected to the new ID, and a localized success toast is shown.

## Validation and Errors

- Empty IDs keep the rename input open and show a warning.
- Entering the unchanged ID closes as a no-op or reports that nothing changed;
  it must not rewrite YAML.
- Exact and normalized collisions are rejected. Normalization follows Hermes'
  runtime identity rules: trim, lowercase, replace spaces with hyphens, and
  ignore a leading `custom:` reference prefix when comparing routing names.
- Built-in and `providers:` rows never expose the rename shortcut.
- Rename errors are localized and leave the original provider usable.

## Test Strategy

Tests are written before implementation and cover:

- configuration-level rename of `custom_providers[].name`;
- updating `model.provider` only when it points at the old provider;
- preservation of unrelated YAML sections, provider fields, and model fields;
- collision, blank-ID, missing-provider, built-in, and dict-only rejection;
- service-level local key migration, display-name preservation, ordering, and
  rollback behavior;
- list/detail `r` handling, input submission, route redirection, and localized
  key hints;
- read-only ID rendering in the Hermes edit form;
- the full serial Rust test suite and `cargo check` before completion.

## Scope Boundaries

- No general rename support is added for other applications.
- Model IDs and model display names are unchanged.
- Existing Hermes provider aliases are only used for collision/reference
  resolution; this feature does not introduce a permanent alias table.
- No automatic rename is inferred from edits to the friendly display name.
