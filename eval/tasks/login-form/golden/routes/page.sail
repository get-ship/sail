<column gap="6" sx={style.pad(8)}>
  <h1>Sign in</h1>
  <card>
    <column gap="4" sx={style.pad(6)}>
      <label for="email">Email</label>
      <input id="email" type="email" name="email" placeholder="you@example.com" />
      <label for="password">Password</label>
      <input id="password" type="password" name="password" placeholder="Password" />
      <row gap="3" align="center">
        <button type="submit" tone="primary">Sign in</button>
        <a href="/forgot">Forgot password?</a>
      </row>
    </column>
  </card>
</column>
