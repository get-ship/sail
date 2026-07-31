#!/usr/bin/env python3
"""Canned trajectory replay test — recorded events → expected run row."""
from __future__ import annotations

import os
import sys
from pathlib import Path

LAB = Path(os.environ.get("SAIL_LAB", Path.home() / "dev/sail-lab")).expanduser()
REPO = Path(os.environ.get("SAIL_REPO", Path.home() / "dev/Ship/sail")).expanduser()

# Prefer lab fixtures (synced); fall back to repo bench fixtures
CANDIDATES = [
    LAB / "fixtures" / "canned-trajectory",
    REPO / "bench" / "fixtures" / "canned-trajectory",
]

def main() -> int:
    fixture = None
    for c in CANDIDATES:
        if (c / "events.jsonl").exists() and (c / "meta.json").exists():
            fixture = c
            break
    if not fixture:
        print("no canned-trajectory fixture found", file=sys.stderr)
        return 2

    harvest = LAB / "harness" / "harvest.py"
    if not harvest.exists():
        print(f"missing {harvest}", file=sys.stderr)
        return 2

    # import harvest module by path
    sys.path.insert(0, str(LAB / "harness"))
    import harvest as H  # type: ignore

    events = fixture / "events.jsonl"
    meta = fixture / "meta.json"
    expected = fixture / "expected.json"
    db = LAB / "lab.sqlite"
    # ensure schema
    if not db.exists() and (LAB / "schema.sql").exists():
        import sqlite3
        conn = sqlite3.connect(str(db))
        conn.executescript((LAB / "schema.sql").read_text())
        conn.close()

    return H.replay(events, meta, expected if expected.exists() else None, db)


if __name__ == "__main__":
    raise SystemExit(main())
