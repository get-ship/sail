<!--
  FooterLinks — link columns and a short mark. Defaults only.
  pad(page) so the band never kisses the glass. min=xs is four 1fr
  tracks once inset (min=sm + gap-6 + pad-lg only fits three).
-->
<scope>
  style shell = { pad(page) }
</scope>

<column gap="6" style(shell)>
  <grid min="xs" gap="6">
    <column gap="2">
      <strong>Ship</strong>
      <p>Build product UI with L1 defaults. If a composition needs paint on every tag, the atoms are wrong.</p>
    </column>
    <column gap="2">
      <h3>About</h3>
      <a href="#">Features</a>
      <a href="#">Pricing</a>
      <a href="#">Support</a>
      <a href="#">Forums</a>
    </column>
    <column gap="2">
      <h3>Project</h3>
      <a href="#">Contribute</a>
      <a href="#">Media assets</a>
      <a href="#">Changelog</a>
      <a href="#">Releases</a>
    </column>
    <column gap="2">
      <h3>Community</h3>
      <a href="#">Join Discord</a>
      <a href="#">Follow on Twitter</a>
      <a href="#">Email newsletter</a>
      <a href="#">GitHub discussions</a>
    </column>
  </grid>
  <separator />
  <row align="center" justify="between">
    <small>© 2026 Ship. All rights reserved.</small>
    <row nowrap gap="3">
      <a href="#">Twitter</a>
      <a href="#">Youtube</a>
      <a href="#">Instagram</a>
    </row>
  </row>
</column>
