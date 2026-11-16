#!/usr/bin/env python3
"""Regenerate or verify the pinned OpenAPI and AsyncAPI operation snapshots.

The script intentionally uses only Python's standard library. It reads the
small, stable YAML subset used by Deepgram's modular operation definitions:
mapping keys, `$ref`, `operationId`, `action`, and
`x-fern-sdk-method-name`. It is not a general YAML parser.

Run from the Rust SDK repository:

    python3 scripts/refresh_parity_inventory.py \
      --docs-dir ../deepgram-docs --check

The docs checkout must be at the exact revision in ``PARITY_LEDGER.yml``.
Use ``--write`` after deliberately updating the source pin.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "PARITY_LEDGER.yml"
OPENAPI_SNAPSHOT = ROOT / "parity" / "openapi-operations.json"
ASYNCAPI_SNAPSHOT = ROOT / "parity" / "asyncapi-operations.json"

HTTP_METHODS = {"get", "post", "put", "patch", "delete"}


def ledger_value(name: str) -> str:
    """Read a simple scalar from the parity ledger."""
    pattern = re.compile(rf"^\s*{re.escape(name)}:\s*([^\s#]+)", re.MULTILINE)
    match = pattern.search(LEDGER.read_text())
    if match is None:
        raise ValueError(f"missing {name!r} in {LEDGER}")
    return match.group(1)


def docs_revision(docs_dir: Path) -> str:
    """Return the checked-out commit for a Deepgram docs worktree."""
    return subprocess.check_output(
        ["git", "-C", str(docs_dir), "rev-parse", "HEAD"],
        text=True,
    ).strip()


def top_level_block(document: str, key: str) -> list[str]:
    """Return the unindented YAML mapping block for ``key``."""
    lines = document.splitlines()
    start = next(
        (
            index
            for index, line in enumerate(lines)
            if re.fullmatch(rf"{re.escape(key)}:\s*", line)
        ),
        None,
    )
    if start is None:
        raise ValueError(f"could not find top-level YAML key {key!r}")

    end = len(lines)
    for index in range(start + 1, len(lines)):
        if lines[index] and not lines[index].startswith((" ", "\t", "#")):
            end = index
            break
    return lines[start:end]


def ref_value(lines: list[str]) -> str:
    """Read the sole `$ref` beneath a path entry."""
    for line in lines:
        match = re.fullmatch(r'\s+\$ref:\s*["\']?([^"\']+)["\']?\s*', line)
        if match:
            return match.group(1)
    raise ValueError(f"could not find $ref in path entry:\n{chr(10).join(lines)}")


def decode_pointer_fragment(fragment: str) -> str:
    """Decode the one-component pointer fragments used by the modular specs."""
    if not fragment.startswith("#/"):
        raise ValueError(f"unsupported reference fragment {fragment!r}")
    pointer = fragment.removeprefix("#/")
    if "/" in pointer:
        raise ValueError(f"unsupported multi-component reference fragment {fragment!r}")
    return pointer.replace("~1", "/").replace("~0", "~")


def operation_entries_from_path_block(document: str, key: str) -> list[dict[str, str]]:
    """Extract HTTP methods and operation IDs from one referenced path item."""
    block = top_level_block(document, key)
    operations: list[dict[str, str]] = []
    for index, line in enumerate(block):
        method = re.fullmatch(r"\s{2}(get|post|put|patch|delete):\s*", line)
        if method is None:
            continue
        end = len(block)
        for next_index in range(index + 1, len(block)):
            if re.fullmatch(r"\s{2}(get|post|put|patch|delete):\s*", block[next_index]):
                end = next_index
                break
        operation_id = next(
            (
                id_match.group(1)
                for candidate in block[index + 1 : end]
                if (id_match := re.fullmatch(r"\s+operationId:\s*(\S+)\s*", candidate))
            ),
            None,
        )
        if operation_id is None:
            raise ValueError(f"{key!r} {method.group(1).upper()} has no operationId")
        operations.append({"operation_id": operation_id, "method": method.group(1).upper()})
    return operations


def openapi_inventory(docs_dir: Path) -> list[dict[str, str]]:
    """Resolve the modular OpenAPI path references into an operation inventory."""
    root = docs_dir / "api/specs/openapi/openapi.yml"
    lines = root.read_text().splitlines()
    paths_start = next(index for index, line in enumerate(lines) if line == "paths:")
    paths_end = next(
        index
        for index in range(paths_start + 1, len(lines))
        if lines[index] and not lines[index].startswith((" ", "\t", "#"))
    )

    entries: list[dict[str, str]] = []
    index = paths_start + 1
    while index < paths_end:
        match = re.fullmatch(r"\s{2}(/[^:]+):\s*", lines[index])
        if match is None:
            index += 1
            continue
        path = match.group(1)
        end = index + 1
        while end < paths_end and not re.fullmatch(r"\s{2}/[^:]+:\s*", lines[end]):
            end += 1
        reference = ref_value(lines[index + 1 : end])
        filename, fragment = reference.split("#", 1)
        target = (root.parent / filename).read_text()
        key = decode_pointer_fragment(f"#{fragment}")
        for operation in operation_entries_from_path_block(target, key):
            entries.append({**operation, "path": path})
        index = end

    return entries


def asyncapi_inventory(docs_dir: Path) -> list[dict[str, Any]]:
    """Extract every declared AsyncAPI operation from modular operation files."""
    entries: list[dict[str, Any]] = []
    for operation_file in sorted((docs_dir / "api/specs/asyncapi/operations").glob("*.yml")):
        lines = operation_file.read_text().splitlines()
        index = 0
        while index < len(lines):
            operation = re.fullmatch(r"([A-Za-z0-9]+):\s*", lines[index])
            if operation is None:
                index += 1
                continue
            end = index + 1
            while end < len(lines) and not re.fullmatch(r"[A-Za-z0-9]+:\s*", lines[end]):
                end += 1
            block = lines[index + 1 : end]
            action = next(
                (
                    action_match.group(1)
                    for line in block
                    if (action_match := re.fullmatch(r"\s+action:\s*(send|receive)\s*", line))
                ),
                None,
            )
            if action is None:
                raise ValueError(
                    f"{operation_file.name}:{operation.group(1)} has no action"
                )
            method = next(
                (
                    method_match.group(1)
                    for line in block
                    if (
                        method_match := re.fullmatch(
                            r"\s+x-fern-sdk-method-name:\s*(\S+)\s*", line
                        )
                    )
                ),
                None,
            )
            entries.append(
                {
                    "schema_file": operation_file.name,
                    "operation": operation.group(1),
                    "action": action,
                    "sdk_method": method,
                }
            )
            index = end
    return entries


def write_or_check(path: Path, inventory: list[dict[str, Any]], write: bool) -> None:
    """Write one normalized JSON snapshot, or reject drift in check mode."""
    rendered = json.dumps(inventory, indent=2) + "\n"
    if write:
        path.write_text(rendered)
    elif path.read_text() != rendered:
        raise ValueError(
            f"{path.relative_to(ROOT)} differs from the pinned spec inventory; "
            "run this command with --write after reviewing the source revision"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--docs-dir",
        type=Path,
        required=True,
        help="Deepgram docs checkout at the revision recorded in PARITY_LEDGER.yml",
    )
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="verify checked-in snapshots")
    mode.add_argument("--write", action="store_true", help="rewrite checked-in snapshots")
    args = parser.parse_args()

    expected_revision = ledger_value("revision")
    actual_revision = docs_revision(args.docs_dir)
    if actual_revision != expected_revision:
        raise SystemExit(
            f"docs revision mismatch: ledger requires {expected_revision}, "
            f"but {args.docs_dir} is at {actual_revision}"
        )

    rest = openapi_inventory(args.docs_dir)
    stream = asyncapi_inventory(args.docs_dir)
    if len({entry["operation_id"] for entry in rest}) != 50:
        raise SystemExit(f"expected 50 unique OpenAPI operations, found {len(rest)}")
    if len({(entry["schema_file"], entry["operation"]) for entry in stream}) != 84:
        raise SystemExit(f"expected 84 unique AsyncAPI operations, found {len(stream)}")

    write_or_check(OPENAPI_SNAPSHOT, rest, args.write)
    write_or_check(ASYNCAPI_SNAPSHOT, stream, args.write)
    print(
        f"{'wrote' if args.write else 'verified'} 50 OpenAPI and "
        "84 AsyncAPI operations"
    )


if __name__ == "__main__":
    main()
