#!/usr/bin/env python3
"""Migrate brace control-flow to element control-flow in .sail sources.

  {#if c}…{:else if d}…{:else}…{/if}
  → <if cond={c}>…<else-if cond={d}>…</else-if><else>…</else></if>

Not a full HTML parser — works on common monorepo patterns. Review diffs.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

# Markers (brace form)
RE_OPEN_IF = re.compile(r"\{#if\s+((?:[^{}]|\{[^{}]*\})*)\}")
RE_ELSE_IF = re.compile(r"\{:else\s+if\s+((?:[^{}]|\{[^{}]*\})*)\}")
RE_ELSE = re.compile(r"\{:else\}")
RE_CLOSE_IF = re.compile(r"\{/if\}")

RE_OPEN_FOR = re.compile(
    r"\{#for\s+(.+?)\s+of\s+(.+?)(?:\s+key=\{([^}]*)\})?\s*\}"
)
RE_CLOSE_FOR = re.compile(r"\{/for\}")

RE_OPEN_MATCH = re.compile(r"\{#match\s+((?:[^{}]|\{[^{}]*\})*)\}")
RE_CASE = re.compile(r"\{:case\s+((?:[^{}]|\{[^{}]*\})*)\}")
RE_DEFAULT = re.compile(r"\{:default\}")
RE_CLOSE_MATCH = re.compile(r"\{/match\}")

RE_WL = re.compile(r"\{#while-loading\s+of\s+(\w+)\s*\}")
RE_CLOSE_WL = re.compile(r"\{/while-loading\}")
RE_WE = re.compile(r"\{#while-error\s+(\w+)\s+of\s+(\w+)\s*\}")
RE_CLOSE_WE = re.compile(r"\{/while-error\}")


def migrate_if_region(s: str) -> str:
    """Close open else-if/else when the next clause or close appears."""
    # Multi-pass token replace with stack for if clauses
    out: list[str] = []
    i = 0
    # stack of pending open clause closers to insert before next clause: list[str]
    pending_close: list[str] = []

    def flush_pending():
        while pending_close:
            out.append(pending_close.pop())

    while i < len(s):
        m = RE_OPEN_IF.match(s, i)
        if m:
            flush_pending()
            cond = m.group(1).strip()
            out.append(f"<if cond={{{cond}}}>")
            pending_close.append("</if>")  # wrong - if close is only at end
            # Use a different approach - line based with stack
            i = m.end()
            continue
        # fallback: simpler global regex pipeline for non-nested first
        break
    else:
        return "".join(out)
    return migrate_simple(s)


def migrate_simple(s: str) -> str:
    """Stack-based scanner for if/for/match/while."""
    # Find all markers with positions
    markers: list[tuple[int, int, str, tuple]] = []

    def add_all(rx: re.Pattern, kind: str):
        for m in rx.finditer(s):
            markers.append((m.start(), m.end(), kind, m.groups()))

    add_all(RE_OPEN_IF, "open_if")
    add_all(RE_ELSE_IF, "else_if")
    add_all(RE_ELSE, "else")
    add_all(RE_CLOSE_IF, "close_if")
    add_all(RE_OPEN_FOR, "open_for")
    add_all(RE_CLOSE_FOR, "close_for")
    add_all(RE_OPEN_MATCH, "open_match")
    add_all(RE_CASE, "case")
    add_all(RE_DEFAULT, "default")
    add_all(RE_CLOSE_MATCH, "close_match")
    add_all(RE_WL, "open_wl")
    add_all(RE_CLOSE_WL, "close_wl")
    add_all(RE_WE, "open_we")
    add_all(RE_CLOSE_WE, "close_we")
    markers.sort(key=lambda t: t[0])

    if not markers:
        return s

    # if-stack: each frame tracks open clause tag that needs </else-if> etc.
    # clause_open: None | "else-if" | "else"
    parts: list[str] = []
    pos = 0
    # stack of dicts for nested if/for/match
    stack: list[dict] = []

    def close_clause(frame: dict):
        c = frame.get("clause")
        if c == "else-if":
            parts.append("</else-if>")
        elif c == "else":
            parts.append("</else>")
        elif c == "case":
            parts.append("</case>")
        elif c == "default":
            parts.append("</default>")
        frame["clause"] = None

    for start, end, kind, groups in markers:
        parts.append(s[pos:start])
        if kind == "open_if":
            stack.append({"kind": "if", "clause": None})
            parts.append(f"<if cond={{{groups[0].strip()}}}>")
        elif kind == "else_if":
            if stack and stack[-1]["kind"] == "if":
                close_clause(stack[-1])
                stack[-1]["clause"] = "else-if"
            parts.append(f"<else-if cond={{{groups[0].strip()}}}>")
        elif kind == "else":
            if stack:
                top = stack[-1]
                if top["kind"] in ("if", "for"):
                    close_clause(top)
                    top["clause"] = "else"
            parts.append("<else>")
        elif kind == "close_if":
            if stack and stack[-1]["kind"] == "if":
                close_clause(stack[-1])
                stack.pop()
            parts.append("</if>")
        elif kind == "open_for":
            left, iterable, key = groups[0].strip(), groups[1].strip(), groups[2]
            # left is `item` or `(item, i)`
            as_name, index = left, None
            if left.startswith("(") and left.endswith(")"):
                bits = [b.strip() for b in left[1:-1].split(",")]
                as_name = bits[0]
                index = bits[1] if len(bits) > 1 else None
            attrs = [f'as="{as_name}"', f"of={{{iterable}}}"]
            if index:
                attrs.insert(1, f'index="{index}"')
            if key is not None:
                attrs.append(f"key={{{key.strip()}}}")
            stack.append({"kind": "for", "clause": None})
            parts.append(f"<for {' '.join(attrs)}>")
        elif kind == "close_for":
            if stack and stack[-1]["kind"] == "for":
                close_clause(stack[-1])
                stack.pop()
            parts.append("</for>")
        elif kind == "open_match":
            stack.append({"kind": "match", "clause": None})
            parts.append(f"<match on={{{groups[0].strip()}}}>")
        elif kind == "case":
            if stack and stack[-1]["kind"] == "match":
                close_clause(stack[-1])
                stack[-1]["clause"] = "case"
            parts.append(f"<case of={{{groups[0].strip()}}}>")
        elif kind == "default":
            if stack and stack[-1]["kind"] == "match":
                close_clause(stack[-1])
                stack[-1]["clause"] = "default"
            parts.append("<default>")
        elif kind == "close_match":
            if stack and stack[-1]["kind"] == "match":
                close_clause(stack[-1])
                stack.pop()
            parts.append("</match>")
        elif kind == "open_wl":
            parts.append(f'<while-loading of="{groups[0]}">')
        elif kind == "close_wl":
            parts.append("</while-loading>")
        elif kind == "open_we":
            parts.append(f'<while-error as="{groups[0]}" of="{groups[1]}">')
        elif kind == "close_we":
            parts.append("</while-error>")
        pos = end

    parts.append(s[pos:])
    return "".join(parts)


def main() -> int:
    roots = [Path("apps"), Path("packages"), Path("crates")]
    n = 0
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*.sail"):
            text = path.read_text(encoding="utf-8")
            if "{#" not in text and "{:" not in text and "{/" not in text:
                continue
            # skip if only {expr} not blocks — still run if has # or :
            if not re.search(r"\{[#/:]|\{:else", text):
                continue
            new = migrate_simple(text)
            if new != text:
                path.write_text(new, encoding="utf-8")
                n += 1
                print(path)
    print(f"updated {n} files", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
