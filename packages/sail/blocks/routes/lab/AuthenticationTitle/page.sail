<!--
  Lab canvas for AuthenticationTitle. Mount only — not a template.
  Source of truth: src/blocks/AuthenticationTitle/AuthenticationTitle.sail
-->
<scope>
  style canvas = { pad(6) }
</scope>

<column gap="3" style(canvas)>
  <small>AuthenticationTitle</small>
  <column gap="6">
    <column gap="1" align="center">
      <h1>Welcome back!</h1>
      <p>Do not have an account yet? <a href="#">Create account</a></p>
    </column>
    <card variant="outline">
      <column gap="4">
        <field>
          <label slot="label" for="auth-email" required>Email</label>
          <input id="auth-email" type="email" placeholder="you@mantine.dev" required />
        </field>
        <field>
          <label slot="label" for="auth-password" required>Password</label>
          <input id="auth-password" type="password" placeholder="Your password" required />
        </field>
        <row justify="between" align="center" gap="2">
          <label>
            <checkbox />
            Remember me
          </label>
          <a href="#">Forgot password?</a>
        </row>
        <button>Sign in</button>
      </column>
    </card>
  </column>
</column>
