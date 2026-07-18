<scope>
  props: {
    open = false
  }
</scope>
<div data-testid="modal-root" attr:data-open={open() ? 'true' : 'false'}>
  <div data-testid="backdrop" on:click={() => open.set(false)}></div>
  <div>
    <div data-testid="header"><slot name="header"></slot></div>
    <div data-testid="body"><slot></slot></div>
  </div>
</div>
