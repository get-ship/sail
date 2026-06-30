// Example slot content per primitive for the /playground storybook — the light
// children the viewer injects into the tag-mode stage so slot-driven components
// render populated (a button with a label, a card with a body). Components that
// render their own UI (input, checkbox, calendar, …) need nothing here.
export const EXAMPLES: Record<string, string> = {
  'sui-button': 'Save changes',
  'sui-badge': 'New',
  'sui-chip': 'Design',
  'sui-label': 'Email address',
  'sui-link': 'View documentation',
  'sui-code': 'const sum = (a, b) => a + b',
  'sui-alert': 'Your changes have been saved.',
  'sui-card': '<h3 style="margin:0 0 .5rem">Project Atlas</h3><p style="margin:0;color:var(--muted-foreground)">A workspace for mapping your ideas.</p>',
  'sui-empty-state': 'No results found',
  'sui-brand-mark': 'Ship',
  'sui-markdown': '# Hello\n\nSome **bold** and _italic_ text.',
  'sui-cluster': '<button>One</button><button>Two</button><button>Three</button>',
  'sui-avatar': 'MC',
  'sui-error-state': 'Something went wrong',
  'sui-tooltip': 'Helpful hint',
  'sui-hover-card': 'Hover me',
  'sui-status-dot': '',
  'sui-collapsible': '<summary>Show details</summary><p>Hidden content revealed on toggle.</p>',
}
