<controller>
import { controller } from '@ship/sail-runtime'
export default controller((ctx) => ({
  rows: [{ id: 1, label: "a" }, { id: 2, label: "b" }]
}))
</controller>

<ul>
  <for as="row" of={rows()} key={row.id}>
    <li>{row.label}</li>
  </for>
</ul>
