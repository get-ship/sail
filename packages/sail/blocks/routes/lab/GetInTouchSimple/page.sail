<!--
  Lab canvas for GetInTouchSimple. Mount only — not a template.
  Source of truth: src/blocks/GetInTouchSimple/GetInTouchSimple.sail
-->
<scope>
  style canvas = { pad(6) }
</scope>

<column gap="3" style(canvas)>
  <small>GetInTouchSimple</small>
  <form>
    <column gap="4">
      <center>
        <h2>Get in touch</h2>
      </center>
      <grid min="sm" gap="4">
        <field>
          <label slot="label" for="get-in-touch-name">Name</label>
          <input id="get-in-touch-name" name="name" surface="filled" placeholder="Your name" />
        </field>
        <field>
          <label slot="label" for="get-in-touch-email">Email</label>
          <input id="get-in-touch-email" name="email" type="email" surface="filled" placeholder="Your email" />
        </field>
      </grid>
      <field>
        <label slot="label" for="get-in-touch-subject">Subject</label>
        <input id="get-in-touch-subject" name="subject" surface="filled" placeholder="Subject" />
      </field>
      <field>
        <label slot="label" for="get-in-touch-message">Message</label>
        <textarea id="get-in-touch-message" name="message" surface="filled" placeholder="Your message" rows="5"></textarea>
      </field>
      <center>
        <button type="submit">Send message</button>
      </center>
    </column>
  </form>
</column>
