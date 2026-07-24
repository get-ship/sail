<!--
  <menu-item> — item inside dropdown/context-menu. Host sx owns paint; no residue.
  Button L1 contract supplies inline-flex/align/gap chrome.

  Paint: fg / hover-bg from component.menu-item tokens (cssNames
  menu-item-foreground, menu-item-hover-bg).
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<button
  type="button"
  sx="w: full p: 2 px: 2_5 border: 0 text: start font-family: body font-size: sm leading: tight color: menu-item-foreground bg: transparent rounded: sm cursor: pointer whitespace: nowrap hover:bg: menu-item-hover-bg focus:bg: menu-item-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring disabled:opacity: 50 disabled:cursor: not-allowed"
>
  <slot></slot>
</button>
