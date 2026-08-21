<!--
  <sail-diff> — unified diff renderer.
  from/to run LCS; value is a raw unified-diff string and wins when set.
  Host role=region + aria-label="diff"; each line is a listitem with kind label.
  Lines use <pre> so space runs stay intact (no white-space Style setter for pre).
-->

<scope>
  props: {
    from = ''
    to = ''
    value = ''
  }

  style root = { size({ w: 100%, min-w: 0 }).overflow({ x: auto }).bg(app) }
  style mono = { display(contents).type({ family: mono, size: xs, leading: normal }) }
  style lineAdd = { pad({ x: 2, y: px }).size({ w: 100%, min-w: 0 }).bg({ color: success, mix: 15 }) }
  style lineDel = { pad({ x: 2, y: px }).size({ w: 100%, min-w: 0 }).bg({ color: danger, mix: 15 }) }
  style lineHunk = { pad({ x: 2, y: px }).size({ w: 100%, min-w: 0 }).bg(muted) }
  style lineCtx = { pad({ x: 2, y: px }).size({ w: 100%, min-w: 0 }) }
  style signAdd = { shrink(0).size({ w: 4 }).fg(success).type({ weight: semibold }) }
  style signDel = { shrink(0).size({ w: 4 }).fg(danger).type({ weight: semibold }) }
  style signHunk = { shrink(0).size({ w: 4 }).fg(info) }
  style signCtx = { shrink(0).size({ w: 4 }).fg(muted-foreground) }
  style textBody = { grow(1).size({ min-w: 0 }).fg(foreground).type({ size: xs }) }
  style textHunk = { grow(1).size({ min-w: 0 }).fg(info).type({ size: xs }) }
</scope>

<controller>

import type { Scope } from './sail-diff.sail'
import { buildDiffLines } from './diff-lines'

export default controller(({ from, to, value }: Scope) => {
    const lines = derived(() => buildDiffLines(from(), to(), value()))

    mount((host) => {
      if (!hostHasAttr(host, 'role')) hostSetAttr(host, 'role', 'region')
      if (!hostHasAttr(host, 'aria-label')) hostSetAttr(host, 'aria-label', 'diff')
    })

    return { lines }
  })
</controller>

<!-- data-line-kind is pack-internal part identity for tests / consumers. -->
<column role="list" gap="0" style(root)>
  <span style(mono)>
    <for {lines() as line} key={line.key}>
      <match {line.kind}>
        {'add' =>
          <row
            role="listitem"
            data-line-kind="add"
            aria-label={line.label}
            nowrap
            gap="0"
            style(lineAdd)
          >
            <small aria-hidden="true" style(signAdd)>{line.sign}</small>
            <pre style(textBody)>{line.text}</pre>
          </row>
        }{'del' =>
          <row
            role="listitem"
            data-line-kind="del"
            aria-label={line.label}
            nowrap
            gap="0"
            style(lineDel)
          >
            <small aria-hidden="true" style(signDel)>{line.sign}</small>
            <pre style(textBody)>{line.text}</pre>
          </row>
        }{'hunk' =>
          <row
            role="listitem"
            data-line-kind="hunk"
            aria-label={line.label}
            nowrap
            gap="0"
            style(lineHunk)
          >
            <small aria-hidden="true" style(signHunk)>{line.sign}</small>
            <pre style(textHunk)>{line.text}</pre>
          </row>
        }{else =>
          <row
            role="listitem"
            data-line-kind="ctx"
            aria-label={line.label}
            nowrap
            gap="0"
            style(lineCtx)
          >
            <small aria-hidden="true" style(signCtx)>{line.sign}</small>
            <pre style(textBody)>{line.text}</pre>
          </row>
        }
      </match>
    </for>
  </span>
</column>
