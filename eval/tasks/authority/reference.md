# Design reference — Authority (approval flow)

Distilled from the Ship "Spaces" interaction-model mock (`scene-authority`).
This is the behavioral source of truth; you are NOT matching pixels. Re-express
it as a real Ship view in SUI idioms.

## The model: the hub decides, spaces request

The **hub** is the only surface that can grant capabilities or approve risky
commands. Running spaces (agents) make requests; the human approves or denies in
the hub. Two surfaces matter for this task:

## A. Authority queue (the main view)

Header:
- Title: **Authority**
- Subtitle: *Capability requests, command approvals, and the identities your spaces speak for.*
- Two header buttons: **Replay decisions**, **Policy…**

Pending section:
- Section label: **Awaiting your decision**
- A live counter: `{n} pending · oldest 38s`
- A grid of **pending cards**, one per request. Each card shows:
  - `<Agent> · <Space>` (e.g. `Editor · Paperback`)
  - a kind line (mono, caps): **wants to run** (command) or **requests capability**
  - the body:
    - command → `$ <command>` in a mono/dark block (e.g. `$ git push --force-with-lease origin main`)
    - capability → `<scope> <path>` (e.g. `fs:read ~/Documents/drafts/`)
  - a **Why · <reason>** line
  - a risk tag: `medium` / `high`
  - three actions: **Deny**, **Allow once**, **Allow this session** (primary)

Below the queue (secondary, lower priority for this task): a **Decisions** log
(recent allow/deny rows) and **Active grants** (each with a **revoke** action).
Include them if time allows; the queue + modal are the graded core.

## B. Capability request modal

Opened when a pending capability card is focused/expanded. A centered dialog:

- Title (mono, caps): **Capability request**
- Subheading: `<Agent> wants to read your <thing>` (e.g. *Editor wants to read your filesystem*)
- Section **SCOPE**: a scope tag + path (e.g. `fs:read ~/Documents/drafts/`), detail *recursive · 28 files*
- Section **WHY**: prose reason
- Section **FOR HOW LONG**: three single-select options —
  - **Just this file** (one-shot)
  - **Until I close <Space>** (session) — selected by default
  - **Always** (persistent)
- Footer: hint `⎋ to deny`, buttons **Deny** and **Allow** (primary)
- Tail line: *Decided in hub · never inside a space.*

## Data shape (apply-our-insight — define it, fake it in tests)

There is no `authority` SDK domain yet. Code the view against this shape and the
behavioral tests fake it at the `ship` boundary:

```ts
ship.authority.pending()   // live → { kind:'ok', value: AuthorityRequest[] }
ship.authority.grants()    // live → Grant[]
ship.authority.decide({ id, verdict: 'allow_once'|'allow_session'|'deny', duration? })  // mutation
```

```ts
interface AuthorityRequest {
  id: string
  agent: string          // "Editor"
  space: string          // "Paperback"
  kind: 'command' | 'capability'
  command?: string       // when kind==='command'
  scope?: string         // when kind==='capability', e.g. "fs:read"
  path?: string          // e.g. "~/Documents/drafts/"
  why: string
  risk: 'low' | 'medium' | 'high'
  ageLabel: string       // "38s"
}
```

Approving/denying a request removes it from the pending list (optimistic) and the
counter decrements.
