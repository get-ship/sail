<!--
  Lesson 02 solved — the whole surface is sx.

  Spacing, color, radius, border, shadow, and type are tokens; interaction
  states are prefixes on the same vocabulary (hover:, focus-within:,
  current:). Nothing here is a raw value — a px or a hex would not compile.
-->
<column gap="4">
  <row gap="1">
    <a href="#usage" aria-current="page"
       sx="px-3 py-1 rounded-full text-sm font-semibold text-muted-foreground hover:text-foreground hover:bg-surface-2 current:bg-primary current:text-primary-foreground">Usage</a>
    <a href="#billing"
       sx="px-3 py-1 rounded-full text-sm font-semibold text-muted-foreground hover:text-foreground hover:bg-surface-2 current:bg-primary current:text-primary-foreground">Billing</a>
    <a href="#limits"
       sx="px-3 py-1 rounded-full text-sm font-semibold text-muted-foreground hover:text-foreground hover:bg-surface-2 current:bg-primary current:text-primary-foreground">Limits</a>
  </row>

  <column gap="3" sx="p-5 bg-surface-1 border border-border rounded-lg shadow-sm hover:shadow-hover">
    <row align="center" justify="between">
      <strong sx="text-sm tracking-wide text-muted-foreground">Compute hours</strong>
      <small sx="text-muted-foreground">renews in 12 days</small>
    </row>
    <row gap="2" align="end">
      <strong sx="text-3xl tracking-tight">412</strong>
      <small sx="text-muted-foreground pb-1">of 500 hrs</small>
    </row>
    <p sx="text-sm text-muted-foreground leading-relaxed">Burst capacity unlocks automatically over 90% usage.</p>
    <row gap="2" align="center" sx="p-2 rounded-md border border-border-subtle focus-within:border focus-within:border-primary">
      <input placeholder="Set a usage alert (hours)" size="sm" />
      <button size="sm" variant="outline" tone="primary">Notify me</button>
    </row>
  </column>
</column>
