// Code snippets for the data page. Kept OUT of the controller so the compiler's
// scope scan isn't fooled by patterns inside these strings.
export const code = {
  data: `<!-- profile.sui -->
<client name="api" />
<data name="user" source="api.users.get(id())" />

{#if user().kind === 'loading'}
  <sui-spinner></sui-spinner>
{:else if user().kind === 'error'}
  <sui-alert tone="destructive">{user().error.message}</sui-alert>
{:else}
  <h1>{user().value.name}</h1>
{/if}`,
  live: `<!-- a live source re-renders as new values arrive -->
<data name="prices" source="api.ticker.stream()" live />

{#for row of prices() key={row.symbol}}
  <tr><td>{row.symbol}</td><td>{row.price}</td></tr>
{/for}`,
}
