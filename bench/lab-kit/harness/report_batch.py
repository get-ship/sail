#!/usr/bin/env python3
"""Write batch SUMMARY.md from lab.sqlite for runs in a batch report dir."""
from __future__ import annotations

import json
import os
import sqlite3
import sys
from pathlib import Path

LAB = Path(os.environ.get("SAIL_LAB", Path.home() / "dev/sail-lab")).expanduser()


def main() -> None:
    if len(sys.argv) < 2:
        print("usage: report_batch.py <batch_report_dir>", file=sys.stderr)
        sys.exit(2)
    batch_dir = Path(sys.argv[1]).resolve()
    db = LAB / "lab.sqlite"
    if not db.exists():
        print(f"no db at {db}", file=sys.stderr)
        sys.exit(1)

    # Collect run_ids from one-pagers' companion harvest or from run dirs listed in batch
    run_dirs_file = batch_dir / "run_dirs.txt"
    run_dirs: list[str] = []
    if run_dirs_file.exists():
        run_dirs = [ln.strip() for ln in run_dirs_file.read_text().splitlines() if ln.strip()]

    conn = sqlite3.connect(str(db))
    conn.row_factory = sqlite3.Row
    rows = []
    if run_dirs:
        for rd in run_dirs:
            cur = conn.execute("SELECT * FROM runs WHERE run_dir = ?", (rd,))
            r = cur.fetchone()
            if r:
                rows.append(r)
    else:
        # all runs whose run_dir basename matches a report md
        mds = list(batch_dir.glob("*.md"))
        names = {p.stem for p in mds if p.name != "SUMMARY.md"}
        cur = conn.execute("SELECT * FROM runs")
        for r in cur.fetchall():
            if Path(r["run_dir"]).name in names:
                rows.append(r)
    conn.close()

    # Table: task × arm aggregates
    from collections import defaultdict
    groups: dict[tuple[str, str], list] = defaultdict(list)
    for r in rows:
        groups[(r["task_id"], r["arm"])].append(r)

    lines = [
        f"# Batch summary",
        "",
        f"batch_dir: `{batch_dir}`",
        f"runs: {len(rows)}",
        "",
        "## Measures (task × arm)",
        "",
        "| task | arm | n | compile_ok | vocab_share (avg) | turns (avg) | wall_ms (avg) | tokens_in | tokens_out | failures |",
        "|---|---|---:|---|---:|---:|---:|---|---|---|",
    ]

    for (task, arm), rs in sorted(groups.items()):
        n = len(rs)
        comp = []
        vocab = []
        turns = []
        walls = []
        tin_null = 0
        tout_null = 0
        fails = []
        for r in rs:
            if r["compile_ok"] is not None:
                comp.append(r["compile_ok"])
            if r["vocab_share"] is not None:
                vocab.append(r["vocab_share"])
            if r["turns"] is not None:
                turns.append(r["turns"])
            if r["wall_ms"] is not None:
                walls.append(r["wall_ms"])
            if r["tokens_in"] is None:
                tin_null += 1
            if r["tokens_out"] is None:
                tout_null += 1
            try:
                fails.extend(json.loads(r["failure_classes_json"] or "[]"))
            except json.JSONDecodeError:
                pass
        comp_s = f"{sum(comp)}/{len(comp)}" if comp else "n/a"
        vocab_s = f"{sum(vocab)/len(vocab):.2f}" if vocab else "n/a"
        turns_s = f"{sum(turns)/len(turns):.1f}" if turns else "n/a"
        wall_s = f"{sum(walls)/len(walls):.0f}" if walls else "n/a"
        tin_s = f"{tin_null}/{n} NULL" if tin_null else "present"
        tout_s = f"{tout_null}/{n} NULL" if tout_null else "present"
        fail_s = ",".join(sorted(set(fails))) or "—"
        lines.append(
            f"| {task} | {arm} | {n} | {comp_s} | {vocab_s} | {turns_s} | {wall_s} | {tin_s} | {tout_s} | {fail_s} |"
        )

    lines.extend(["", "## Per-task verdicts", ""])
    tasks = sorted({t for t, _ in groups})
    for task in tasks:
        sail = groups.get((task, "sail"), [])
        react = groups.get((task, "react-tw"), [])
        sail_ok = sum(1 for r in sail if r["compile_ok"] == 1)
        sail_n = len(sail)
        react_n = len(react)
        lines.append(
            f"- **{task}**: sail clean {sail_ok}/{sail_n}; react-tw runs {react_n}. "
            f"Beauty human-pending. Decide context-vs-language from one-pagers, not this line alone."
        )

    lines.extend([
        "",
        "## Notes",
        "",
        "- Missing tokens are NULL + flagged on one-pagers (never silent drop).",
        "- beauty_json is NULL until operator grades.",
        "- Screenshot L1 fidelity gap (RENDER-1) — beauty suspect for sail arm.",
        "",
    ])

    (batch_dir / "SUMMARY.md").write_text("\n".join(lines))
    print(f"wrote {batch_dir / 'SUMMARY.md'}")


if __name__ == "__main__":
    main()
