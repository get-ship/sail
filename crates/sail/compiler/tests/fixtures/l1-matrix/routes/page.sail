<!-- Owned L1 index. Links to every component detail page. -->
<scope>
  style shell = { pad(6).bg(background) }
  style measure = { size({ w: 100%, max-w: breakpoint-desktop }) }
  style cardLink = { display(block).pad(3).border({ color: border, width: 1 }).radius(md).bg(surface).fg(foreground).cursor(pointer).type({ family: body }), when: { hover: border({ color: primary }).bg(muted) } }
</scope>

<column gap="8" style(shell)>
  <column gap="2" style(measure)>
    <row gap="3" align="center" justify="between">
      <column gap="1">
        <small>L1 Component Matrix</small>
        <h1>Components</h1>
      </column>
      <theme-picker />
    </row>
    <p>Every L1 component with rest-state and axis-sweep specimens. Click through to inspect.</p>
  </column>

  <column gap="2" style(measure)>
    <h2>Actions</h2>
    <row gap="2">
      <a href="/button" style(cardLink)><strong>Button</strong></a>
      <a href="/badge" style(cardLink)><strong>Badge</strong></a>
      <a href="/chip" style(cardLink)><strong>Chip</strong></a>
      <a href="/toggle-group" style(cardLink)><strong>Toggle Group</strong></a>
      <a href="/menu-item" style(cardLink)><strong>Menu Item</strong></a>
      <a href="/pagination" style(cardLink)><strong>Pagination</strong></a>
    </row>
  </column>

  <column gap="2" style(measure)>
    <h2>Fields</h2>
    <row gap="2">
      <a href="/input" style(cardLink)><strong>Input</strong></a>
      <a href="/field" style(cardLink)><strong>Field</strong></a>
      <a href="/fieldset" style(cardLink)><strong>Fieldset</strong></a>
      <a href="/file" style(cardLink)><strong>File Upload</strong></a>
      <a href="/select" style(cardLink)><strong>Select</strong></a>
      <a href="/combobox" style(cardLink)><strong>Combobox</strong></a>
      <a href="/date" style(cardLink)><strong>Date</strong></a>
      <a href="/color" style(cardLink)><strong>Color</strong></a>
      <a href="/slider" style(cardLink)><strong>Slider</strong></a>
      <a href="/checkbox" style(cardLink)><strong>Checkbox</strong></a>
      <a href="/switch" style(cardLink)><strong>Switch</strong></a>
      <a href="/radio-group" style(cardLink)><strong>Radio Group</strong></a>
      <a href="/textarea" style(cardLink)><strong>Textarea</strong></a>
    </row>
  </column>

  <column gap="2" style(measure)>
    <h2>Navigation</h2>
    <row gap="2">
      <a href="/tabs" style(cardLink)><strong>Tabs</strong></a>
      <a href="/accordion" style(cardLink)><strong>Accordion</strong></a>
      <a href="/navbar" style(cardLink)><strong>Navbar</strong></a>
      <a href="/breadcrumb" style(cardLink)><strong>Breadcrumb</strong></a>
      <a href="/pagination" style(cardLink)><strong>Pagination</strong></a>
    </row>
  </column>

  <column gap="2" style(measure)>
    <h2>Feedback</h2>
    <row gap="2">
      <a href="/alert" style(cardLink)><strong>Alert</strong></a>
      <a href="/toast" style(cardLink)><strong>Toast</strong></a>
      <a href="/progress" style(cardLink)><strong>Progress</strong></a>
      <a href="/meter" style(cardLink)><strong>Meter</strong></a>
      <a href="/skeleton" style(cardLink)><strong>Skeleton</strong></a>
      <a href="/spinner" style(cardLink)><strong>Spinner</strong></a>
      <a href="/empty-state" style(cardLink)><strong>Empty State</strong></a>
      <a href="/error-state" style(cardLink)><strong>Error State</strong></a>
      <a href="/error-boundary" style(cardLink)><strong>Error Boundary</strong></a>
    </row>
  </column>

  <column gap="2" style(measure)>
    <h2>Overlays</h2>
    <row gap="2">
      <a href="/dialog" style(cardLink)><strong>Dialog</strong></a>
      <a href="/drawer" style(cardLink)><strong>Drawer</strong></a>
      <a href="/popover" style(cardLink)><strong>Popover</strong></a>
      <a href="/tooltip" style(cardLink)><strong>Tooltip</strong></a>
      <a href="/hover-card" style(cardLink)><strong>Hover Card</strong></a>
      <a href="/context-menu" style(cardLink)><strong>Context Menu</strong></a>
      <a href="/dropdown" style(cardLink)><strong>Dropdown</strong></a>
    </row>
  </column>

  <column gap="2" style(measure)>
    <h2>Content</h2>
    <row gap="2">
      <a href="/card" style(cardLink)><strong>Card</strong></a>
      <a href="/table" style(cardLink)><strong>Table</strong></a>
      <a href="/code-block" style(cardLink)><strong>Code Block</strong></a>
      <a href="/prose" style(cardLink)><strong>Prose</strong></a>
      <a href="/ticker" style(cardLink)><strong>Ticker</strong></a>
      <a href="/scroll-area" style(cardLink)><strong>Scroll Area</strong></a>
      <a href="/resizable" style(cardLink)><strong>Resizable</strong></a>
    </row>
  </column>

  <column gap="2" style(measure)>
    <h2>Media & Identity</h2>
    <row gap="2">
      <a href="/avatar" style(cardLink)><strong>Avatar</strong></a>
      <a href="/icon" style(cardLink)><strong>Icon</strong></a>
      <a href="/kbd" style(cardLink)><strong>Keyboard</strong></a>
      <a href="/separator" style(cardLink)><strong>Separator</strong></a>
      <a href="/a" style(cardLink)><strong>Link</strong></a>
      <a href="/theme-picker" style(cardLink)><strong>Theme Picker</strong></a>
    </row>
  </column>
</column>
