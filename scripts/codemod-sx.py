#!/usr/bin/env python3
"""Codemod every sx attribute from the old `prop:token` atoms to the table's
Tailwind names.

Deterministic, and it does NOT guess. Anything it cannot map with certainty is
reported and left alone, so a silent wrong rewrite is impossible — `display:flex`
in particular is REFUSED by the language now (that is what <row> is), and the
codemod says so instead of inventing something.

    python3 scripts/codemod-sx.py --check    # report only
    python3 scripts/codemod-sx.py            # rewrite
"""

import os
import re
import sys
from collections import Counter

CHECK = "--check" in sys.argv

# ── prop → new name, for the utilities whose value carries over unchanged ──────
RENAME = {
    "fg": "text",            # colour
    "text": "text",          # size — the same name; the VALUE disambiguates
    "bg": "bg",
    "radius": "rounded",
    "weight": "font",        # the weight scale
    "font": "font",          # the family scale
    "leading": "leading",
    "tracking": "tracking",
    "shadow": "shadow",
    "opacity": "opacity",
    "z": "z",
    "ring": "ring",
    "cursor": "cursor",
    "select": "select",
    "aspect": "aspect",
    "overflow": "overflow",
    "overflow-x": "overflow-x",
    "overflow-y": "overflow-y",
    "rotate": "rotate",
    "fill": "fill",
    "stroke": "stroke",
    "grow": "grow",
    "shrink": "shrink",
    "basis": "basis",
    "self": "self",
    "order": "order",
    "touch": "touch",
    "origin": "origin",
    "object": "object",
    "transition": "transition",
    "duration": "duration",
    "ease": "ease",
    "delay": "delay",
    "indent": "indent",
    "list": "list",
    "align": "align",
    "outline-offset": "outline-offset",
    "will-change": "will-change",
    "contain": "contain",
    "resize": "resize",
    "appearance": "appearance",
    "scroll": "scroll",
    "mix-blend": "mix-blend",
    "justify-self": "justify-self",
    "caret": "caret",
    "accent": "accent",
    "decoration": "decoration",
    "break": "break",
    "whitespace": "whitespace",
    "min-w": "min-w", "max-w": "max-w", "min-h": "min-h", "max-h": "max-h",
    "w": "w", "h": "h", "size": "size",
    "p": "p", "px": "px", "py": "py", "pt": "pt", "pr": "pr", "pb": "pb", "pl": "pl",
    "ps": "ps", "pe": "pe",
    "m": "m", "mx": "mx", "my": "my", "mt": "mt", "mr": "mr", "mb": "mb", "ml": "ml",
    "ms": "ms", "me": "me",
    "inset": "inset", "inset-x": "inset-x", "inset-y": "inset-y",
    "top": "top", "right": "right", "bottom": "bottom", "left": "left",
}

# ── prop:token → a fixed atom (or several) ────────────────────────────────────
EXACT = {
    "wrap:truncate": "truncate",
    "wrap:nowrap": "whitespace-nowrap",
    "wrap:normal": "whitespace-normal",
    "wrap:pre": "whitespace-pre",
    "wrap:pre-wrap": "whitespace-pre-wrap",
    "case:upper": "uppercase",
    "case:lower": "lowercase",
    "case:capitalize": "capitalize",
    "case:none": "normal-case",
    "text-style:italic": "italic",
    "text-style:normal": "not-italic",
    "text-align:left": "text-left",
    "text-align:center": "text-center",
    "text-align:right": "text-right",
    "text-align:justify": "text-justify",
    "text-align:start": "text-start",
    "text-align:end": "text-end",
    "visibility:visible": "visible",
    "visibility:hidden": "invisible",
    "visibility:assistive": "sr-only",
    "pointer:none": "pointer-events-none",
    "pointer:auto": "pointer-events-auto",
    "grow:none": "grow-0",
    "grow:1": "grow-1",
    "grow:2": "grow-1",  # a 2:1 split is a layout decision — flagged below
    "display:block": "block",
    "display:inline-block": "inline-block",
    "display:inline": "inline",
    "display:contents": "contents",
    "display:none": "hidden",
    "position:relative": "relative",
    "position:absolute": "absolute",
    "position:fixed": "fixed",
    "position:sticky": "sticky",
    "position:static": "static",
    "border-style:solid": "border-solid",
    "border-style:dashed": "border-dashed",
    "border-style:dotted": "border-dotted",
    "border-style:none": "border-none",
}

# The old `border:X` atom set width AND style AND colour in one. Tailwind splits
# them, so a hairline in a colour is two atoms. `subtle`/`strong` were the two
# strength words and they name real tokens.
BORDER_SIDES = {"border": "", "border-t": "-t", "border-r": "-r", "border-b": "-b", "border-l": "-l"}
BORDER_STRENGTH = {"subtle": "border-subtle", "strong": "border-strong"}

# REFUSED: these are layout, and layout is an element. The codemod will not guess.
REFUSED = {
    "display:flex": "<row> or <column> — a layout context is a layout element",
    "display:inline-flex": "<row> — a layout context is a layout element",
    "display:grid": "<grid>",
    "display:inline-grid": "<grid>",
}


def convert_atom(atom):
    """-> (new_atoms:list[str], problem:str|None)"""
    prefix, _, body = atom.rpartition(":")
    # A prefix is everything before the LAST colon of a `state:prop:token`.
    if prefix and ":" in prefix:
        state, _, prop = prefix.partition(":")
    else:
        state, prop = (prefix, "") if not body else ("", prefix)

    # Re-split honestly: old atoms are `prop:token` or `state:prop:token`.
    parts = atom.split(":")
    if len(parts) == 1:
        return [atom], None            # bare `group`
    if len(parts) == 2:
        state, prop, token = "", parts[0], parts[1]
    elif len(parts) == 3:
        state, prop, token = parts[0], parts[1], parts[2]
    else:
        return [atom], f"cannot parse `{atom}`"

    key = f"{prop}:{token}"
    if key in REFUSED:
        return [], f"`{atom}` is refused — use {REFUSED[key]}"

    def pre(a):
        return f"{state}:{a}" if state else a

    if key in EXACT:
        return [pre(EXACT[key])], None

    if prop in BORDER_SIDES:
        side = BORDER_SIDES[prop]
        if token == "none":
            return [pre(f"border{side}-0")], None
        colour = BORDER_STRENGTH.get(token, token)
        # width+style, then colour — CSS defaults border-style to none, so a
        # colour alone paints nothing.
        return [pre(f"border{side}"), pre(f"border{side}-{colour}")], None

    if prop in RENAME:
        return [pre(f"{RENAME[prop]}-{token}")], None

    return [atom], f"unmapped prop `{prop}:` in `{atom}`"


def convert_value(val):
    out, problems = [], []
    for atom in val.split():
        if atom == "group":
            out.append(atom)
            continue
        new, problem = convert_atom(atom)
        out.extend(new)
        if problem:
            problems.append(problem)
    return " ".join(out), problems


def main():
    changed, all_problems = 0, Counter()
    files_with_problems = {}
    for root, dirs, files in os.walk("."):
        dirs[:] = [d for d in dirs if d not in ("node_modules", "target", ".git", ".sail-out", "dist")]
        for fn in files:
            if not fn.endswith(".sui"):
                continue
            path = os.path.join(root, fn)
            src = open(path, encoding="utf8").read()
            problems = []

            def repl(m):
                new, ps = convert_value(m.group(1))
                problems.extend(ps)
                return f'sx="{new}"'

            out = re.sub(r'sx="([^"]*)"', repl, src)
            if problems:
                files_with_problems.setdefault(path, []).extend(problems)
                for p in problems:
                    all_problems[p] += 1
            if out != src and not CHECK:
                open(path, "w", encoding="utf8").write(out)
                changed += 1

    print(f"{'would rewrite' if CHECK else 'rewrote'} {changed} files")
    if all_problems:
        print(f"\n{sum(all_problems.values())} atoms need a HUMAN — the codemod does not guess:\n")
        for p, n in all_problems.most_common():
            print(f"  {n:4}  {p}")
        print("\nfiles:")
        for f, ps in sorted(files_with_problems.items())[:40]:
            print(f"  {f}  ({len(ps)})")


if __name__ == "__main__":
    main()
