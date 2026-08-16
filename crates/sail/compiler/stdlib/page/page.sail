<!--
  <page> — document shell. Children stack. The page never kisses the viewport.
  Measure column, horizontal inset, column gap. Not a rail or app chrome.
-->

<scope>
  style base = {display(flex).dir(column).size(measure-lg).pad({ x: 6 }).margin({ x: auto })}
</scope>

<column gap="6" style(base)>
  <slot></slot>
</column>
