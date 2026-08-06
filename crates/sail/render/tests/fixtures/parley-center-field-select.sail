<!-- RENDER-0 regression: center > column > field > select with options panicked parley
     via grid min-content measure of padded select option text. -->
<center>
  style.pad(6).minH(screen).bg(background)

  <column gap="4" w="full" max-w="panel-max-w-md">
    <h1>
      style.fontSize(2xl).fontWeight(semibold).fg(foreground)
Shipping address</h1>
    <card w="full">
      <form>
        <column gap="4" w="full">
          style.pad(5)

          <field w="full">
            <label slot="label" required>Name</label>
            <input type="text" name="name" placeholder="Ada Lovelace" w="full" />
          </field>
          <field w="full">
            <label slot="label" required>Country</label>
            <select name="country" w="full">
              <option value="">Select a country</option>
              <option value="US">United States</option>
              <option value="CA">Canada</option>
              <option value="Other">Other</option>
            </select>
          </field>
          <row gap="2" align="center" justify="end" w="full">
            <button type="button" variant="outline">Reset</button>
            <button type="submit" variant="solid" tone="primary">Save address</button>
          </row>
        </column>
      </form>
    </card>
  </column>
</center>
