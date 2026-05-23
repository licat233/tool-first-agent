#!/usr/bin/env python3
"""Register or update a tool in registry/tools.yaml."""

from __future__ import annotations

import argparse
import json
import shlex
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "registry" / "tools.yaml"
DETECT = ROOT / "scripts" / "detect-tools.py"
RETAIN = ROOT / "scripts" / "retain-tool-memory.py"


def load_yaml(path: Path) -> dict:
    try:
        import yaml  # type: ignore

        return yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    except Exception:
        pass

    result = subprocess.run(
        ["ruby", "-ryaml", "-rjson", "-e", "puts JSON.dump(YAML.load_file(ARGV[0]))", str(path)],
        capture_output=True,
        text=True,
    )
    if result.returncode == 0 and result.stdout.strip():
        return json.loads(result.stdout)
    raise RuntimeError("Could not parse tools.yaml. Install PyYAML or Ruby.")


def dump_yaml(data: dict, path: Path) -> None:
    try:
        import yaml  # type: ignore

        path.write_text(
            yaml.safe_dump(data, sort_keys=False, allow_unicode=True, default_flow_style=False),
            encoding="utf-8",
        )
        return
    except Exception:
        pass

    result = subprocess.run(
        ["ruby", "-ryaml", "-rjson", "-e", "STDOUT.write(JSON.parse(STDIN.read).to_yaml)"],
        input=json.dumps(data, ensure_ascii=False),
        capture_output=True,
        text=True,
    )
    if result.returncode == 0 and result.stdout.strip():
        path.write_text(result.stdout, encoding="utf-8")
        return
    raise RuntimeError("Could not write YAML. Install PyYAML or Ruby.")


def parse_command(value: str) -> tuple[str, str]:
    if "=" not in value:
        raise argparse.ArgumentTypeError("--command must be name='template'")
    name, template = value.split("=", 1)
    name = name.strip()
    template = template.strip()
    if not name or not template:
        raise argparse.ArgumentTypeError("--command must be name='template'")
    return name, template


def split_values(values: list[str]) -> list[str]:
    result: list[str] = []
    for value in values:
        result.extend(item.strip() for item in value.split(",") if item.strip())
    return result


def ask(label: str, default: str = "") -> str:
    suffix = f" [{default}]" if default else ""
    value = input(f"{label}{suffix}: ").strip()
    return value or default


def ask_many(label: str) -> list[str]:
    value = ask(f"{label} (comma-separated)")
    return [item.strip() for item in value.split(",") if item.strip()]


def interactive_fill(args: argparse.Namespace) -> argparse.Namespace:
    args.category = args.category or ask("Category")
    if not args.category:
        raise SystemExit("Category is required.")
    if args.priority is None:
        args.priority = int(ask("Priority, lower is preferred", "50"))
    if not args.detect_name:
        args.detect_name = ask_many(f"Detect names for {args.tool}") or [args.tool]
    if not args.handle:
        handle = ask("What does this tool handle?")
        args.handle = [handle] if handle else []
    if not args.command:
        commands = []
        while True:
            key = ask("Command key, empty to stop")
            if not key:
                break
            template = ask("Command template")
            if template:
                commands.append((key, template))
        args.command = commands
    if not args.fallback:
        args.fallback = ask_many("Fallback tools")
    return args


def build_spec(args: argparse.Namespace) -> dict:
    detect_names = split_values(args.detect_name) or [args.tool]
    known_paths = split_values(args.known_path)
    found = shutil.which(args.tool)
    home = str(Path.home())
    if found and found.startswith(home):
        known = "~" + found[len(home) :]
        if known not in known_paths:
            known_paths.append(known)

    spec: dict = {
        "priority": args.priority if args.priority is not None else 50,
        "detect_names": detect_names,
        "version_args": shlex.split(args.version_args or "--version"),
        "handles": args.handle or [],
        "commands": dict(args.command or []),
        "fallbacks": split_values(args.fallback),
    }
    if known_paths:
        spec["known_paths"] = known_paths
    app_paths = split_values(args.app_bundle_path)
    if app_paths:
        spec["app_bundle_paths"] = app_paths
    return spec


def retain_availability(records: list[dict]) -> None:
    for record in records:
        subprocess.run(
            [
                sys.executable,
                str(RETAIN),
                "--record-type",
                "availability",
                "--category",
                record["category"],
                "--tool",
                record["tool"],
                "--status",
                record["status"],
                "--path",
                record.get("path", ""),
                "--version",
                record.get("version", ""),
                "--confidence",
                str(record.get("confidence", 0.7)),
            ],
            check=False,
        )


def main() -> int:
    parser = argparse.ArgumentParser(description="Register a new candidate tool.")
    parser.add_argument("tool")
    parser.add_argument("--category")
    parser.add_argument("--priority", type=int)
    parser.add_argument("--detect-name", action="append", default=[])
    parser.add_argument("--known-path", action="append", default=[])
    parser.add_argument("--app-bundle-path", action="append", default=[])
    parser.add_argument("--version-args")
    parser.add_argument("--handle", action="append", default=[])
    parser.add_argument("--command", action="append", type=parse_command, default=[])
    parser.add_argument("--fallback", action="append", default=[])
    parser.add_argument("--description", default="")
    parser.add_argument("--interactive", action="store_true")
    parser.add_argument("--retain", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--no-detect", action="store_true")
    args = parser.parse_args()

    if args.interactive or not args.category:
        args = interactive_fill(args)

    registry = load_yaml(REGISTRY)
    registry.setdefault(args.category, {})
    if args.description:
        registry[args.category]["description"] = args.description
    else:
        registry[args.category].setdefault("description", "")
    registry[args.category].setdefault("tools", {})

    existed = args.tool in registry[args.category]["tools"]
    registry[args.category]["tools"][args.tool] = build_spec(args)

    if args.dry_run:
        print(json.dumps({args.category: {args.tool: registry[args.category]["tools"][args.tool]}}, ensure_ascii=False, indent=2))
        return 0

    dump_yaml(registry, REGISTRY)
    print(f"{'Updated' if existed else 'Registered'} {args.tool} in category {args.category}.")

    if args.no_detect:
        return 0

    detect = subprocess.run(
        [sys.executable, str(DETECT), "--tool", args.tool, "--json", "--write-cache"],
        capture_output=True,
        text=True,
    )
    print(detect.stdout.strip())
    if detect.returncode != 0:
        print(detect.stderr, file=sys.stderr)
        return detect.returncode
    if args.retain:
        retain_availability(json.loads(detect.stdout or "[]"))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"register-tool error: {exc}", file=sys.stderr)
        raise SystemExit(2)
