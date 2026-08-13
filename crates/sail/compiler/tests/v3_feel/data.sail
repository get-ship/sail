<scope>
  style statValue = { type(display-sm) }
  style statLabel = { type(label).fg(muted-foreground) }
  style tableWrap = { size(full).overflow({ x: auto }).border(subtle).radius(lg) }
  style rowHover = { when: { hover: bg(surface-1) } }
</scope>

<grid cols="3" gap="4">
  <card style(pad({ x: 4, y: 3 }))>
    <p style(statLabel)>Deployments</p>
    <p style(statValue)>1,284</p>
    <p style(type(body-sm).fg(success))>+12% this week</p>
  </card>
</grid>

<card>
  <table style(tableWrap)>
    <thead><tr><th>Service</th><th>Status</th><th>Latency</th></tr></thead>
    <tbody>
      <for {services()} as="svc" key={svc.id}>
        <tr style(rowHover)>
          <td>{svc.name}</td>
          <td><badge tone={svc.tone}>{svc.status}</badge></td>
          <td style(type(mono-sm))>{svc.latency}ms</td>
        </tr>
      </for>
    </tbody>
  </table>
</card>
