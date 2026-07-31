<div data-demo="true">
  <h1>Sail Demo</h1>
  <p>Visual sanity-check for the Sail component bundle.</p>

  <section>
    <h2>button — solid</h2>
    <button solid>Default</button>
    <button solid primary>Primary</button>
    <button solid success>Success</button>
    <button solid warning>Warning</button>
    <button solid destructive>Destructive</button>
    <button solid info>Info</button>
  </section>

  <section>
    <h2>button — soft</h2>
    <button soft>Default</button>
    <button soft primary>Primary</button>
    <button soft success>Success</button>
    <button soft destructive>Destructive</button>
  </section>

  <section>
    <h2>button — outline</h2>
    <button outline primary>Primary</button>
    <button outline success>Success</button>
    <button outline destructive>Destructive</button>
  </section>

  <section>
    <h2>button — ghost</h2>
    <button ghost primary>Primary</button>
    <button ghost destructive>Destructive</button>
  </section>

  <section>
    <h2>button — sizes</h2>
    <button solid primary sm>Small</button>
    <button solid primary md>Medium</button>
    <button solid primary lg>Large</button>
  </section>

  <section>
    <h2>button — states</h2>
    <button solid primary disabled>Disabled</button>
    <button solid primary disabled aria-busy="true"><spinner size="sm"></spinner> Loading</button>
  </section>

  <section>
    <h2>sail-badge (dissolved to bare span + sx)</h2>
    <span sx={style.minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(muted).borderWidth(1).border(border).fg(muted-foreground)}>default</span>
    <span sx={style.minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(primary).borderWidth(1).border(primary).fg(primary-foreground)}>primary</span>
    <span sx={style.minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(success).borderWidth(1).border(success).fg(success-foreground)}>success</span>
    <span sx={style.minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(warning).borderWidth(1).border(warning).fg(warning-foreground)}>warning solid</span>
    <span sx={style.minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(transparent).borderWidth(1).border(info).fg(info)}>info outline</span>
    <span sx={style.minH(control-h-2xs).padX(2).fontSize(xs).fontWeight(semibold).radius(sm).bg(destructive).borderWidth(1).border(destructive).fg(destructive-foreground)}>destructive</span>
  </section>

  <section>
    <h2>sail-card</h2>
    <card>
      <h3 slot="header">Card title</h3>
      <p>Card body content lives in the default slot.</p>
      <div slot="footer"><button solid primary>Action</button></div>
    </card>
  </section>
</div>
