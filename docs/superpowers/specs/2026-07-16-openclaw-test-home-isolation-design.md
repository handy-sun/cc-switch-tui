# OpenClaw Test HOME Isolation Design

## Problem

`src-tauri/tests/openclaw_config.rs` includes the production
`openclaw_config.rs` module directly. This also compiles and runs the module's
unit tests inside the integration-test binary. The integration test's mock
configuration layer ignores `CC_SWITCH_TEST_HOME` and still resolves the app
configuration directory as `$HOME/.cc-switch`, so nested tests can create
OpenClaw backups in a developer's real home directory.

## Design

Keep the existing source-inclusion test structure and correct its mock path
resolution. The mock `home_dir()` will prefer its in-memory override, then
`CC_SWITCH_TEST_HOME`, and only then fall back to `dirs::home_dir()`. Its mock
application configuration directory will use `.cc-switch-tui`, matching the
production default.

This keeps the change local to the integration-test harness. Production backup
creation and retention behavior remain unchanged.

## Regression Coverage

Add a serial integration test that gives `HOME` and `CC_SWITCH_TEST_HOME`
different temporary directories. It must prove that `get_app_config_dir()`
resolves to `CC_SWITCH_TEST_HOME/.cc-switch-tui`, not to either legacy
`$HOME/.cc-switch` or the real user home.

Verification will run the regression test, the duplicated nested OpenClaw unit
tests, and the complete test suite with one test thread. The real
`~/.cc-switch` tree will be fingerprinted before and after the relevant test
runs to confirm that no files are created or modified there.

## Scope

- Modify only the OpenClaw integration-test harness and its tests.
- Do not change production configuration or backup behavior.
- Do not delete the existing `~/.cc-switch` directory.
- Do not expose additional library APIs.
