// Usage snippets shown beside each live component. Kept out of the controller
// so the scope scan isn't fooled by patterns inside the strings.
export const code = {
  button: `<sui-button tone="primary">Save changes</sui-button>
<sui-button variant="outline">Cancel</sui-button>
<sui-button variant="ghost">Ghost</sui-button>`,
  chip: `<sui-chip>Design</sui-chip>
<sui-chip tone="primary" selected>Selected</sui-chip>
<sui-chip removable>Removable</sui-chip>`,
  badge: `<sui-badge>Default</sui-badge>
<sui-badge tone="success">Live</sui-badge>
<sui-badge tone="primary" variant="solid">New</sui-badge>`,
  input: `<sui-input placeholder="you@example.com"></sui-input>`,
  checkbox: `<sui-checkbox checked>Subscribe to updates</sui-checkbox>`,
  switch: `<sui-switch checked tone="primary"></sui-switch>`,
  alert: `<sui-alert tone="success">Your changes were saved.</sui-alert>`,
  spinner: `<sui-spinner></sui-spinner>`,
  avatar: `<sui-avatar initials="MC"></sui-avatar>
<sui-avatar name="Ada Lovelace"></sui-avatar>`,
  card: `<sui-card>A card groups related content on a raised surface.</sui-card>`,
  code: `<sui-code>const sum = (a, b) => a + b</sui-code>`,
}
