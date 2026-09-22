<scope>
style card = { pad(4).bg(primary).radius(md) }
</scope>
<controller>
import { controller } from '@ship/sail-runtime'

export default controller(() => ({
  first: 'Alpha',
  second: 'Beta',
}))
</controller>
<div data-testid="host">
  <div data-testid="tile" style(card)>
    <p data-testid="title">{first}</p>
    <span data-testid="slot-a">A</span>
  </div>
  <div data-testid="tile" style(card)>
    <p data-testid="title">{second}</p>
    <span data-testid="slot-b">B</span>
  </div>
</div>
