#!/usr/bin/env python3
"""Verify one exact, behavior-preserving artifact-owner source transfer."""

from __future__ import annotations

import argparse
import fnmatch
import re
import subprocess
import sys
from dataclasses import dataclass


TRANSFER_PATH = "scripts/validation/artifact-owner-transfers.txt"
OWNER_PATH = "scripts/validation/artifact-version-owners.txt"


def git(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args], check=check, text=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL
    )


def show(revision: str, path: str) -> str:
    return git("show", f"{revision}:{path}").stdout


def exists(revision: str, path: str) -> bool:
    return git("cat-file", "-e", f"{revision}:{path}", check=False).returncode == 0


def policy_owns(revision: str, owner_class: str, path: str) -> bool:
    for raw in show(revision, OWNER_PATH).splitlines():
        if not raw or raw.startswith("#"):
            continue
        fields = raw.split("|")
        if len(fields) >= 2 and fields[0] == owner_class and fnmatch.fnmatchcase(path, fields[1]):
            return True
    return False


@dataclass
class LexState:
    block_comment: bool = False
    string_quote: str | None = None
    raw_hashes: int | None = None
    escaped: bool = False


def brace_delta(line: str, state: LexState) -> tuple[int, bool]:
    delta = 0
    saw_open = False
    index = 0
    while index < len(line):
        if state.block_comment:
            end = line.find("*/", index)
            if end < 0:
                return delta, saw_open
            state.block_comment = False
            index = end + 2
            continue
        if state.raw_hashes is not None:
            end_token = '"' + ("#" * state.raw_hashes)
            end = line.find(end_token, index)
            if end < 0:
                return delta, saw_open
            state.raw_hashes = None
            index = end + len(end_token)
            continue
        if state.string_quote is not None:
            char = line[index]
            if state.escaped:
                state.escaped = False
            elif char == "\\":
                state.escaped = True
            elif char == state.string_quote:
                state.string_quote = None
            index += 1
            continue
        if line.startswith("//", index):
            break
        if line.startswith("/*", index):
            state.block_comment = True
            index += 2
            continue
        raw = re.match(r'r(#+)?"', line[index:])
        if raw:
            state.raw_hashes = len(raw.group(1) or "")
            index += len(raw.group(0))
            continue
        char = line[index]
        if char == '"':
            state.string_quote = char
            state.escaped = False
        elif char == "{":
            delta += 1
            saw_open = True
        elif char == "}":
            delta -= 1
        index += 1
    return delta, saw_open


NAMED_ITEM = re.compile(
    r"^(?:pub(?:\([^)]*\))?\s+)?(const|static|struct|enum|fn|type|trait)\s+"
    r"([A-Za-z_][A-Za-z0-9_]*)(?=\s*(?::|\{|\(|<|where\b|;|$))"
)
IMPL_ITEM = re.compile(
    r"^impl\s+(.+?)(?=\s*(?:\{|where\b|$))"
)


def item_key(line: str) -> str | None:
    stripped = line.strip()
    match = NAMED_ITEM.match(stripped)
    if match:
        kind, name = match.groups()
        return f"{kind} {name}"
    match = IMPL_ITEM.match(stripped)
    if not match:
        return None
    header = re.sub(r"\s+", " ", match.group(1).strip())
    return f"impl {header}"


def rust_items(source: str) -> dict[str, list[str]]:
    lines = source.splitlines(keepends=True)
    items: dict[str, list[str]] = {}
    state = LexState()
    depth = 0
    pending_attribute: int | None = None
    active_key: str | None = None
    active_start = 0
    active_saw_open = False
    for line_number, line in enumerate(lines):
        stripped = line.strip()
        if active_key is None and depth == 0:
            if stripped.startswith("#["):
                pending_attribute = line_number if pending_attribute is None else pending_attribute
            else:
                key = item_key(stripped)
                if key is not None:
                    active_key = key
                    active_start = pending_attribute if pending_attribute is not None else line_number
                    active_saw_open = False
                elif stripped and not stripped.startswith("//"):
                    pending_attribute = None
        delta, saw_open = brace_delta(line, state)
        depth += delta
        if active_key is not None:
            active_saw_open = active_saw_open or saw_open
            if (active_saw_open and depth == 0) or (not active_saw_open and stripped.endswith(";")):
                text = "".join(lines[active_start : line_number + 1]).rstrip() + "\n"
                items.setdefault(active_key, []).append(text)
                active_key = None
                pending_attribute = None
        if depth < 0:
            raise ValueError("unbalanced Rust source")
    if active_key is not None or depth != 0:
        raise ValueError("unterminated Rust item")
    return items


def fail(message: str) -> int:
    print(message, file=sys.stderr)
    return 1


def relocation_equivalent(before: str, after: str) -> bool:
    """Allow only the intra-parent visibility needed after a module extraction."""
    if before == after:
        return True
    return before == re.sub(r"^(\s*)pub\(super\)\s+", r"\1", after, flags=re.MULTILINE)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--class", dest="owner_class", required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--before", required=True)
    parser.add_argument("--after", required=True)
    args = parser.parse_args()

    try:
        after_transfers = show(args.after, TRANSFER_PATH).splitlines()
    except subprocess.CalledProcessError:
        return fail("artifact owner transfer declaration is missing")
    before_transfers = (
        show(args.before, TRANSFER_PATH).splitlines()
        if exists(args.before, TRANSFER_PATH)
        else []
    )
    matches = []
    for raw in after_transfers:
        if not raw or raw.startswith("#"):
            continue
        fields = raw.split("|")
        if len(fields) != 4:
            return fail(f"malformed artifact owner transfer declaration: {raw}")
        if fields[:3] == [args.owner_class, args.source, args.target]:
            matches.append((raw, fields[3]))
    if len(matches) != 1:
        return fail("artifact owner transfer must have exactly one matching declaration")
    declaration, items_path = matches[0]
    if declaration in before_transfers:
        return fail("artifact owner transfer declaration is not on the transfer edge")
    if exists(args.before, args.target) or not exists(args.after, args.target):
        return fail("artifact owner transfer target must be absent before and present after")
    if not exists(args.before, args.source) or not exists(args.after, args.source):
        return fail("artifact owner transfer source must exist before and after")
    if not policy_owns(args.before, args.owner_class, args.source):
        return fail("artifact owner transfer source was not registered in the same class")
    if not policy_owns(args.after, args.owner_class, args.target):
        return fail("artifact owner transfer target is not registered in the same class")
    if policy_owns(args.after, args.owner_class, args.source):
        return fail("artifact owner transfer source remains registered after extraction")
    try:
        expected_keys = [
            line for line in show(args.after, items_path).splitlines() if line and not line.startswith("#")
        ]
        before_items = rust_items(show(args.before, args.source))
        after_source_items = rust_items(show(args.after, args.source))
        target_items = rust_items(show(args.after, args.target))
    except (subprocess.CalledProcessError, ValueError) as error:
        return fail(f"unable to derive artifact owner transfer fingerprint: {error}")
    if len(expected_keys) != len(set(expected_keys)) or not expected_keys:
        return fail("artifact owner transfer item inventory must be nonempty and unique")
    if set(target_items) != set(expected_keys):
        return fail("artifact owner transfer target contains missing or undeclared Rust items")
    for key in expected_keys:
        if len(before_items.get(key, [])) != 1 or len(target_items.get(key, [])) != 1:
            return fail(f"artifact owner transfer item {key!r} is missing or duplicated")
        if not relocation_equivalent(before_items[key][0], target_items[key][0]):
            return fail(f"artifact owner transfer item {key!r} changed during extraction")
        if key in after_source_items:
            return fail(f"artifact owner transfer item {key!r} remains in the prior owner")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
