<!--
  Look: painted boxes that spend every type slot.
  Spelling: type({ family, size, weight, leading, tracking, align, case, wrap, list, decoration, italic, nums, break })
  Owner: language (type slot table)
  Default: composites body / body-sm / heading / label / mono / mono-sm / display / display-sm
  Trap: tokens/test/fixtures/font-family-raw.sail — family is role tokens only
-->
<column gap="6">
  <l0-case look="presets — the product molecules" spell="type(body) · type(body-sm) · type(heading) · type(label) · type(mono) · type(mono-sm) · type(display) · type(display-sm)">
    <row gap="3">
      <l0-type-preset kind="body">body</l0-type-preset>
      <l0-type-preset kind="body-sm">body-sm</l0-type-preset>
      <l0-type-preset kind="heading">heading</l0-type-preset>
      <l0-type-preset kind="label">label</l0-type-preset>
      <l0-type-preset kind="mono">mono</l0-type-preset>
      <l0-type-preset kind="mono-sm">mono-sm</l0-type-preset>
      <l0-type-preset kind="display">display</l0-type-preset>
      <l0-type-preset kind="display-sm">display-sm</l0-type-preset>
    </row>
  </l0-case>
  <l0-case look="family — body · heading · mono · display" spell="type family: body | heading | mono | display">
    <row gap="3">
      <l0-type-face kind="family-body">body</l0-type-face>
      <l0-type-face kind="family-heading">heading</l0-type-face>
      <l0-type-face kind="family-mono">mono</l0-type-face>
      <l0-type-face kind="family-display">display</l0-type-face>
    </row>
  </l0-case>
  <l0-case look="size — xs · sm · base · md · lg · xl · 2xl" spell="type size: xs | sm | base | md | lg | xl | 2xl">
    <row gap="3">
      <l0-type-face kind="size-xs">xs</l0-type-face>
      <l0-type-face kind="size-sm">sm</l0-type-face>
      <l0-type-face kind="size-base">base</l0-type-face>
      <l0-type-face kind="size-md">md</l0-type-face>
      <l0-type-face kind="size-lg">lg</l0-type-face>
      <l0-type-face kind="size-xl">xl</l0-type-face>
      <l0-type-face kind="size-2xl">2xl</l0-type-face>
    </row>
  </l0-case>
  <l0-case look="weight — normal · medium · semibold · bold" spell="type weight: normal | medium | semibold | bold">
    <row gap="3">
      <l0-type-face kind="weight-normal">normal</l0-type-face>
      <l0-type-face kind="weight-medium">medium</l0-type-face>
      <l0-type-face kind="weight-semibold">semibold</l0-type-face>
      <l0-type-face kind="weight-bold">bold</l0-type-face>
    </row>
  </l0-case>
  <l0-case look="leading — none · tight · snug · normal · relaxed" spell="type leading: none | tight | snug | normal | relaxed">
    <row gap="3">
      <l0-type-face kind="leading-none">Aa Gg</l0-type-face>
      <l0-type-face kind="leading-tight">Aa Gg</l0-type-face>
      <l0-type-face kind="leading-snug">Aa Gg</l0-type-face>
      <l0-type-face kind="leading-normal">Aa Gg</l0-type-face>
      <l0-type-face kind="leading-relaxed">Aa Gg</l0-type-face>
    </row>
  </l0-case>
  <l0-case look="tracking — tighter · tight · normal · wide · wider" spell="type tracking: tighter | tight | normal | wide | wider">
    <row gap="3">
      <l0-type-face kind="tracking-tighter">Aa</l0-type-face>
      <l0-type-face kind="tracking-tight">Aa</l0-type-face>
      <l0-type-face kind="tracking-normal">Aa</l0-type-face>
      <l0-type-face kind="tracking-wide">Aa</l0-type-face>
      <l0-type-face kind="tracking-wider">Aa</l0-type-face>
    </row>
  </l0-case>
  <l0-case look="align — left · center · right · justify · start · end" spell="type align: left | center | right | justify | start | end">
    <column gap="2">
      <l0-type-face kind="align-left">Aa</l0-type-face>
      <l0-type-face kind="align-center">Aa</l0-type-face>
      <l0-type-face kind="align-right">Aa</l0-type-face>
      <l0-type-face kind="align-justify">Aa Aa Aa Aa</l0-type-face>
      <l0-type-face kind="align-start">Aa</l0-type-face>
      <l0-type-face kind="align-end">Aa</l0-type-face>
    </column>
  </l0-case>
  <l0-case look="case — uppercase · lowercase · capitalize · normal · label-transform" spell="type case: uppercase | lowercase | capitalize | normal | label-transform">
    <row gap="3">
      <l0-type-face kind="case-uppercase">Aa Bb</l0-type-face>
      <l0-type-face kind="case-lowercase">Aa Bb</l0-type-face>
      <l0-type-face kind="case-capitalize">Aa Bb</l0-type-face>
      <l0-type-face kind="case-normal">Aa Bb</l0-type-face>
      <l0-type-face kind="case-label">Aa Bb</l0-type-face>
    </row>
  </l0-case>
  <l0-case look="wrap — normal · nowrap · truncate · pretty · words · pre-wrap" spell="type wrap: normal | nowrap | truncate | pretty | words | pre-wrap">
    <row gap="3">
      <l0-type-face kind="wrap-normal">the quick brown fox jumps</l0-type-face>
      <l0-type-face kind="wrap-nowrap">the quick brown fox jumps</l0-type-face>
      <l0-type-face kind="wrap-truncate">the quick brown fox jumps</l0-type-face>
      <l0-type-face kind="wrap-pretty">the quick brown fox jumps</l0-type-face>
      <l0-type-face kind="wrap-words">supercalifragilistic</l0-type-face>
      <l0-type-face kind="wrap-pre-wrap">the  quick  fox</l0-type-face>
    </row>
  </l0-case>
  <l0-case look="list — none · disc · decimal" spell="type list: none | disc | decimal">
    <row gap="3">
      <l0-type-face kind="list-none">
        <ul>
          <li>one</li>
          <li>two</li>
        </ul>
      </l0-type-face>
      <l0-type-face kind="list-disc">
        <ul>
          <li>one</li>
          <li>two</li>
        </ul>
      </l0-type-face>
      <l0-type-face kind="list-decimal">
        <ul>
          <li>one</li>
          <li>two</li>
        </ul>
      </l0-type-face>
    </row>
  </l0-case>
  <l0-case look="decoration — none · underline · line-through" spell="type decoration: none | underline | line-through">
    <row gap="3">
      <l0-type-face kind="decoration-none">Aa</l0-type-face>
      <l0-type-face kind="decoration-underline">Aa</l0-type-face>
      <l0-type-face kind="decoration-line-through">Aa</l0-type-face>
    </row>
  </l0-case>
  <l0-case look="italic — true · false" spell="type italic: true | false">
    <row gap="3">
      <l0-type-face kind="italic-true">Ag</l0-type-face>
      <l0-type-face kind="italic-false">Ag</l0-type-face>
    </row>
  </l0-case>
  <l0-case look="nums — tabular · lining · oldstyle · proportional · ordinal · slashed-zero · normal" spell="type nums: tabular | lining | oldstyle | proportional | ordinal | slashed-zero | normal">
    <row gap="3">
      <l0-type-face kind="nums-tabular">111 888</l0-type-face>
      <l0-type-face kind="nums-lining">111 888</l0-type-face>
      <l0-type-face kind="nums-oldstyle">111 888</l0-type-face>
      <l0-type-face kind="nums-proportional">111 888</l0-type-face>
      <l0-type-face kind="nums-ordinal">1st</l0-type-face>
      <l0-type-face kind="nums-slashed-zero">100</l0-type-face>
      <l0-type-face kind="nums-normal">111 888</l0-type-face>
    </row>
  </l0-case>
  <l0-case look="break — words · normal · anywhere" spell="type break: words | normal | anywhere">
    <row gap="3">
      <l0-type-face kind="break-words">supercalifragilistic</l0-type-face>
      <l0-type-face kind="break-normal">supercalifragilistic</l0-type-face>
      <l0-type-face kind="break-anywhere">supercalifragilistic</l0-type-face>
    </row>
  </l0-case>
</column>
