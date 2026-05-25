#!/usr/bin/env python3

"""Update the project version in Cargo.toml and Cargo.lock."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


PROJECT_NAME = "cc-switch-tui"
SEMVER_RE = re.compile(
    r"^(?:v)?(?P<version>(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*))$"
)


@dataclass(frozen=True)
class PlannedChange:
    path: Path
    old: str
    new: str


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def normalize_version(raw: str) -> str:
    match = SEMVER_RE.fullmatch(raw.strip())
    if not match:
        raise ValueError("Version must use X.Y.Z format, for example 1.2.3 or v1.2.3")
    return match.group("version")


def replace_package_version_in_cargo_toml(text: str, version: str) -> tuple[str, str]:
    in_package = False
    old_version: str | None = None
    output: list[str] = []

    for line in text.splitlines(keepends=True):
        stripped = line.strip()
        if stripped == "[package]":
            in_package = True
        elif stripped.startswith("[") and stripped.endswith("]"):
            in_package = False

        if in_package and stripped.startswith("version"):
            match = re.match(
                r'(?P<prefix>\s*version\s*=\s*")'
                r'(?P<version>[^"]+)'
                r'(?P<suffix>".*?)'
                r'(?P<newline>\r?\n)?$',
                line,
            )
            if not match:
                raise ValueError("Could not parse package version in Cargo.toml")
            old_version = match.group("version")
            line = (
                f'{match.group("prefix")}{version}{match.group("suffix")}'
                f'{match.group("newline") or ""}'
            )

        output.append(line)

    if old_version is None:
        raise ValueError("Could not find [package] version in Cargo.toml")
    return "".join(output), old_version


def replace_package_version_in_cargo_lock(text: str, version: str) -> tuple[str, str]:
    package_block_re = re.compile(
        rf'(?ms)(\[\[package\]\]\nname = "{re.escape(PROJECT_NAME)}"\nversion = ")([^"]+)(")'
    )
    match = package_block_re.search(text)
    if not match:
        raise ValueError(f"Could not find {PROJECT_NAME} package in Cargo.lock")
    return package_block_re.sub(rf"\g<1>{version}\3", text, count=1), match.group(2)


def plan_change(path: Path, old_version: str, version: str) -> PlannedChange | None:
    if old_version == version:
        return None
    return PlannedChange(path=path, old=old_version, new=version)


def update_files(version: str, dry_run: bool) -> list[PlannedChange]:
    root = repo_root()
    cargo_toml = root / "src-tauri" / "Cargo.toml"
    cargo_lock = root / "src-tauri" / "Cargo.lock"

    toml_text = cargo_toml.read_text(encoding="utf-8")
    lock_text = cargo_lock.read_text(encoding="utf-8")

    next_toml, old_toml_version = replace_package_version_in_cargo_toml(toml_text, version)
    next_lock, old_lock_version = replace_package_version_in_cargo_lock(lock_text, version)

    changes = [
        change
        for change in [
            plan_change(cargo_toml, old_toml_version, version),
            plan_change(cargo_lock, old_lock_version, version),
        ]
        if change is not None
    ]

    if not dry_run:
        cargo_toml.write_text(next_toml, encoding="utf-8")
        cargo_lock.write_text(next_lock, encoding="utf-8")

    return changes


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Update cc-switch-tui version in Cargo.toml and Cargo.lock."
    )
    parser.add_argument("version", help="Target version, for example 1.2.3 or v1.2.3")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show planned changes without writing files.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Return non-zero if files are not already at the target version.",
    )
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        version = normalize_version(args.version)
        changes = update_files(version, dry_run=args.dry_run or args.check)
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    if not changes:
        print(f"Version is already {version}.")
        return 0

    for change in changes:
        print(f"{change.path.relative_to(repo_root())}: {change.old} -> {change.new}")

    if args.check:
        return 1
    if args.dry_run:
        print("Dry run only; no files were changed.")
    else:
        print(f"Updated project version to {version}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
