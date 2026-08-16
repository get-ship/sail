<!--
  AuthenticationTitle — identity fields under a title, one primary action.
  The form is the measure column. The card is a surface inside it, not the page.
-->
<form label="Sign in">
  <column gap="6">
    <column gap="1" align="center">
      <h1>Welcome back!</h1>
      <p>Do not have an account yet? <a href="#">Create account</a></p>
    </column>
    <card variant="outline">
      <column gap="4">
        <field>
          <label slot="label" for="auth-email" required>Email</label>
          <input id="auth-email" type="email" placeholder="you@ship.dev" required />
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
</form>
