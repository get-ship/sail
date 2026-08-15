<!--
  Lab canvas for InputWithButton. Mount only — not a template.
  Source of truth: src/blocks/InputWithButton/InputWithButton.sail
-->
<scope>
  style canvas = { pad(6) }
</scope>

<column gap="3" style(canvas)>
  <small>InputWithButton</small>
  <row align="end" gap="2">
    <field>
      <label slot="label" for="input-with-button-q">Search questions</label>
      <input id="input-with-button-q" placeholder="Search questions" />
    </field>
    <button>Search</button>
  </row>
</column>
