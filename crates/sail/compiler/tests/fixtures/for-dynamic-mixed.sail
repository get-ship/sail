<controller>
import { controller } from '@ship/sail-runtime'
export default controller((ctx) => ({
  rows: [{ id: 1, label: "a" }, { id: 2, label: "b" }]
}))
</controller>

<table>
  <for {rows()} as="row" key={row.id}>
    <tr aria-selected={rows().length > 0}><td>{row.label}</td></tr>
  </for>
</table>
