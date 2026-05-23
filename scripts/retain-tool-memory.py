#!/usr/bin/env python3
"""Retain a structured tool memory into Hindsight default bank."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from datetime import datetime, timezone


def now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def stable_doc_id(record: dict) -> str:
    parts = [
        "tool-inventory",
        record.get("record_type", "record"),
        record.get("category", "general"),
        record.get("tool", "unknown"),
        record.get("task", "availability"),
    ]
    clean = []
    for part in parts:
        clean.append("".join(ch if ch.isalnum() else "-" for ch in str(part).lower()).strip("-"))
    return "-".join(p for p in clean if p)


def main() -> int:
    parser = argparse.ArgumentParser(description="Retain structured tool memory.")
    parser.add_argument("--record-type", required=True, choices=["availability", "recipe", "failure", "policy"])
    parser.add_argument("--category", required=True)
    parser.add_argument("--tool", required=True)
    parser.add_argument("--task", default="")
    parser.add_argument("--status", required=True)
    parser.add_argument("--path", default="")
    parser.add_argument("--version", default="")
    parser.add_argument("--command-template", default="")
    parser.add_argument("--failure-reason", default="")
    parser.add_argument("--confidence", type=float, default=None)
    parser.add_argument("--sync", action="store_true", help="Do not use Hindsight --async")
    parser.add_argument("--print-only", action="store_true", help="Print JSON without retaining")
    args = parser.parse_args()

    confidence = args.confidence
    if confidence is None:
        confidence = 0.95 if args.status == "verified_success" else 0.98 if args.status == "available" else 0.35

    record = {
        "namespace": "agent_tool_inventory",
        "memory_type": "tool_inventory",
        "record_type": args.record_type,
        "category": args.category,
        "tool": args.tool,
        "task": args.task,
        "status": args.status,
        "scope": "local_machine",
        "verified_at": now(),
        "confidence": confidence,
        "tags": ["tool-inventory", f"tool-category-{args.category}"],
    }
    if args.path:
        record["path"] = args.path
    if args.version:
        record["version"] = args.version
    if args.command_template:
        record["command_template"] = args.command_template
    if args.failure_reason:
        record["failure_reason"] = args.failure_reason

    content = json.dumps(record, ensure_ascii=False, sort_keys=True)
    if args.print_only:
        print(json.dumps(record, ensure_ascii=False, indent=2))
        return 0

    if not shutil.which("hindsight"):
        print("hindsight CLI not found", file=sys.stderr)
        return 2

    cmd = [
        "hindsight",
        "memory",
        "retain",
        "default",
        content,
        "-c",
        f"tool-inventory — {args.category}",
        "-d",
        stable_doc_id(record),
        "--document-tags",
        f"tool-inventory,tool-category-{args.category}",
        "--output",
        "json",
    ]
    if not args.sync:
        cmd.append("--async")

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=20)
    if result.returncode != 0:
        print(result.stderr or result.stdout, file=sys.stderr)
        return result.returncode
    print(result.stdout.strip() or content)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
