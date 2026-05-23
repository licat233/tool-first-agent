#!/usr/bin/env python3
"""Query the tool-first local registry by category or task text."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "registry" / "tools.yaml"


def load_yaml(path: Path) -> dict:
    try:
        import yaml  # type: ignore

        with path.open("r", encoding="utf-8") as f:
            return yaml.safe_load(f) or {}
    except Exception:
        pass

    ruby = subprocess.run(
        ["ruby", "-ryaml", "-rjson", "-e", "puts JSON.dump(YAML.load_file(ARGV[0]))", str(path)],
        capture_output=True,
        text=True,
    )
    if ruby.returncode == 0 and ruby.stdout.strip():
        return json.loads(ruby.stdout)

    yq = subprocess.run(["yq", "-o=json", ".", str(path)], capture_output=True, text=True)
    if yq.returncode == 0 and yq.stdout.strip():
        return json.loads(yq.stdout)

    raise RuntimeError("Could not parse tools.yaml. Install PyYAML, ruby, or yq.")


def matches_task(tool: dict, text: str) -> bool:
    haystack = " ".join(
        [
            " ".join(tool.get("handles", []) or []),
            " ".join((tool.get("commands", {}) or {}).keys()),
            " ".join((tool.get("commands", {}) or {}).values()),
        ]
    ).lower()
    return all(token in haystack for token in text.lower().split())


def main() -> int:
    parser = argparse.ArgumentParser(description="Query candidate tools from registry.")
    parser.add_argument("--category", help="Category name such as document, pdf, media, data")
    parser.add_argument("--task", default="", help="Optional task text to rank/filter tools")
    parser.add_argument("--json", action="store_true", help="Emit JSON only")
    args = parser.parse_args()

    data = load_yaml(REGISTRY)
    categories = [args.category] if args.category else sorted(data.keys())
    output = []

    for category in categories:
        if category not in data:
            continue
        section = data[category] or {}
        tools = section.get("tools", {}) or {}
        rows = []
        for name, spec in tools.items():
            spec = spec or {}
            if args.task and not matches_task(spec, args.task):
                # Keep category candidates visible, but rank non-matches later.
                match = False
            else:
                match = True
            rows.append(
                {
                    "category": category,
                    "tool": name,
                    "priority": int(spec.get("priority", 999)),
                    "match": match,
                    "detect_names": spec.get("detect_names", [name]),
                    "known_paths": spec.get("known_paths", []),
                    "app_bundle_paths": spec.get("app_bundle_paths", []),
                    "handles": spec.get("handles", []),
                    "commands": spec.get("commands", {}),
                    "fallbacks": spec.get("fallbacks", []),
                }
            )
        rows.sort(key=lambda r: (not r["match"], r["priority"], r["tool"]))
        output.extend(rows)

    if args.json:
        print(json.dumps(output, ensure_ascii=False, indent=2))
        return 0

    if not output:
        print("No candidates found.")
        return 1

    for row in output:
        marker = "*" if row["match"] else "-"
        print(f"{marker} {row['category']} / {row['tool']} (priority {row['priority']})")
        if row["handles"]:
            print(f"  handles: {'; '.join(row['handles'])}")
        if row["commands"]:
            for key, command in row["commands"].items():
                print(f"  {key}: {command}")
        print(f"  detect: {', '.join(row['detect_names'])}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"query-registry error: {exc}", file=sys.stderr)
        raise SystemExit(2)
