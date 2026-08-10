# @ship/l1-audit

Per-L1 approval matrix for the component audit mission. **JSON is the SSOT**; this CLI keeps updates mechanical.

## Matrix row

One file per tag: `matrix/<tag>.json`.

Gates: `variants` · `themes` · `drift` · `detail` · `operator`.

Statuses: `todo` | `in_progress` | `partial` | `reviewed` | `pending` | `approved` | `blocked`.

## CLI

```sh
cd packages/sail/l1-audit
node bin/l1-matrix.mjs summary
node bin/l1-matrix.mjs get button
node bin/l1-matrix.mjs set button gates.operator.status=approved
node bin/l1-matrix.mjs seed   # stubs for any stdlib tag missing a row
```

Pilot: **button** only until the gate shape is ratified.
