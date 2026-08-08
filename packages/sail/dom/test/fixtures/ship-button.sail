<scope>
  props: {
    tone: 'primary'
    disabled = false
    loading = false
    label = ''
  }
</scope>
<button data-test="root" disabled={disabled() || loading()} aria-busy={loading() ? 'true' : 'false'} aria-label={label()}>
  <span data-test="label">{label()}</span>
  <if {loading()}>
    <span data-spinner="true" data-test="spinner" aria-hidden="true"></span>
  </if>
</button>
