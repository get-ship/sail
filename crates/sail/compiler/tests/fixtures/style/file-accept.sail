<!-- Owned construct: MIME accept + named style on <file> / <input>. -->
<scope>
  style page = { size({ w: 100% }) }
  style fieldShell = { size({ w: 100%, max-w: 64 }).pad(2) }
  style fullW = { size({ w: 100% }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs }) }
</scope>
<controller>
export default controller(() => {
  const accept = signal('image/*')
  const heading = () => '<file>'
  const exAccept = () => `<file accept="image/*" />\n<file accept=".pdf,application/pdf" multiple />`
  const exDrop = () => `<file variant="drop" accept="image/*" multiple />`
  return { accept, heading, exAccept, exDrop }
})
</controller>
<column gap="6" style(page)>
  <column style(fieldShell)>
    <file
      accept={accept()}
      style(fullW)
    />
  </column>
  <label style(knobLabel)>Accept</label>
  <input type="text" size="sm" bind:value={accept} placeholder="image/*" />
  <file accept="image/*" style(fullW) />
  <file variant="drop" accept="image/*" multiple style(fullW) />
  <file accept=".pdf,application/pdf" multiple style(fullW) />
</column>
