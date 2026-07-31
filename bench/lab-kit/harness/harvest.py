#!/usr/bin/env python3
"""Harvest a completed sandbox into lab.sqlite + artifacts.

Usage:
  harvest.py <sandbox>
  harvest.py --replay <events.jsonl> --meta <meta.json> [--expected <expected.json>]

Missing tokens → NULL + flag (never silent drop).
Beauty stays NULL until human graded.
"""
from __future__ import annotations

import argparse
import base64
import collections
import hashlib
import json
import os
import re
import sqlite3
import subprocess
import sys
from pathlib import Path
from typing import Any

LAB = Path(os.environ.get("SAIL_LAB", Path.home() / "dev/sail-lab")).expanduser()
REPO = Path(os.environ.get("SAIL_REPO", Path.home() / "dev/Ship/sail")).expanduser()
HARNESS_VER = os.environ.get("HARNESS_VER", "eval2-0.1.0")

# Real L1 / layout inventory only — not raw HTML (those are neither side of the ratio).
L1_TAGS = {
    "column", "row", "grid", "center", "stack",
    "card", "button", "input", "label", "badge", "field", "navbar",
    "table", "tabs", "textarea", "dialog", "switch", "select",
    "checkbox", "radio", "slot",
}
PRIMITIVE_TAGS = {"div", "span"}
# Structural tags counted neither as L1 nor primitive (eval vocab ratio is L1 vs primitives).
OTHER_TAGS = {
    "h1", "h2", "h3", "h4", "h5", "h6", "p", "a", "ul", "ol", "li", "img",
    "form", "section", "header", "footer", "main", "nav", "aside", "article",
    "controller", "scope", "script", "style", "template",
}

FAILURE_TAXONOMY = {
    "invented-syntax",
    "missing-example",
    "toolchain-friction",
    "language-gap",
    "strictness-wall",
    "premature-victory",
}


def load_jsonc_ish(path: Path) -> dict[str, Any]:
    raw = path.read_text()
    lines = [ln for ln in raw.splitlines() if not ln.strip().startswith("//")]
    text = "\n".join(lines)
    text = re.sub(r",\s*}", "}", re.sub(r",\s*]", "]", text))
    return json.loads(text)


def ensure_db(db_path: Path) -> sqlite3.Connection:
    schema = LAB / "schema.sql"
    if not schema.exists():
        schema = REPO / "bench" / "schema.sql"
    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA foreign_keys = ON")
    if schema.exists():
        conn.executescript(schema.read_text())
    return conn


def parse_events(events_path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    if not events_path.exists():
        return rows
    for line in events_path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            rows.append({"ts": None, "event": {"raw": line, "hookEventName": "parse_error"}})
    return rows


def analyze_events(events: list[dict[str, Any]]) -> dict[str, Any]:
    tools: collections.Counter[str] = collections.Counter()
    hook_names: collections.Counter[str] = collections.Counter()
    check_timeline: list[dict[str, Any]] = []
    permission_denials = 0
    sail_cli_calls = 0
    mcp_calls = 0
    write_turns = 0
    compactions = 0
    first_ts = last_ts = None
    session_id = None
    transcript_path = None
    max_tokens = None

    for row in events:
        ts = row.get("ts")
        if ts:
            first_ts = first_ts or ts
            last_ts = ts
        ev = row.get("event") or {}
        name = (ev.get("hookEventName") or ev.get("hook_event_name") or "?").lower()
        hook_names[name] += 1
        session_id = session_id or ev.get("sessionId")
        transcript_path = transcript_path or ev.get("transcriptPath")

        # Compaction / context-collapse signals (hook name or payload breadcrumbs)
        ev_blob = json.dumps(ev).lower()
        if (
            "compact" in name
            or name in ("pre_compact", "post_compact", "precompact", "postcompact")
            or '"compaction"' in ev_blob
            or "context_compact" in ev_blob
            or "sessionupdate" in name and "compact" in ev_blob
        ):
            compactions += 1

        tn = ev.get("toolName") or ev.get("tool_name")
        # Count tools / checks on post_tool_use only (pre+post would double-count)
        is_post = name in ("post_tool_use", "posttooluse", "post_tool_use_failure", "posttoolusefailure")
        if tn and is_post:
            tools[tn] += 1
            inp = ev.get("toolInput") or ev.get("tool_input") or {}
            inp_s = json.dumps(inp)
            res = ev.get("toolResult") or ev.get("tool_result")
            res_s = json.dumps(res) if res is not None else ""

            if tn in ("write", "Write", "search_replace", "StrReplace", "Edit"):
                write_turns += 1

            if "sail" in tn.lower() or "mcp" in tn.lower():
                mcp_calls += 1
            if "sail" in inp_s and ("check" in inp_s or "op " in inp_s or ".sail-bin/sail" in inp_s):
                sail_cli_calls += 1

            if "permission" in res_s.lower() and "denied" in res_s.lower():
                permission_denials += 1
            if "permission denied" in res_s.lower():
                permission_denials += 1

            cmd = ""
            if isinstance(inp, dict):
                cmd = str(inp.get("command") or inp.get("cmd") or "")
            if "sail check" in cmd or (".sail-bin/sail" in cmd and "check" in cmd):
                ok = None
                if isinstance(res, dict):
                    out = str(res.get("stdout") or res.get("output") or res)
                    code = res.get("exitCode", res.get("exit_code"))
                    if code is not None:
                        ok = int(code) == 0
                    elif "ok" in out.lower() and "error" not in out.lower():
                        ok = True
                    elif "error" in out.lower():
                        ok = False
                check_timeline.append({
                    "ts": ts,
                    "command": cmd[:300],
                    "ok": ok,
                })

        # token breadcrumbs on events
        meta = ev.get("_meta") or {}
        for key in ("totalTokens", "total_tokens", "tokens"):
            if key in meta and isinstance(meta[key], (int, float)):
                max_tokens = max(max_tokens or 0, int(meta[key]))

    # turns ≈ authoring writes/edits; fall back to stop count
    stop_n = hook_names.get("stop", 0)
    turns = write_turns or tools.get("write", 0) + tools.get("search_replace", 0)
    if not turns and stop_n:
        turns = stop_n

    # ONLY first timeline entry with ok is True — never invent success from host compile
    iterations_to_clean = None
    attempt = 0
    for c in check_timeline:
        attempt += 1
        if c.get("ok") is True:
            iterations_to_clean = attempt
            break

    return {
        "tools": dict(tools),
        "hooks": dict(hook_names),
        "check_timeline": check_timeline,
        "permission_denials": permission_denials,
        "sail_cli_calls": sail_cli_calls,
        "mcp_calls": mcp_calls,
        "compactions": compactions,
        "turns": turns,
        "iterations_to_clean": iterations_to_clean,
        "first_ts": first_ts,
        "last_ts": last_ts,
        "session_id": session_id,
        "transcript_path": transcript_path,
        "event_token_hint": max_tokens,
        "event_count": len(events),
    }


def scrape_tokens(transcript_path: str | None, sandbox: Path) -> tuple[int | None, int | None, list[str]]:
    """Best-effort token capture. Returns (in, out, flags)."""
    flags: list[str] = []
    tokens_in = tokens_out = None

    candidates: list[Path] = []
    if transcript_path:
        candidates.append(Path(transcript_path))
    # grok session dirs
    sessions = Path.home() / ".grok" / "sessions"
    if sessions.exists():
        # find updates.jsonl mentioning sandbox
        for p in sessions.rglob("updates.jsonl"):
            try:
                if str(sandbox) in p.read_text(errors="ignore")[:2000] or sandbox.name in str(p):
                    candidates.append(p)
            except OSError:
                pass

    max_total = None
    for path in candidates:
        if not path.exists():
            continue
        try:
            for line in path.read_text(errors="ignore").splitlines():
                if "token" not in line.lower() and "usage" not in line.lower():
                    continue
                try:
                    d = json.loads(line)
                except json.JSONDecodeError:
                    continue
                blob = json.dumps(d)
                # _meta.totalTokens is cumulative context, not in/out split
                def walk(obj: Any) -> None:
                    nonlocal tokens_in, tokens_out, max_total
                    if isinstance(obj, dict):
                        for k, v in obj.items():
                            lk = k.lower()
                            if lk in ("input_tokens", "prompt_tokens", "tokens_in") and isinstance(v, (int, float)):
                                tokens_in = int(v)
                            if lk in ("output_tokens", "completion_tokens", "tokens_out") and isinstance(v, (int, float)):
                                tokens_out = int(v)
                            if lk in ("totaltokens", "total_tokens") and isinstance(v, (int, float)):
                                max_total = max(max_total or 0, int(v))
                            walk(v)
                    elif isinstance(obj, list):
                        for x in obj:
                            walk(x)
                walk(d)
        except OSError:
            continue

    if tokens_in is None and tokens_out is None:
        flags.append("missing_tokens")
    elif tokens_in is None:
        flags.append("missing_tokens_in")
    elif tokens_out is None:
        flags.append("missing_tokens_out")
    return tokens_in, tokens_out, flags


def collect_sail_files(root: Path) -> list[Path]:
    out: list[Path] = []
    root = root.resolve()
    for p in root.rglob("*.sail"):
        try:
            rel_parts = p.resolve().relative_to(root).parts
        except ValueError:
            continue
        # Skip only hidden / vendor dirs *inside* the sandbox, not e.g. lab `.tmp` parents
        if any(part.startswith(".") or part in ("node_modules", "target") for part in rel_parts[:-1]):
            continue
        out.append(p)
    return sorted(out)


def heuristic_vocab(files: list[Path]) -> float | None:
    """L1 share = L1 / (L1 + primitives). Unknown/raw HTML/invented tags count neither side."""
    if not files:
        return None
    l1 = prim = 0
    tag_re = re.compile(r"<\s*/?\s*([A-Za-z][\w-]*)")
    for f in files:
        try:
            text = f.read_text(errors="ignore")
        except OSError:
            continue
        for m in tag_re.finditer(text):
            tag = m.group(1).lower()
            if tag in PRIMITIVE_TAGS:
                prim += 1
            elif tag in L1_TAGS:
                l1 += 1
            # OTHER_TAGS and unknown/invented tags: neither side of the ratio
    denom = l1 + prim
    if denom == 0:
        return 0.0
    return l1 / denom


def run_sail_check(sandbox: Path, sail_bin: Path) -> tuple[bool | None, list[dict[str, Any]], int]:
    files = collect_sail_files(sandbox)
    if not files or not sail_bin.exists():
        return None, [], 0
    diagnostics: list[dict[str, Any]] = []
    ok_all = True
    for f in files:
        rel = str(f.relative_to(sandbox))
        try:
            r = subprocess.run(
                [str(sail_bin), "check", rel],
                cwd=str(sandbox),
                capture_output=True,
                text=True,
                timeout=120,
            )
        except (OSError, subprocess.TimeoutExpired) as e:
            ok_all = False
            diagnostics.append({"path": rel, "error": str(e)})
            continue
        if r.returncode != 0:
            ok_all = False
        diagnostics.append({
            "path": rel,
            "exit": r.returncode,
            "stdout": (r.stdout or "")[:2000],
            "stderr": (r.stderr or "")[:1000],
        })
    return ok_all, diagnostics, len(files)


def run_layout(sandbox: Path, sail_bin: Path) -> list[dict[str, Any]]:
    findings: list[dict[str, Any]] = []
    if not sail_bin.exists():
        return findings
    for f in collect_sail_files(sandbox):
        rel = str(f.relative_to(sandbox))
        try:
            r = subprocess.run(
                [str(sail_bin), "op", "SailAuthoring/LayoutCheck", "--json", json.dumps({"path": rel})],
                cwd=str(sandbox),
                capture_output=True,
                text=True,
                timeout=60,
            )
            text = (r.stdout or "") + (r.stderr or "")
            # try parse JSON findings
            try:
                data = json.loads(text[text.find("{"):] if "{" in text else text)
                if isinstance(data, dict):
                    fl = data.get("findings") or data.get("layoutFindings") or []
                    if isinstance(fl, list):
                        for item in fl:
                            if isinstance(item, dict):
                                item = {**item, "path": rel}
                            findings.append(item if isinstance(item, dict) else {"path": rel, "raw": item})
            except Exception:
                if r.returncode != 0 and text.strip():
                    findings.append({"path": rel, "note": text[:500]})
        except (OSError, subprocess.TimeoutExpired):
            pass
    return findings


def screenshot_sail(sandbox: Path, sail_bin: Path) -> dict[str, Any]:
    """Host-side SailRender/Screenshot. Prefer host sail (outside sandbox)."""
    result: dict[str, Any] = {"ok": False, "fidelity_note": "L1 paint may be incomplete (RENDER-1)"}
    host_sail = REPO / "target" / "release" / "sail"
    bin_path = host_sail if host_sail.exists() else sail_bin
    page = sandbox / "routes" / "page.sail"
    if not page.exists():
        sails = collect_sail_files(sandbox)
        if not sails:
            result["error"] = "no .sail files for screenshot"
            return result
        page = sails[0]
    rel = str(page.relative_to(sandbox))
    try:
        r = subprocess.run(
            [str(bin_path), "op", "SailRender/Screenshot", "--json", json.dumps({"path": rel})],
            cwd=str(sandbox),
            capture_output=True,
            text=True,
            timeout=60,
        )
        out = r.stdout or ""
        (sandbox / ".probe-shot.json").write_text(out)
        data = None
        try:
            data = json.loads(out)
        except json.JSONDecodeError:
            if "{" in out:
                data = json.loads(out[out.find("{"):])
        if not data:
            result["error"] = (r.stderr or out)[:500]
            return result
        b64 = data.get("imagePng") or data.get("image_b64") or data.get("image") or ""
        if b64:
            (sandbox / "screenshot.png").write_bytes(base64.b64decode(b64))
            result["ok"] = True
            result["png"] = "screenshot.png"
        result["regions"] = data.get("regions")
        result["dynamics"] = data.get("dynamics")
        result["viewport"] = data.get("viewport")
        result["ms"] = data.get("ms")
        result["theme"] = data.get("theme")
    except (OSError, subprocess.TimeoutExpired, json.JSONDecodeError) as e:
        result["error"] = str(e)
    return result


def _event_lines(sandbox: Path) -> list[str]:
    path = sandbox / ".probe-events.jsonl"
    if not path.exists():
        return []
    return path.read_text(errors="ignore").splitlines()


def _line_has_mcp_failure(line: str) -> bool:
    """True only when a single event co-locates MCP with a real failure signal.

    Avoids independent whole-log ANDs of 'mcp' + 'fail' (skill prose / help text
    routinely mention MCP without any failure).
    """
    try:
        row = json.loads(line)
    except (json.JSONDecodeError, TypeError):
        return False
    ev = row.get("event") or row
    if not isinstance(ev, dict):
        return False

    name = str(ev.get("hookEventName") or ev.get("hook_event_name") or "").lower()
    tn = str(ev.get("toolName") or ev.get("tool_name") or "").lower()
    inp = ev.get("toolInput") or ev.get("tool_input") or {}
    res = ev.get("toolResult") or ev.get("tool_result")
    # Narrow MCP-related surface: mcp tool names, sail-mcp binary, or explicit MCP server config
    mcpish = (
        "mcp" in tn
        or "sail-mcp" in tn
        or (isinstance(inp, dict) and "mcp" in json.dumps(inp).lower() and (
            "sail-mcp" in json.dumps(inp).lower()
            or "mcpserver" in json.dumps(inp).lower().replace("_", "")
            or "mcpServers" in json.dumps(inp)
        ))
    )
    # Also treat post_tool_use_failure of a clearly MCP-named tool
    if "failure" in name and ("mcp" in tn or "sail-mcp" in tn):
        return True

    res_s = json.dumps(res).lower() if res is not None else ""
    inp_s = json.dumps(inp).lower() if inp is not None else ""
    combined = f"{tn} {inp_s} {res_s}"

    if not mcpish and "sail-mcp" not in combined and "mcp server" not in combined:
        return False

    # Real failure phrases only — NOT bare "mcp server" (help/skill text)
    failure_markers = (
        "permission denied",
        "eacces",
        "enoent",
        "failed to start",
        "failed to connect",
        "failed to spawn",
        "spawn failed",
        "connection refused",
        "cannot execute",
        "exec format error",
        "no such file",
        "command not found",
        "server failed",
        "mcp error",
        "tool_result_error",
    )
    if any(m in res_s for m in failure_markers):
        # require mcp signal in tool name, input, or result (same event)
        if "mcp" in combined or "sail-mcp" in combined:
            return True
    # Structured non-zero exit on an mcp-related shell command
    if isinstance(res, dict):
        code = res.get("exitCode", res.get("exit_code"))
        cmd = str(inp.get("command") if isinstance(inp, dict) else "")
        if code not in (None, 0) and ("sail-mcp" in cmd or "mcp" in tn):
            return True
    return False


def classify_failures(
    compile_ok: bool | None,
    check_timeline: list[dict[str, Any]],
    diagnostics: list[dict[str, Any]],
    events_summary: dict[str, Any],
    sandbox: Path,
) -> list[str]:
    classes: list[str] = []
    diag_text = json.dumps(diagnostics).lower()
    lines = _event_lines(sandbox)
    event_text = "\n".join(lines).lower()

    if compile_ok is False:
        if "unknown" in diag_text or "unexpected" in diag_text or "expected" in diag_text:
            classes.append("invented-syntax")
        if "not allowed" in diag_text or "off-list" in diag_text or "not in" in diag_text:
            classes.append("strictness-wall")
        if not classes:
            classes.append("language-gap")

    # Permission denials: structured counter preferred; whole-log only for exact phrase
    if events_summary.get("permission_denials", 0) > 0:
        classes.append("toolchain-friction")
    elif any("permission denied" in ln.lower() for ln in lines):
        classes.append("toolchain-friction")

    # MCP/toolchain failure: same-event only (never independent full-file "mcp" AND "fail")
    if any(_line_has_mcp_failure(ln) for ln in lines):
        classes.append("toolchain-friction")

    # premature victory: declared stop without clean check
    if check_timeline and all(c.get("ok") is not True for c in check_timeline):
        if events_summary.get("hooks", {}).get("stop", 0) > 0 and compile_ok is False:
            classes.append("premature-victory")
    if not check_timeline and compile_ok is False:
        classes.append("premature-victory")

    # missing-example: only when compile failed AND an event mentions searching for examples
    if compile_ok is False:
        for ln in lines:
            low = ln.lower()
            if "example" in low and any(
                k in low for k in ("grep", "search", "glob", "find", "missing")
            ):
                classes.append("missing-example")
                break

    # unique preserve order
    seen = set()
    out = []
    for c in classes:
        if c in FAILURE_TAXONOMY and c not in seen:
            seen.add(c)
            out.append(c)
    return out


def make_run_id(meta: dict[str, Any], sandbox: Path) -> str:
    base = f"{meta.get('task_id')}-{meta.get('arm')}-{meta.get('n')}-{sandbox.name}"
    return hashlib.sha1(base.encode()).hexdigest()[:16]


def harvest_sandbox(sandbox: Path, db_path: Path | None = None) -> dict[str, Any]:
    sandbox = sandbox.resolve()
    meta_path = sandbox / ".lab-meta.json"
    if not meta_path.exists():
        raise SystemExit(f"missing .lab-meta.json in {sandbox}")
    meta = json.loads(meta_path.read_text())
    arm = meta.get("arm", "sail")
    flags: list[str] = []

    skill_source = str(meta.get("skill_source") or "")
    if arm == "sail":
        if skill_source.startswith("generated:"):
            pass
        elif skill_source == "probe-fallback":
            flags.append("skill_not_generated")
            flags.append("skill_probe_fallback")
        elif skill_source == "stub":
            flags.append("skill_not_generated")
            flags.append("skill_stub")
        elif skill_source in ("", "n/a"):
            flags.append("skill_source_unknown")
        else:
            flags.append(f"skill_source:{skill_source}")

    events = parse_events(sandbox / ".probe-events.jsonl")
    if not events:
        flags.append("hook_gap_no_events")
    summary = analyze_events(events)

    tokens_in, tokens_out, token_flags = scrape_tokens(summary.get("transcript_path"), sandbox)
    flags.extend(token_flags)

    sail_bin = sandbox / ".sail-bin" / "sail"
    compile_ok: bool | None = None
    vocab: float | None = None
    layout_findings: list[dict[str, Any]] = []
    diagnostics: list[dict[str, Any]] = []
    shot: dict[str, Any] = {}

    if arm == "sail":
        compile_ok, diagnostics, _n = run_sail_check(sandbox, sail_bin)
        vocab = heuristic_vocab(collect_sail_files(sandbox))
        layout_findings = run_layout(sandbox, sail_bin)
        shot = screenshot_sail(sandbox, sail_bin)
        if not shot.get("ok"):
            flags.append("screenshot_failed")
        flags.append("screenshot_l1_fidelity_gap")  # known RENDER-1
    else:
        flags.append("react_screenshot_asymmetric")
        shot = {"ok": False, "note": "react arm: use browser screenshot offline; render asymmetry is real"}

    failure_classes = classify_failures(
        compile_ok, summary["check_timeline"], diagnostics, summary, sandbox
    )

    # iterations ONLY from timeline ok=True; host compile clean without that → NULL + flag
    iterations = summary.get("iterations_to_clean")
    if iterations is None:
        if summary.get("check_timeline"):
            if compile_ok is True:
                flags.append("iterations_host_only")
            else:
                flags.append("iterations_unknown")
        else:
            flags.append("iterations_unknown")

    run_id = meta.get("run_id") or make_run_id(meta, sandbox)
    wall_ms = meta.get("wall_ms")
    from datetime import datetime, timezone
    ts = meta.get("started_at") or summary.get("first_ts") or ""
    if not ts:
        ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        flags.append("ts_defaulted")

    row = {
        "run_id": run_id,
        "ts": ts,
        "task_id": meta.get("task_id"),
        "arm": arm,
        "model": meta.get("model", "grok"),
        "toolchain": meta.get("toolchain"),
        "profile_hash": meta.get("profile_hash"),
        "harness_ver": meta.get("harness_ver", HARNESS_VER),
        "wall_ms": wall_ms,
        "turns": summary.get("turns"),
        "tokens_in": tokens_in,
        "tokens_out": tokens_out,
        "compile_ok": None if compile_ok is None else (1 if compile_ok else 0),
        "iterations_to_clean": iterations,
        "vocab_share": vocab,
        "layout_findings_json": json.dumps(layout_findings),
        "failure_classes_json": json.dumps(failure_classes),
        "beauty_json": None,  # human graded later
        "run_dir": str(sandbox),
    }

    harvest_blob = {
        "row": row,
        "flags": flags,
        "skill_source": skill_source,
        "events_summary": {
            k: summary[k]
            for k in (
                "tools", "hooks", "check_timeline", "permission_denials",
                "sail_cli_calls", "mcp_calls", "compactions", "event_count", "session_id",
            )
            if k in summary
        },
        "diagnostics": diagnostics,
        "screenshot": shot,
        "prompt": (sandbox / ".lab-prompt.txt").read_text() if (sandbox / ".lab-prompt.txt").exists() else "",
    }
    (sandbox / ".lab-harvest.json").write_text(json.dumps(harvest_blob, indent=2) + "\n")

    # Agent final sources (git diff)
    try:
        diff = subprocess.run(
            ["git", "-C", str(sandbox), "diff", "HEAD"],
            capture_output=True, text=True, timeout=30,
        )
        (sandbox / ".lab-diff.patch").write_text(diff.stdout or "")
    except (OSError, subprocess.TimeoutExpired):
        pass

    db_path = db_path or (LAB / "lab.sqlite")
    conn = ensure_db(db_path)
    conn.execute(
        """
        INSERT OR REPLACE INTO runs (
          run_id, ts, task_id, arm, model, toolchain, profile_hash, harness_ver,
          wall_ms, turns, tokens_in, tokens_out, compile_ok, iterations_to_clean,
          vocab_share, layout_findings_json, failure_classes_json, beauty_json, run_dir
        ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
        """,
        (
            row["run_id"], row["ts"], row["task_id"], row["arm"], row["model"],
            row["toolchain"], row["profile_hash"], row["harness_ver"],
            row["wall_ms"], row["turns"], row["tokens_in"], row["tokens_out"],
            row["compile_ok"], row["iterations_to_clean"], row["vocab_share"],
            row["layout_findings_json"], row["failure_classes_json"],
            row["beauty_json"], row["run_dir"],
        ),
    )
    conn.execute("DELETE FROM run_events WHERE run_id = ?", (run_id,))
    for seq, ev in enumerate(events):
        conn.execute(
            "INSERT INTO run_events (run_id, seq, event_json) VALUES (?,?,?)",
            (run_id, seq, json.dumps(ev)),
        )
    conn.commit()
    conn.close()

    # Report one-pager if BATCH_DIR set, else under reports/adhoc
    batch_dir = os.environ.get("SAIL_LAB_BATCH_DIR")
    if batch_dir:
        report_dir = Path(batch_dir)
    else:
        report_dir = LAB / "reports" / "adhoc"
    report_dir.mkdir(parents=True, exist_ok=True)
    write_one_pager(report_dir / f"{sandbox.name}.md", harvest_blob, sandbox)

    print(json.dumps({"run_id": run_id, "flags": flags, "compile_ok": compile_ok, "report": str(report_dir / f"{sandbox.name}.md")}))
    return harvest_blob


def _token_cell(val: Any, flags: list[str], side: str) -> str:
    if val is not None:
        return str(val)
    if "missing_tokens" in flags or f"missing_tokens_{side}" in flags:
        return f"**NULL (FLAG missing_tokens_{side if side else 'both'})**"
    return "**NULL (FLAG)**"


def write_one_pager(path: Path, harvest: dict[str, Any], sandbox: Path) -> None:
    row = harvest["row"]
    flags = harvest.get("flags") or []
    es = harvest.get("events_summary") or {}
    prompt = harvest.get("prompt") or ""
    shot = harvest.get("screenshot") or {}
    skill_source = harvest.get("skill_source") or ""

    # Final source collapsible
    source_blocks = []
    for f in collect_sail_files(sandbox)[:8]:
        try:
            body = f.read_text(errors="ignore")
        except OSError:
            continue
        rel = f.relative_to(sandbox)
        source_blocks.append(f"<details><summary>{rel}</summary>\n\n```sail\n{body}\n```\n</details>\n")
    app_jsx = sandbox / "src" / "App.jsx"
    if app_jsx.exists():
        source_blocks.append(
            f"<details><summary>src/App.jsx</summary>\n\n```jsx\n{app_jsx.read_text(errors='ignore')}\n```\n</details>\n"
        )

    flag_lines = "\n".join(f"- **FLAG:** `{f}`" for f in flags) if flags else "- (none)"
    check_lines = es.get("check_timeline") or []
    check_md = "\n".join(
        f"- `{c.get('ts')}` ok={c.get('ok')} `{c.get('command', '')[:120]}`" for c in check_lines
    ) or "- (no check invocations logged)"

    shot_md = "_no screenshot_"
    if (sandbox / "screenshot.png").exists():
        shot_md = f"![screenshot]({sandbox / 'screenshot.png'})\n\n_KNOWN: L1 paint fidelity gap (RENDER-1). Beauty suspect._"
    elif shot.get("note"):
        shot_md = f"_{shot.get('note')}_"

    beauty = row.get("beauty_json")
    beauty_md = "_NULL — human grade pending_" if beauty is None else f"```json\n{beauty}\n```"
    tin = _token_cell(row.get("tokens_in"), flags, "in")
    tout = _token_cell(row.get("tokens_out"), flags, "out")

    md = f"""# Run {sandbox.name}

| field | value |
|---|---|
| task | `{row.get('task_id')}` |
| arm | `{row.get('arm')}` |
| model | `{row.get('model')}` |
| toolchain | `{row.get('toolchain')}` |
| profile_hash | `{row.get('profile_hash')}` |
| skill_source | `{skill_source}` |
| harness | `{row.get('harness_ver')}` |
| run_id | `{row.get('run_id')}` |
| run_dir | `{row.get('run_dir')}` |

## Prompt

{prompt}

## Numbers

| measure | value |
|---|---|
| wall_ms | {row.get('wall_ms')} |
| turns | {row.get('turns')} |
| tokens_in | {tin} |
| tokens_out | {tout} |
| compile_ok | {row.get('compile_ok')} |
| iterations_to_clean | {row.get('iterations_to_clean')} |
| vocab_share | {row.get('vocab_share')} |
| compactions | {es.get('compactions', 0)} |

## Check timeline

{check_md}

## Tool breakdown

```json
{json.dumps(es.get('tools') or {}, indent=2)}
```

sail_cli_calls={es.get('sail_cli_calls')} mcp_calls={es.get('mcp_calls')} permission_denials={es.get('permission_denials')} compactions={es.get('compactions', 0)}

## Failure classes

```json
{row.get('failure_classes_json')}
```

## Anomalies / flags

{flag_lines}

## Screenshot

{shot_md}

## Beauty (human)

{beauty_md}

## Final source

{''.join(source_blocks) if source_blocks else '_no sources found_'}
"""
    path.write_text(md)


def replay(events_path: Path, meta_path: Path, expected_path: Path | None, db_path: Path) -> int:
    """Build a synthetic sandbox from canned trajectory and harvest."""
    import shutil
    import tempfile

    meta = json.loads(meta_path.read_text())
    events_text = events_path.read_text()

    # Prefer lab disk over /tmp (tmpfs fills quickly with binary copies)
    tmp_root = LAB / ".tmp"
    tmp_root.mkdir(parents=True, exist_ok=True)
    tmp = Path(tempfile.mkdtemp(prefix="sail-lab-replay-", dir=str(tmp_root)))

    def link_or_skip(src: Path, dst: Path) -> None:
        """Hardlink preferred; never full-copy multi‑MB bins into temp (disk/tmpfs)."""
        dst.parent.mkdir(parents=True, exist_ok=True)
        try:
            if dst.exists():
                dst.unlink()
            os.link(src, dst)
        except OSError:
            # Fall back to absolute symlink (same machine)
            try:
                if dst.exists() or dst.is_symlink():
                    dst.unlink()
                os.symlink(src, dst)
            except OSError as e:
                raise SystemExit(f"cannot link sail bin for replay: {e}") from e

    try:
        (tmp / ".lab-meta.json").write_text(json.dumps(meta, indent=2) + "\n")
        (tmp / ".lab-prompt.txt").write_text(meta.get("prompt", "canned prompt") + "\n")
        (tmp / ".probe-events.jsonl").write_text(events_text)
        routes = tmp / "routes"
        routes.mkdir(parents=True)
        page = meta.get("source_page") or "<column><p>canned</p></column>\n"
        (routes / "page.sail").write_text(page)
        (tmp / "sail.config.jsonc").write_text(
            '{\n  "theme": { "default": "ship", "defaultMode": "light", "enabled": ["ship"] },\n  "render": "csr"\n}\n'
        )
        tc = list((LAB / "toolchains").glob("*/bin/sail"))
        release = REPO / "target" / "release" / "sail"
        src_bin = tc[0] if tc else (release if release.exists() else None)
        if src_bin is not None:
            link_or_skip(src_bin, tmp / ".sail-bin" / "sail")

        # force run_id from meta
        if "run_id" in meta:
            pass
        harvest = harvest_sandbox(tmp, db_path=db_path)
        row = harvest["row"]

        if expected_path and expected_path.exists():
            expected = json.loads(expected_path.read_text())
            errs = []
            for k, v in expected.items():
                if k == "tool_counts":
                    tools = (harvest.get("events_summary") or {}).get("tools") or {}
                    for tk, tv in v.items():
                        if tools.get(tk) != tv:
                            errs.append(f"tool_counts[{tk}]: got {tools.get(tk)} want {tv}")
                    continue
                if k == "check_invocations":
                    n = len((harvest.get("events_summary") or {}).get("check_timeline") or [])
                    if n != v:
                        errs.append(f"check_invocations: got {n} want {v}")
                    continue
                if k == "event_count":
                    n = (harvest.get("events_summary") or {}).get("event_count")
                    if n != v:
                        errs.append(f"event_count: got {n} want {v}")
                    continue
                if k == "flags":
                    got = set(harvest.get("flags") or [])
                    for f in v:
                        if f not in got:
                            errs.append(f"missing flag {f} in {got}")
                    continue
                if k == "turns":
                    # turns heuristic may vary; allow >= expected writes
                    if (row.get("turns") or 0) < v and row.get("turns") != v:
                        # exact preferred
                        if row.get("turns") != v:
                            errs.append(f"turns: got {row.get('turns')} want {v}")
                    continue
                if row.get(k) != v:
                    errs.append(f"{k}: got {row.get(k)!r} want {v!r}")
            if errs:
                print("REPLAY FAIL:")
                for e in errs:
                    print(" ", e)
                return 1
            print("REPLAY OK", row.get("run_id"))
            return 0
        print(json.dumps(row, indent=2))
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("sandbox", nargs="?", help="sandbox directory to harvest")
    ap.add_argument("--replay", type=Path, help="events.jsonl for canned replay")
    ap.add_argument("--meta", type=Path, help="meta.json for canned replay")
    ap.add_argument("--expected", type=Path, help="expected.json assertions")
    ap.add_argument("--db", type=Path, default=None)
    args = ap.parse_args()

    db = args.db or (LAB / "lab.sqlite")

    if args.replay:
        if not args.meta:
            print("--meta required with --replay", file=sys.stderr)
            sys.exit(2)
        sys.exit(replay(args.replay, args.meta, args.expected, db))

    if not args.sandbox:
        ap.error("sandbox or --replay required")
    harvest_sandbox(Path(args.sandbox), db_path=db)


if __name__ == "__main__":
    main()
