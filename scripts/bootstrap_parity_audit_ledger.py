#!/usr/bin/env python3
"""Generate the operation-level Rust parity audit ledger from pinned snapshots."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OPENAPI = ROOT / "parity" / "openapi-operations.json"
ASYNCAPI = ROOT / "parity" / "asyncapi-operations.json"
OUTPUT = ROOT / "parity" / "operation-audit-ledger.json"


def main() -> None:
    openapi = json.loads(OPENAPI.read_text())
    asyncapi = json.loads(ASYNCAPI.read_text())

    ledger = {
        "schema_version": 1,
        "status": "untriaged",
        "instructions": {
            "status_values": [
                "untriaged",
                "implemented",
                "missing_rust_implementation",
                "intentional_rust_native_difference",
                "api_spec_defect",
                "product_contract_blocker",
            ],
            "rule": "Replace untriaged only after comparing the pinned spec, "
            "Rust, and all three generated SDK snapshots. Record evidence and "
            "a concrete test plan for every non-implemented row.",
        },
        "openapi_operations": [
            {
                "id": f"openapi:{entry['operation_id']}",
                "operation_id": entry["operation_id"],
                "method": entry["method"],
                "path": entry["path"],
                "status": "untriaged",
                "review_owner": "rust-sdk-parity",
                "audit_scope": [
                    "request path, query, headers, and body",
                    "response fields, enums, nullability, and errors",
                    "generated SDK surface comparison",
                ],
                "test_plan": "Capture the Rust request or deserialize a pinned "
                "response fixture for every implemented surface.",
            }
            for entry in openapi
        ],
        "asyncapi_operations": [
            {
                "id": f"asyncapi:{entry['schema_file']}:{entry['operation']}:{entry['action']}",
                "schema_file": entry["schema_file"],
                "operation": entry["operation"],
                "action": entry["action"],
                "generated_sdk_method": entry["sdk_method"],
                "status": "untriaged",
                "review_owner": "rust-sdk-parity",
                "audit_scope": [
                    "message fields, enums, defaults, and unknown values",
                    "client-control or server-event direction",
                    "generated SDK surface comparison",
                ],
                "test_plan": "Serialize a client message or deserialize a pinned "
                "server-frame fixture, matching the operation direction.",
            }
            for entry in asyncapi
        ],
    }
    OUTPUT.write_text(json.dumps(ledger, indent=2) + "\n")
    print(
        f"wrote {len(openapi)} OpenAPI and {len(asyncapi)} AsyncAPI audit rows to {OUTPUT.relative_to(ROOT)}"
    )


if __name__ == "__main__":
    main()
