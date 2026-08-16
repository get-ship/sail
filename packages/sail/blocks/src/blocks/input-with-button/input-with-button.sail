<!--
  InputWithButton — search field + submit. Defaults only.
  size(measure) is the field, not the row: a hugging input is
  the label width. Field fill stretches the control inside.
-->
<scope>
  style shell = { size(measure) }
</scope>

<row align="end" gap="2">
  <column style(shell)>
    <field>
      <label slot="label" for="input-with-button-q">Search questions</label>
      <input id="input-with-button-q" placeholder="Search questions" />
    </field>
  </column>
  <button>Search</button>
</row>
