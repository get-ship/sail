<!--
  <menu-item> — item inside dropdown/context-menu. Host sx owns paint; no residue.
  Button L1 contract supplies inline-flex/align/gap chrome.
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<button
  type="button"
  sx="w-full p-2 px-2_5 border-0 text-start font-body text-sm leading-tight text-foreground bg-transparent rounded-sm cursor-pointer whitespace-nowrap hover:bg-muted focus:bg-muted focus:outline-solid focus:outline-2 focus:outline-ring disabled:opacity-50 disabled:cursor-not-allowed"
>
  <slot></slot>
</button>
