<controller>
import { controller, signal, derived, effect, form } from '@ship/sail-runtime'

interface AddressValues extends Record<string, unknown> {
  name: string
  email: string
  country: string
  state: string
}

interface Region {
  code: string
  name: string
}

const US_STATES: Region[] = [
  { code: 'AL', name: 'Alabama' },
  { code: 'AK', name: 'Alaska' },
  { code: 'AZ', name: 'Arizona' },
  { code: 'CA', name: 'California' },
  { code: 'CO', name: 'Colorado' },
  { code: 'FL', name: 'Florida' },
  { code: 'GA', name: 'Georgia' },
  { code: 'IL', name: 'Illinois' },
  { code: 'MA', name: 'Massachusetts' },
  { code: 'NY', name: 'New York' },
  { code: 'TX', name: 'Texas' },
  { code: 'WA', name: 'Washington' },
]

const CA_PROVINCES: Region[] = [
  { code: 'AB', name: 'Alberta' },
  { code: 'BC', name: 'British Columbia' },
  { code: 'MB', name: 'Manitoba' },
  { code: 'NB', name: 'New Brunswick' },
  { code: 'NL', name: 'Newfoundland and Labrador' },
  { code: 'NS', name: 'Nova Scotia' },
  { code: 'NT', name: 'Northwest Territories' },
  { code: 'NU', name: 'Nunavut' },
  { code: 'ON', name: 'Ontario' },
  { code: 'PE', name: 'Prince Edward Island' },
  { code: 'QC', name: 'Quebec' },
  { code: 'SK', name: 'Saskatchewan' },
  { code: 'YT', name: 'Yukon' },
]

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

export default controller(() => {
  const success = signal(false)
  const attempted = signal(false)

  const shipping = form<AddressValues>({
    initial: {
      name: '',
      email: '',
      country: '',
      state: '',
    },
    validate: ({ name, email, country, state }) => {
      const errors: Partial<Record<keyof AddressValues, string | null>> = {}
      if (!name.trim()) errors.name = 'Name is required'
      if (!email.trim()) errors.email = 'Email is required'
      else if (!EMAIL_RE.test(email.trim())) errors.email = 'Enter a valid email'
      if (!country) errors.country = 'Country is required'
      if ((country === 'US' || country === 'CA') && !state) {
        errors.state = country === 'US' ? 'State is required' : 'Province is required'
      }
      return errors
    },
    submit: async () => {
      await new Promise((resolve) => setTimeout(resolve, 600))
      success.set(true)
    },
  })

  // Cascade: country change resets and repopulates the region field.
  let prevCountry = shipping.fields.country()
  effect(() => {
    const next = shipping.fields.country()
    if (next !== prevCountry) {
      shipping.fields.state.set('')
      prevCountry = next
      success.set(false)
    }
  })

  // Editing after a successful save clears the success banner.
  effect(() => {
    if (shipping.dirty()) success.set(false)
  })

  const regions = derived((): Region[] => {
    const c = shipping.fields.country()
    if (c === 'US') return US_STATES
    if (c === 'CA') return CA_PROVINCES
    return []
  })

  const needsRegion = derived(() => {
    const c = shipping.fields.country()
    return c === 'US' || c === 'CA'
  })

  const regionLabel = derived(() =>
    shipping.fields.country() === 'CA' ? 'Province' : 'State',
  )

  const onSubmit = (e?: Event): Promise<void> => {
    attempted.set(true)
    return shipping.submit(e)
  }

  const onReset = (): void => {
    shipping.reset()
    attempted.set(false)
    success.set(false)
  }

  const nameInvalid = derived(() => attempted() && !!shipping.errors().name)
  const emailInvalid = derived(() => attempted() && !!shipping.errors().email)
  const countryInvalid = derived(() => attempted() && !!shipping.errors().country)
  const stateInvalid = derived(() => attempted() && !!shipping.errors().state)

  return {
    name: shipping.fields.name,
    email: shipping.fields.email,
    country: shipping.fields.country,
    state: shipping.fields.state,
    errors: shipping.errors,
    dirty: shipping.dirty,
    pending: shipping.pending,
    valid: shipping.valid,
    lastError: shipping.lastError,
    success,
    attempted,
    regions,
    needsRegion,
    regionLabel,
    nameInvalid,
    emailInvalid,
    countryInvalid,
    stateInvalid,
    onSubmit,
    onReset,
  }
})
</controller>

<center>
  style({ box: pad(6).minH(screen), paint: bg(background) })

  <column gap="4">
    style({ box: w(full).maxW(panel-max-w-md) })

    <column gap="1">
      <h1>
        style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(semibold) })
Shipping address</h1>
      <p>
        style({ paint: fg(muted-foreground), text: fontSize(sm) })

        Enter where we should send your order. State options depend on country.
      </p>
    </column>

    <row gap="2" align="center">
      <if {dirty()}>
        <badge tone="warning" variant="soft">Unsaved changes</badge>
      </if>
      <if {pending()}>
        <badge tone="info" variant="soft">Submitting…</badge>
      </if>
      <if {success()}>
        <badge tone="success" variant="soft">Saved</badge>
      </if>
    </row>

    <if {success()}>
      <alert tone="success">
        <span slot="title">Address saved</span>
        Shipping details were accepted. You can keep editing if needed.
      </alert>
    </if>

    <if {pending()}>
      <alert tone="info">
        <span slot="title">Submitting</span>
        Saving your shipping address…
      </alert>
    </if>

    <if {lastError()}>
      <alert tone="destructive">
        <span slot="title">Could not save</span>
        {lastError()!.message}
      </alert>
    </if>

    <card>
      style({ box: w(full) })

      <form on:submit={onSubmit}>
        <column gap="4">
          style({ box: w(full).pad(5) })

          <field invalid={nameInvalid()}>
            style({ box: w(full) })

            <label slot="label" required>Name</label>
            <input
              type="text"
              name="name"
              autocomplete="name"
              placeholder="Ada Lovelace"
              bind:value={name}
              disabled={pending()}
            >
              style({ box: w(full) })
            </input>
            <span slot="error">{errors().name}</span>
          </field>

          <field invalid={emailInvalid()}>
            style({ box: w(full) })

            <label slot="label" required>Email</label>
            <input
              type="email"
              name="email"
              autocomplete="email"
              placeholder="ada@example.com"
              bind:value={email}
              disabled={pending()}
            >
              style({ box: w(full) })
            </input>
            <span slot="error">{errors().email}</span>
            <span slot="hint">We use this only for delivery updates.</span>
          </field>

          <field invalid={countryInvalid()}>
            style({ box: w(full) })

            <label slot="label" required>Country</label>
            <select
              name="country"
              bind:value={country}
              disabled={pending()}
            >
              style({ box: w(full) })

              <option value="">Select a country</option>
              <option value="US">United States</option>
              <option value="CA">Canada</option>
              <option value="Other">Other</option>
            </select>
            <span slot="error">{errors().country}</span>
          </field>

          <if {needsRegion()}>
            <field invalid={stateInvalid()}>
              style({ box: w(full) })

              <label slot="label" required>{regionLabel()}</label>
              <select
                name="state"
                bind:value={state}
                disabled={pending()}
              >
                style({ box: w(full) })

                <option value="">Select {regionLabel()}</option>
                <for {regions()} as="region" key={region.code}>
                  <option value={region.code}>{region.name}</option>
                </for>
              </select>
              <span slot="error">{errors().state}</span>
            </field>
          <else>
            <field>
              style({ box: w(full) })

              <label slot="label">State / Province</label>
              <select name="state" disabled>
                style({ box: w(full) })

                <option value="">Not required for Other</option>
              </select>
              <span slot="hint">Region is not required when country is Other.</span>
            </field>
          </else></if>

          <row gap="2" align="center" justify="end">
            style({ box: w(full) })

            <button type="button" variant="outline" on:click={onReset} disabled={pending()}>
              Reset
            </button>
            <button type="submit" variant="solid" tone="primary" disabled={pending()}>
              <if {pending()}>
                <row gap="2" align="center">
                  <spinner size="sm"></spinner>
                  <span>Saving…</span>
                </row>
              <else>
                Save address
              </else></if>
            </button>
          </row>

          <if {attempted() && !valid() && !pending()}>
            <p role="alert">
              style({ paint: fg(destructive), text: fontSize(sm) })

              Fix the highlighted fields before saving.
            </p>
          </if>
        </column>
      </form>
    </card>
  </column>
</center>
