<!-- RENDER-0 regression: center > column > field > select with options panicked parley
     via grid min-content measure of padded select option text. -->
<center style({ box: pad(6).minH(screen), paint: bg(background) })>
  <column gap="4" style({ box: w(full).maxW(panel-max-w-md) })>
    <h1 style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(semibold) })>Shipping address</h1>
    <card style({ box: w(full) })>
      <form>
        <column gap="4" style({ box: w(full).pad(5) })>
          <field style({ box: w(full) })>
            <label slot="label" required>Name</label>
            <input type="text" name="name" placeholder="Ada Lovelace" style({ box: w(full) })>
          </field>
          <field style({ box: w(full) })>
            <label slot="label" required>Country</label>
            <select name="country" style({ box: w(full) })>
              <option value="">Select a country</option>
              <option value="US">United States</option>
              <option value="CA">Canada</option>
              <option value="Other">Other</option>
            </select>
          </field>
          <row gap="2" align="center" justify="end" style({ box: w(full) })>
            <button type="button" variant="outline">Reset</button>
            <button type="submit" variant="solid" tone="primary">Save address</button>
          </row>
        </column>
      </form>
    </card>
  </column>
</center>
