# Hermes YAML Sequence Indentation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every block-sequence `-` written by the Hermes section serializer indent two spaces beyond its parent key, including nested block sequences.

**Architecture:** Keep `serde_yaml` as the data serializer, then normalize its indentless block-sequence layout in one private text-formatting helper before section replacement. Track active sequence indentation levels while walking serializer-generated lines so sequence items and their descendants shift together, without changing the YAML value or the surrounding section-preservation flow.

**Tech Stack:** Rust, `serde_yaml`, built-in unit tests, Cargo.

---

### Task 1: Add the sequence-indentation regression test

**Files:**
- Modify: `src-tauri/src/hermes_config.rs` in the `#[cfg(test)]` module near the section replacement tests

- [ ] **Step 1: Write the failing test**

Add a unit test that builds a section containing a top-level sequence with a nested sequence, serializes it through `serialize_yaml_section`, and checks exact output plus semantic round-trip:

```rust
#[test]
fn serialize_section_indents_sequence_items_beyond_parent_keys() {
    let value: serde_yaml::Value = serde_yaml::from_str(
        "- name: foo\n  aliases:\n  - first\n  - second\n- name: bar\n",
    )
    .unwrap();

    let serialized = serialize_yaml_section("custom_providers", &value).unwrap();

    assert_eq!(
        serialized,
        "custom_providers:\n  - name: foo\n    aliases:\n      - first\n      - second\n  - name: bar\n"
    );
    let reparsed: serde_yaml::Value = serde_yaml::from_str(&serialized).unwrap();
    assert_eq!(reparsed["custom_providers"], value);
}
```

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml hermes_config::tests::serialize_section_indents_sequence_items_beyond_parent_keys -- --exact
```

Expected: the test fails because current `serde_yaml` output places each `-` at the same indentation as its parent key.

### Task 2: Normalize serializer-generated block-sequence indentation

**Files:**
- Modify: `src-tauri/src/hermes_config.rs` near `serialize_yaml_section`

- [ ] **Step 1: Add the minimal formatter**

Add a private helper that walks serializer-generated lines, tracks original indentation levels where a block sequence begins, and adds two spaces for every active sequence level:

```rust
fn indent_yaml_sequences(yaml: &str) -> String {
    let mut sequence_indents = Vec::new();
    let mut output = String::with_capacity(yaml.len());

    for line in yaml.split_inclusive('\n') {
        let line_without_newline = line.strip_suffix('\n').unwrap_or(line);
        let indent = line_without_newline
            .bytes()
            .take_while(|byte| *byte == b' ')
            .count();
        let content = &line_without_newline[indent..];
        let is_sequence_item = content == "-" || content.starts_with("- ");

        if is_sequence_item {
            while sequence_indents.last().is_some_and(|level| *level > indent) {
                sequence_indents.pop();
            }
            if sequence_indents.last().copied() != Some(indent) {
                sequence_indents.push(indent);
            }
        } else if !content.is_empty() {
            while sequence_indents.last().is_some_and(|level| *level >= indent) {
                sequence_indents.pop();
            }
        }

        output.push_str(&" ".repeat(sequence_indents.len() * 2));
        output.push_str(line_without_newline);
        if line.ends_with('\n') {
            output.push('\n');
        }
    }

    output
}
```

- [ ] **Step 2: Apply the formatter at the section serializer boundary**

Change the end of `serialize_yaml_section` to return the normalized string:

```rust
Ok(indent_yaml_sequences(&yaml_str))
```

- [ ] **Step 3: Run the focused test and verify GREEN**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml hermes_config::tests::serialize_section_indents_sequence_items_beyond_parent_keys -- --exact
```

Expected: one test passes.

- [ ] **Step 4: Run all Hermes configuration tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml hermes_config::tests
```

Expected: all selected tests pass.

### Task 3: Review, verify, and commit

**Files:**
- Review: `src-tauri/src/hermes_config.rs`
- Include: `docs/superpowers/plans/2026-07-13-hermes-yaml-sequence-indentation.md`

- [ ] **Step 1: Format and inspect the change**

Run:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
git diff --check
git diff
```

Expected: formatting and whitespace checks pass; the diff is limited to the formatter, its integration, and its regression test.

- [ ] **Step 2: Run the complete Rust verification**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Expected: every command exits successfully with zero failures or warnings.

- [ ] **Step 3: Commit the verified implementation**

```bash
git add src-tauri/src/hermes_config.rs
git add -f docs/superpowers/plans/2026-07-13-hermes-yaml-sequence-indentation.md
git commit -m "fix(hermes): indent YAML sequence items"
```
