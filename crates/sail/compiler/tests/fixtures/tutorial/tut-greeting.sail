<!-- Owned tutorial fixture: a component is a <scope> block + markup.
     Compile as a component (component_meta set) or props are refused. -->
<scope>
  props: {
    name = "world"
  }
</scope>

<p>Hello, {name()}.</p>
