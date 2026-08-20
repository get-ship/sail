<!--
  <file> — system file picker. Native Browse chrome is unthemeable; the
  native <input type=file> is an opacity overlay so click / drop / form
  stay the platform. variant=drop (default): dashed drop surface.
  variant=picker: same skin, compact well.

  Look: this tag owns its Style. Slots, not :part.
  State: this file's <controller> (summary + invalid/disabled/required/multiple
  as signals, onFiles, keys). No DOM.
  Listen: native input type=file (picker/form) + native <button> (browse)
  + use:hotkeys={keys} Enter/Space on the host.
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    invalid = false
    disabled = false
    name = ''
    required = false
    accept = ''
    multiple = false
    variant: 'picker' | 'drop' = 'drop'
  }

  style pickerHost = {display(flex).position({ scheme: relative }).overflow({ all: hidden }).size({ w: 100%, min-h: control-h-lg }).pad({ x: 4, y: 3 }).group().fg(muted-foreground).bg(muted).border({ color: border, width: 1, style: dashed }).radius(md).cursor(pointer).type({ align: center, family: body, size: sm, weight: medium }), when: {hover: border({ color: primary }).bg({ color: primary, mix: 10 }).fg(foreground), disabled: cursor(not-allowed).filter({ opacity: 50 }).pointerEvents(none)}}
  style dropHost = {display(flex).position({ scheme: relative }).overflow({ all: hidden }).size({ w: 100%, min-h: 32 }).pad(6).group().fg(muted-foreground).bg(muted).border({ color: border, width: 1, style: dashed }).radius(lg).cursor(pointer).type({ align: center, family: body, size: sm, weight: medium }), when: {hover: border({ color: primary }).bg({ color: primary, mix: 10 }).fg(foreground), disabled: cursor(not-allowed).filter({ opacity: 50 }).pointerEvents(none)}}
  style file = {position({ scheme: absolute, inset: 0, z: 1 }).size({ w: 100%, h: 100% }).filter({ opacity: 0 }).cursor(pointer).margin(0).pad(0)}
  style hint = {type({ size: sm })}
  style browse = {appearance(none).bg(transparent).border({ width: 0 }).pad(0).fg(primary).cursor(pointer).type({ family: body, weight: semibold, size: sm })}
</scope>

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  function isOn(v) {
    const x = read(v)
    return x === true || x === '' || x === 'true'
  }
  const hint = 'Drop a file or click to browse'
  const summary = signal(hint)
  function onFiles(e) {
    const list = e && e.target && e.target.files
    const n = list && list.length ? list.length : 0
    if (n === 0) {
      summary.set(hint)
      return
    }
    if (n === 1) {
      summary.set((list[0] && list[0].name) || '1 file')
      return
    }
    summary.set(n + ' files selected')
  }
  function browse() {
    if (isOn(init.disabled)) return
  }
  const invalid = derived(() => isOn(init.invalid))
  const disabled = derived(() => isOn(init.disabled))
  const required = derived(() => isOn(init.required))
  const multiple = derived(() => isOn(init.multiple))
  const keys = isOn(init.disabled)
    ? {}
    : {
        Enter: () => browse(),
        ' ': () => browse(),
      }
  return { summary, onFiles, browse, keys, invalid, disabled, required, multiple }
})
</controller>

<span
  use:hotkeys={keys}
  invalid={invalid()}
  disabled={disabled()}
  style({group(), match: { variant: { picker: pickerHost, drop: dropHost } }, compound: [
      {invalid: true, border({ color: danger })},
      {variant: picker, size: xs, size({ min-h: control-h-2xs }).pad({ x: 2 }).type({ size: xs })},
      {variant: picker, size: sm, size({ min-h: control-h-sm }).pad({ x: 2_5 }).type({ size: sm })},
      {variant: picker, size: md, size({ min-h: control-h-md }).pad({ x: 3 }).type({ size: base })},
      {variant: picker, size: lg, size({ min-h: control-h-lg }).pad({ x: 4 }).type({ size: md })},
    ]})
>
  <input
    type="file"
    anatomy
    on:change={onFiles}
    label="Upload file"
    name={name()}
    accept={accept()}
    required={required()}
    multiple={multiple()}
    disabled={disabled()}
    style(file)
  >
  <row cluster gap="2" align="center" justify="center">
    <span style(hint)>{summary()}</span>
    <button
      type="button"
      anatomy
      label="Browse"
      tabindex="-1"
      disabled={disabled()}
      on:click={browse}
      style(browse)
    >Browse</button>
  </row>
</span>
