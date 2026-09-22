<scope>
type TileProps = { title: string }
style card = { pad(4).bg(primary).radius(md) }
</scope>
<controller>
import { controller } from '@ship/sail-runtime'

export default controller(() => ({
  first: 'Alpha',
  second: 'Beta',
}))
</controller>
<partial list-tile props: TileProps>
  <div data-testid="tile" style(card)>
    <p data-testid="title">{title}</p>
    <slot/>
  </div>
</partial>
<div data-testid="host">
  <list-tile title={first}><span data-testid="slot-a">A</span></list-tile>
  <list-tile title={second}><span data-testid="slot-b">B</span></list-tile>
</div>
