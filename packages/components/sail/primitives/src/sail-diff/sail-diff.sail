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

  style root = {
    box: w(full).minW(0).overflowX(auto),
    paint: bg(surface-0),
  }
  style mono = {
    box: display(contents),
    text: fontFamily(mono).fontSize(xs).leading(normal),
  }
  style lineAdd = {
    box: padX(2).padY(px).w(full).minW(0),
    paint: colorMix(success, 15),
  }
  style lineDel = {
    box: padX(2).padY(px).w(full).minW(0),
    paint: colorMix(destructive, 15),
  }
  style lineHunk = {
    box: padX(2).padY(px).w(full).minW(0),
    paint: bg(muted),
  }
  style lineCtx = {
    box: padX(2).padY(px).w(full).minW(0),
  }
  style signAdd = {
    box: shrink(0).w(4),
    paint: fg(success),
    text: fontWeight(semibold),
  }
  style signDel = {
    box: shrink(0).w(4),
    paint: fg(destructive),
    text: fontWeight(semibold),
  }
  style signHunk = {
    box: shrink(0).w(4),
    paint: fg(info),
  }
  style signCtx = {
    box: shrink(0).w(4),
    paint: fg(muted-foreground),
  }
  style textBody = {
    box: grow(1).minW(0),
    paint: fg(foreground),
    text: fontSize(xs),
  }
  style textHunk = {
    box: grow(1).minW(0),
    paint: fg(info),
    text: fontSize(xs),
  }
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
