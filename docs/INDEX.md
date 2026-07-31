# Docs index

Every file under `docs/` is linked from this page. Specs and planning notes
live under `.agents/plans/` (org scratch — see release boundary below) and
are **not** product docs.

| File | What it is |
|------|------------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Language / userland seam; governing architecture map |

## Comment hygiene (dev)

Comments carry contracts and **why**. No narration, no dates/names, no agent
voice (`I`, session monologue, TODOs addressed to no one).

## Release boundary (`.agents/`)

`.agents/` holds plans, specs, and postmortems — **org scratch**. It stays in
the private/work monorepo for impl agents. At OSS-cut / public mirror:

- Exclude `.agents/` from the public tree (release packaging or public-mirror
  `.gitignore`).
- Do not ship operator machine paths, mission names, or agent monologue.
- Product docs under `docs/` remain; working specs do not.

This directory is **not** deleted in hygiene passes — only stripped from
release surfaces.
