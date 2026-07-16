# OpenClaw Test HOME Isolation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent the OpenClaw integration-test binary from writing duplicated unit-test backups into a developer's real `~/.cc-switch` directory.

**Architecture:** Keep the existing source-inclusion integration tests, but make their mock configuration resolver follow the production test-home contract. Add a regression test around the mock resolver, then verify both the targeted nested test and the full suite without changing the real legacy directory.

**Tech Stack:** Rust, Cargo test harness, `serial_test`, `tempfile`.

---

### Task 1: Guard the mock application configuration directory

**Files:**
- Modify: `src-tauri/tests/openclaw_config.rs:67-82`
- Test: `src-tauri/tests/openclaw_config.rs`

- [ ] **Step 1: Write the failing regression test**

Add a serial test with an RAII environment guard. Give `HOME` and
`CC_SWITCH_TEST_HOME` separate temporary paths, clear the in-memory override,
and assert that the mock resolver returns the test-home path with the current
application directory name:

```rust
#[test]
#[serial]
fn mock_app_config_dir_prefers_test_home_and_current_directory_name() {
    let home = tempfile::tempdir().expect("create parent home");
    let test_home = tempfile::tempdir().expect("create test home");
    let _env = TestHomeEnvGuard::set(home.path(), test_home.path());

    assert_eq!(
        config::get_app_config_dir(),
        test_home.path().join(".cc-switch-tui")
    );
}
```

The guard must save and restore `HOME`, `CC_SWITCH_TEST_HOME`, and the previous
in-memory test-home override even if the assertion panics.

- [ ] **Step 2: Run the regression test and verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test openclaw_config \
  mock_app_config_dir_prefers_test_home_and_current_directory_name \
  -- --exact --nocapture --test-threads=1
```

Expected: FAIL because the mock resolver ignores `CC_SWITCH_TEST_HOME` and
returns the legacy `$HOME/.cc-switch` path.

- [ ] **Step 3: Implement the minimal mock resolver fix**

Change the test-only resolver to:

```rust
pub fn home_dir() -> Option<PathBuf> {
    crate::test_support::test_home_override()
        .or_else(|| std::env::var_os("CC_SWITCH_TEST_HOME").map(PathBuf::from))
        .or_else(dirs::home_dir)
}

pub fn get_app_config_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cc-switch-tui")
}
```

- [ ] **Step 4: Run the regression test and verify GREEN**

Run the command from Step 2.

Expected: PASS with one test and zero failures.

### Task 2: Prove nested and full-suite isolation

**Files:**
- Verify: `src-tauri/tests/openclaw_config.rs`
- Verify unchanged: `~/.cc-switch`

- [ ] **Step 1: Format and inspect the change**

Run:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
git diff --check
git diff -- src-tauri/tests/openclaw_config.rs
```

Expected: formatting and whitespace checks pass; the code diff stays within
the integration-test harness.

- [ ] **Step 2: Run the duplicated nested OpenClaw tests**

Fingerprint `~/.cc-switch`, run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test openclaw_config \
  'openclaw_config_impl::tests' -- --test-threads=1
```

Expected: all selected tests pass and the real-directory fingerprint remains
identical.

- [ ] **Step 3: Run the complete test suite**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1
```

Expected: zero test failures and no changes under the real `~/.cc-switch`.

- [ ] **Step 4: Commit the implementation**

```bash
git add src-tauri/tests/openclaw_config.rs
git commit -m "fix(test): isolate OpenClaw backup directory"
```
