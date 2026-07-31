# dependent-form

**System stressed:** state / controller

## Mechanical

- Compiles clean
- Vocab share ≥ 0.25
- No layout gate failures

## Scope criteria (beauty / human)

| Criterion | 0 | 1 | 2 |
|---|---|---|---|
| Fields present | Missing required fields | Most fields | Name, email, country, state, submit |
| Validation | None | Partial | Required + email format signals |
| Country→state cascade | Hardcoded / broken | Partial options | Reset + repopulate by country |
| Lifecycle states | None | One of dirty/submitting/success | Dirty + submitting + success visible |

Beauty is human-graded; screenshot fidelity may lag RENDER-1.
