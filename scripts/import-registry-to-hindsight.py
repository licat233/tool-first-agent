#!/usr/bin/env python3
"""Import registry command templates to Hindsight as tool-inventory memories."""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
QUERY = ROOT / "scripts" / "query-registry.py"
RETAIN = ROOT / "scripts" / "retain-tool-memory.py"


def main() -> int:
    result = subprocess.run([sys.executable, str(QUERY), "--json"], capture_output=True, text=True)
    if result.returncode != 0:
        print(result.stderr or result.stdout, file=sys.stderr)
        return result.returncode
    rows = json.loads(result.stdout)
    imported = 0
    for row in rows:
        commands = row.get("commands", {}) or {}
        if not commands:
            continue
        for task, command in commands.items():
            cmd = [
                sys.executable,
                str(RETAIN),
                "--record-type",
                "recipe",
                "--category",
                row["category"],
                "--task",
                task,
                "--tool",
                row["tool"],
                "--command-template",
                command,
                "--status",
                "registry_candidate",
                "--confidence",
                "0.65",
            ]
            subprocess.run(cmd, check=False)
            imported += 1
            time.sleep(0.05)
    print(f"Imported {imported} registry command templates to Hindsight.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
