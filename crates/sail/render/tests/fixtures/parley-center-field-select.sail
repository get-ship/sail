<!-- RENDER-0 regression: center > column > field > select with options panicked parley
     via grid min-content measure of padded select option text. -->
<center style({ pad(6).size({ min-h: screen }).bg(background) })>
  <column gap="4" style({ size({ w: 100%, max-w: overlay-max-w-md }) })>
    <h1 style({ fg(foreground).type({ size: 2xl, weight: semibold }) })>Shipping address</h1>
    <card style({ size({ w: 100% }) })>
      <form>
        <column gap="4" style({ size({ w: 100% }).pad(5) })>
          <field style({ size({ w: 100% }) })>
            <label slot="label" required>Name</label>
            <input type="text" name="name" placeholder="Ada Lovelace" style({ size({ w: 100% }) })>
          </field>
          <field style({ size({ w: 100% }) })>
            <label slot="label" required>Country</label>
            <select name="country" style({ size({ w: 100% }) })>
              <option value="">Select a country</option>
              <option value="US">United States</option>
              <option value="CA">Canada</option>
              <option value="Other">Other</option>
            </select>
          </field>
          <row gap="2" align="center" justify="end" style({ size({ w: 100% }) })>
            <button type="button" variant="outline">Reset</button>
            <button type="submit" variant="solid" tone="primary">Save address</button>
          </row>
        </column>
      </form>
    </card>
  </column>
</center>
