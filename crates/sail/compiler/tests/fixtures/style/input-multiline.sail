<!-- Owned construct: multiline native <input> + named style + if. -->
<scope>
  style page = { size({ w: 100% }) }
  style fullW = { size({ w: 100% }) }
  style mutedSm = { fg(muted-foreground).type({ size: sm }) }
  style mapRow = { size({ w: 100% }).pad({ y: 1 }) }
</scope>
<controller>
export default controller(() => {
  const TYPE_DEFAULTS: Record<string, { placeholder: string; value: string }> = {
    text: { placeholder: 'Display name', value: '' },
    email: { placeholder: 'you@ship.dev', value: '' },
  }
  const inputType = signal('email')
  const isHidden = derived(() => inputType() === 'hidden')
  const isTextField = derived(() => !isHidden())
  const heading = () => '<input>'
  return { inputType, isHidden, isTextField, heading, TYPE_DEFAULTS }
})
</controller>
<column gap="6" style(page)>
  <if {isHidden()}>
    <p style(mutedSm)>Hidden fields take no space.</p>
  </if>
  <if {isTextField()}>
    <input
      type={inputType()}
      placeholder="you@ship.dev"
      style(fullW)
    />
  </if>
  <row gap="3" align="baseline" style(mapRow)>
    <code style({ size({ min-w: 28 }) })>&lt;input&gt;</code>
    <span style(mutedSm)>Text, email</span>
  </row>
  <input type="email" placeholder="you@ship.dev" style(fullW) />
  <file style(fullW) />
  <color value="#c8f542" />
</column>
