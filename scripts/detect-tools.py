#!/usr/bin/env python3
"""Detect only registry candidate tools for a category or explicit tool list."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "registry" / "tools.yaml"
CONFIG_DIR = Path.home() / ".config" / "tool-inventory"
CACHE = CONFIG_DIR / "inventory-cache.json"
KNOWN_DIRS = [
    Path.home() / ".local" / "bin",
    Path.home() / ".hermes" / "bin",
    Path("/opt/homebrew/bin"),
    Path("/usr/local/bin"),
    Path("/usr/bin"),
    Path("/bin"),
]


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


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def path_fingerprint() -> str:
    return hashlib.sha256(os.environ.get("PATH", "").encode()).hexdigest()[:16]


def find_executable(names: list[str], known_paths: list[str], app_paths: list[str]) -> tuple[str | None, str]:
    for name in names:
        found = shutil.which(name)
        if found:
            return found, "command_v"
    for candidate in known_paths + app_paths:
        p = Path(os.path.expanduser(candidate))
        if p.exists() and os.access(p, os.X_OK):
            return str(p), "known_path"
    for directory in KNOWN_DIRS:
        for name in names:
            p = directory / name
            if p.exists() and os.access(p, os.X_OK):
                return str(p), "known_bin_dir"
    return None, "not_found"


def version_for(path: str, args: list[str]) -> tuple[str, bool]:
    if not args:
        return "", True
    try:
        result = subprocess.run([path, *args], capture_output=True, text=True, timeout=4)
        text = (result.stdout or result.stderr).strip().splitlines()
        first = text[0][:240] if text else ""
        return first, result.returncode == 0 or bool(first)
    except Exception as exc:
        return str(exc), False


def collect_candidates(registry: dict, category: str | None, tools: list[str]) -> list[tuple[str, str, dict]]:
    rows = []
    for cat, section in registry.items():
        if category and cat != category:
            continue
        for tool, spec in (section.get("tools", {}) or {}).items():
            if tools and tool not in tools and not set(tools).intersection(spec.get("detect_names", []) or []):
                continue
            rows.append((cat, tool, spec or {}))
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description="Detect registry candidate tools.")
    parser.add_argument("--category", help="Detect all candidate tools in category")
    parser.add_argument("--tool", action="append", default=[], help="Specific tool to detect; repeatable")
    parser.add_argument("--json", action="store_true", help="Emit JSON only")
    parser.add_argument("--write-cache", action="store_true", help="Update ~/.config/tool-inventory/inventory-cache.json")
    args = parser.parse_args()

    if not args.category and not args.tool:
        parser.error("Use --category or --tool")

    registry = load_yaml(REGISTRY)
    rows = collect_candidates(registry, args.category, args.tool)
    results = []

    for category, tool, spec in rows:
        names = spec.get("detect_names", [tool]) or [tool]
        path, method = find_executable(
            names,
            spec.get("known_paths", []) or [],
            spec.get("app_bundle_paths", []) or [],
        )
        version = ""
        version_ok = False
        if path:
            version, version_ok = version_for(path, spec.get("version_args", []) or [])
        status = "available" if path and version_ok else "present_unverified" if path else "missing"
        results.append(
            {
                "namespace": "agent_tool_inventory",
                "memory_type": "tool_inventory",
                "record_type": "availability",
                "category": category,
                "tool": tool,
                "status": status,
                "path": path or "",
                "version": version,
                "detection_method": method,
                "checked_at": utc_now(),
                "path_fingerprint": path_fingerprint(),
                "confidence": 0.98 if status == "available" else 0.7 if status == "present_unverified" else 0.2,
                "tags": ["tool-inventory", f"tool-category-{category}"],
            }
        )

    if args.write_cache:
        CONFIG_DIR.mkdir(parents=True, exist_ok=True)
        existing = {}
        if CACHE.exists():
            try:
                existing = json.loads(CACHE.read_text(encoding="utf-8"))
            except Exception:
                existing = {}
        existing.setdefault("availability", {})
        for item in results:
            existing["availability"][item["tool"]] = item
        existing["updated_at"] = utc_now()
        existing["path_fingerprint"] = path_fingerprint()
        CACHE.write_text(json.dumps(existing, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.json:
        print(json.dumps(results, ensure_ascii=False, indent=2))
        return 0

    for item in results:
        loc = item["path"] or "-"
        ver = f" ({item['version']})" if item["version"] else ""
        print(f"{item['status']:18} {item['category']:10} {item['tool']:16} {loc}{ver}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"detect-tools error: {exc}", file=sys.stderr)
        raise SystemExit(2)
