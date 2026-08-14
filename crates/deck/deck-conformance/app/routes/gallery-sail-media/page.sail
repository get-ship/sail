<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style pageHead = { pad({ b: 4 }).border({ width: 1, color: border, edge: b }) }
  style title = { margin({ b: 2 }).fg(foreground).type({ size: 2xl, weight: bold }) }
  style lede = { fg(muted-foreground).type({ size: sm }) }
  style card = { pad(5).border({ width: 1, color: border }).radius(lg).bg(surface-1) }
  style cardTitle = { fg(foreground).type({ size: lg, weight: semibold, family: mono }) }
  style badge = { pad({ x: 2, y: 1 }).margin({ s: 3 }).fg(warning-foreground).bg(warning).radius(sm).type({ size: xs, weight: medium, family: body, tracking: wide }) }
  style body = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style stage = { pad(3).border({ width: 1, color: divider }).radius(md).bg(surface-2) }
  style caption = { fg(muted-foreground).type({ size: xs, weight: medium, case: lowercase, tracking: wide, family: mono }) }
</scope>

<column gap="8" style(page)>
  <header style(pageHead)>
    <h1 style(title)>sail/media — Domain Stubs</h1>
    <p style(lede)>
      Pack <strong>sail/media</strong>. Each card renders the real custom element
      tag with static mock attribute values so the shell registers and paints.
      A visible badge marks each card as a domain stub — no <code>ship.*</code>
      calls, no live data. Structured inputs (<code>src</code>,
      <code>table</code>) arrive via identity at runtime; this gallery
      exercises only reflected scalar attributes and the <code>text</code>
      string fallback prop.
    </p>
  </header>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>sail-media-av</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>kind: video|audio|auto</code> (reflect),
      <code>autoplay: bool</code> (reflect, default false),
      <code>controls: bool</code> (reflect, default true),
      <code>waveform: bool</code> (reflect, default false).
      The resolved media source arrives via <code>src</code> identity —
      never a URL attribute. This stub shows the tag shell with reflected
      attributes; no source is wired so the native element renders its
      default empty state.
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>kind=video controls</span>
        <sail-media-av kind="video" controls></sail-media-av>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>kind=audio controls</span>
        <sail-media-av kind="audio" controls></sail-media-av>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>kind=audio controls waveform</span>
        <sail-media-av kind="audio" controls waveform></sail-media-av>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>kind=auto controls (default)</span>
        <sail-media-av kind="auto" controls></sail-media-av>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>sail-media-csv</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>delimiter: comma|tab|auto</code> (reflect, default auto),
      <code>header: bool</code> (reflect, default true),
      <code>text: string</code> (SSR/declarative fallback — inline CSV text).
      The resolved CSV/TSV bytes arrive via <code>src</code> identity;
      the <code>text</code> prop is the static fallback and is used here.
      Composes <code>sail-table variant="data"</code> internally.
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>delimiter=comma header text (inline CSV)</span>
        <sail-media-csv delimiter="comma" header text="Name,Role,Status
Alice,Engineer,active
Bob,Designer,active
Carol,PM,inactive"></sail-media-csv>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>delimiter=tab header text (inline TSV)</span>
        <sail-media-csv delimiter="tab" header text="File	Size	Modified
readme.md	1.2 KB	2026-06-01
main.ts	4.8 KB	2026-06-04
style.css	2.1 KB	2026-05-30"></sail-media-csv>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>delimiter=auto no-header</span>
        <sail-media-csv delimiter="auto" text="alpha,1,true
beta,2,false
gamma,3,true"></sail-media-csv>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>sail-media-image</code>
      <span style(badge)>domain-stub — awaits backend spec</span>
    </h2>
    <p style(body)>
      Props: <code>alt: string</code> (reflect),
      <code>fit: contain|cover|actual</code> (reflect, default contain),
      <code>zoomable: bool</code> (reflect, default true),
      <code>gallery: bool</code> (reflect, default false).
      The resolved image source arrives via <code>src</code> identity;
      the stage renders its empty state here. In gallery mode a
      <code>src</code> array drives the thumbnail strip.
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>fit=contain zoomable (default)</span>
        <sail-media-image alt="Sample image — contain fit" fit="contain" zoomable></sail-media-image>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>fit=cover zoomable</span>
        <sail-media-image alt="Sample image — cover fit" fit="cover" zoomable></sail-media-image>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>fit=actual (no zoom)</span>
        <sail-media-image alt="Sample image — actual size" fit="actual"></sail-media-image>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>gallery mode (strip, no src array wired)</span>
        <sail-media-image alt="Gallery demo" fit="contain" gallery></sail-media-image>
      </column>
    </column>
  </column>
</column>
