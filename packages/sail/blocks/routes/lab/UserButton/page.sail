<!--
  Lab canvas for UserButton. Mount only — not a template.
  Source of truth: src/blocks/UserButton/UserButton.sail
-->
<scope>
  style canvas = { pad(6) }
</scope>

<column gap="3" style(canvas)>
  <small>UserButton</small>
  <button variant="ghost">
    <row align="center" gap="3" nowrap>
      <avatar
        src="https://raw.githubusercontent.com/mantinedev/mantine/master/.demo/avatars/avatar-8.png"
        name="Harriette Spoonlicker"
        initials="HS"
      ></avatar>
      <column>
        <strong>Harriette Spoonlicker</strong>
        <small>hspoonlicker@outlook.com</small>
      </column>
      <icon name="chevron-right" size="xs"></icon>
    </row>
  </button>
</column>
