<controller>
  export default controller(() => {
    const footer = [
      {
        id: 'product',
        heading: 'Product',
        links: [
          { href: '/dashboard', label: 'Dashboard' },
          { href: '/pricing', label: 'Pricing' },
          { href: '/settings', label: 'Settings' },
        ],
      },
      {
        id: 'company',
        heading: 'Company',
        links: [
          { href: '/pricing', label: 'Customers' },
          { href: '/pricing', label: 'Changelog' },
          { href: '/pricing', label: 'Contact' },
        ],
      },
      {
        id: 'legal',
        heading: 'Legal',
        links: [
          { href: '/pricing', label: 'Terms' },
          { href: '/pricing', label: 'Privacy' },
          { href: '/pricing', label: 'Security' },
        ],
      },
    ]
    return { footer }
  })
</controller>

<column gap="6">
  <separator />
  <grid min="xs" gap="6">
    <column gap="2">
      <strong>Sail</strong>
      <small>Themes, L1, and a compiler that stay one language.</small>
    </column>
    <for {footer as col} key={col.id}>
      <column gap="2">
        <strong>{col.heading}</strong>
        <for {col.links as link} key={link.label}>
          <a href={link.href}>{link.label}</a>
        </for>
      </column>
    </for>
  </grid>
</column>
