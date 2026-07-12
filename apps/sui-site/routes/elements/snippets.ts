// Snippets for the elements page. Every SUI sample is compiled by the build.
export const code = {
  native: `<!-- MECHANISM 1 — native + skin. Zero JavaScript.
     The browser owns the behavior; we own the surface. -->
<dialog>
  <h2>Delete this?</h2>
  <button value="cancel">Cancel</button>
  <button value="ok" tone="destructive">Delete</button>
</dialog>

<details>
  <summary>Advanced</summary>
  <p>…</p>
</details>`,

  elementMode: `<!-- MECHANISM 2 — element mode. The tag COMPILES AWAY.
     <combobox> lowers to a <div class="sui-combobox"> plus use:combobox.
     No custom element is registered. There is no runtime cost to the tag. -->
<combobox placeholder="Pick a team">
  <option value="a">Alpha</option>
  <option value="b">Beta</option>
</combobox>`,

  action: `<!-- MECHANISM 3 — use:. Behavior is a function over an element.
     This is how every element above gets its keyboard handling. -->
<textarea use:autosize></textarea>`,

  keyword: `<!-- MECHANISM 4 — layout keywords. Lowered by the preprocessor,
     because they take DYNAMIC values that element mode forbids. -->
<row gap="3" align="center">…</row>`,

  variant: `<!-- A VARIANT IS A PROP, not a component.
     A sheet is a dialog that arrives from the side.
     An alert dialog is a dialog with a role. -->
<dialog side="end">…</dialog>
<dialog role="alertdialog">…</dialog>

<!-- which is why the surface is 48 and not 90 -->
<button tone="destructive" size="sm">Delete</button>`,

  anatomy: `<!-- ANATOMY IS CLASSES. Light DOM, so every internal is reachable and
     documented — no ::part(), no shadow boundary to fight. -->
<card>
  <!-- .sui-card > .header · .body · .footer -->
</card>`,
}
