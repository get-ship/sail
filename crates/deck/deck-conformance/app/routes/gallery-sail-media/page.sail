<column gap="8">
  style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })

  <header>
    style({ box: padB(4), paint: borderBWidth(1).borderB(border) })

    <h1>
      style({ box: mb(2), paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })
sail/media — Domain Stubs</h1>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm) })

      Pack <strong>sail/media</strong>. Each card renders the real custom element
      tag with static mock attribute values so the shell registers and paints.
      A visible badge marks each card as a domain stub — no <code>ship.*</code>
      calls, no live data. Structured inputs (<code>src</code>,
      <code>table</code>) arrive via identity at runtime; this gallery
      exercises only reflected scalar attributes and the <code>text</code>
      string fallback prop.
    </p>
  </header>

  <!-- ── sail-media-av ──────────────────────────────────────────────── -->
  <column gap="4">
    style({ box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })

    <h2>
      style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold).fontFamily(mono) })

      <code>sail-media-av</code>
      <span>
        style({ box: padX(2).padY(1).ms(3), paint: fg(warning-foreground).bg(warning).radius(sm), text: fontSize(xs).fontWeight(medium).fontFamily(body).tracking(wide) })
domain-stub — awaits backend spec</span>
    </h2>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) })

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
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
kind=video controls</span>
        <sail-media-av kind="video" controls></sail-media-av>
      </column>
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
kind=audio controls</span>
        <sail-media-av kind="audio" controls></sail-media-av>
      </column>
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
kind=audio controls waveform</span>
        <sail-media-av kind="audio" controls waveform></sail-media-av>
      </column>
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
kind=auto controls (default)</span>
        <sail-media-av kind="auto" controls></sail-media-av>
      </column>
    </column>
  </column>

  <!-- ── sail-media-csv ─────────────────────────────────────────────── -->
  <column gap="4">
    style({ box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })

    <h2>
      style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold).fontFamily(mono) })

      <code>sail-media-csv</code>
      <span>
        style({ box: padX(2).padY(1).ms(3), paint: fg(warning-foreground).bg(warning).radius(sm), text: fontSize(xs).fontWeight(medium).fontFamily(body).tracking(wide) })
domain-stub — awaits backend spec</span>
    </h2>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) })

      Props: <code>delimiter: comma|tab|auto</code> (reflect, default auto),
      <code>header: bool</code> (reflect, default true),
      <code>text: string</code> (SSR/declarative fallback — inline CSV text).
      The resolved CSV/TSV bytes arrive via <code>src</code> identity;
      the <code>text</code> prop is the static fallback and is used here.
      Composes <code>sail-table variant="data"</code> internally.
    </p>
    <column gap="4">
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
delimiter=comma header text (inline CSV)</span>
        <sail-media-csv delimiter="comma" header text="Name,Role,Status
Alice,Engineer,active
Bob,Designer,active
Carol,PM,inactive"></sail-media-csv>
      </column>
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
delimiter=tab header text (inline TSV)</span>
        <sail-media-csv delimiter="tab" header text="File	Size	Modified
readme.md	1.2 KB	2026-06-01
main.ts	4.8 KB	2026-06-04
style.css	2.1 KB	2026-05-30"></sail-media-csv>
      </column>
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
delimiter=auto no-header</span>
        <sail-media-csv delimiter="auto" text="alpha,1,true
beta,2,false
gamma,3,true"></sail-media-csv>
      </column>
    </column>
  </column>

  <!-- ── sail-media-image ───────────────────────────────────────────── -->
  <column gap="4">
    style({ box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })

    <h2>
      style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold).fontFamily(mono) })

      <code>sail-media-image</code>
      <span>
        style({ box: padX(2).padY(1).ms(3), paint: fg(warning-foreground).bg(warning).radius(sm), text: fontSize(xs).fontWeight(medium).fontFamily(body).tracking(wide) })
domain-stub — awaits backend spec</span>
    </h2>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) })

      Props: <code>alt: string</code> (reflect),
      <code>fit: contain|cover|actual</code> (reflect, default contain),
      <code>zoomable: bool</code> (reflect, default true),
      <code>gallery: bool</code> (reflect, default false).
      The resolved image source arrives via <code>src</code> identity;
      the stage renders its empty state here. In gallery mode a
      <code>src</code> array drives the thumbnail strip.
    </p>
    <column gap="4">
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
fit=contain zoomable (default)</span>
        <sail-media-image alt="Sample image — contain fit" fit="contain" zoomable></sail-media-image>
      </column>
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
fit=cover zoomable</span>
        <sail-media-image alt="Sample image — cover fit" fit="cover" zoomable></sail-media-image>
      </column>
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
fit=actual (no zoom)</span>
        <sail-media-image alt="Sample image — actual size" fit="actual"></sail-media-image>
      </column>
      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
gallery mode (strip, no src array wired)</span>
        <sail-media-image alt="Gallery demo" fit="contain" gallery></sail-media-image>
      </column>
    </column>
  </column>
</column>
