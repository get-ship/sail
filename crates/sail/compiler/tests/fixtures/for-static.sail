<controller>
import { controller } from '@ship/sail-runtime'
export default controller((ctx) => ({
  rows: [{ id: 1, label: "a" }, { id: 2, label: "b" }]
}))
</controller>

<ul>
  <for {rows()} as="row" key={row.id}>
    <li>static text only</li>
  </for>
</ul>
