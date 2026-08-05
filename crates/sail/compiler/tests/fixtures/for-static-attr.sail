<controller>
import { controller } from '@ship/sail-runtime'
export default controller((ctx) => ({
  rows: [{ id: 1, label: "a" }, { id: 2, label: "b" }]
}))
</controller>

<table>
  <for {rows()} as="row" key={row.id}>
    <tr id={row.id}><td>cell</td></tr>
  </for>
</table>
