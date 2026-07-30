<column gap="4" sx={style.pad(6)}>
  <h1>Team chat</h1>
  <card>
    <column gap="3" sx={style.pad(4)}>
      <column gap="1">
        <badge>Maya</badge>
        <p>Ship the eval harness today?</p>
      </column>
      <column gap="1">
        <badge>Jon</badge>
        <p>On it — cheap lane first.</p>
      </column>
    </column>
  </card>
  <row gap="2" align="end">
    <column gap="1">
      <label for="message">Message</label>
      <input id="message" name="message" placeholder="Message" />
    </column>
    <button type="button" tone="primary">Send</button>
  </row>
</column>
