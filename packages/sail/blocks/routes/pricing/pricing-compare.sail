<scope>
  type CellProps = { value: string }
</scope>

<controller>
  export default controller(() => {
    const compare = [
      { id: 'projects', feature: 'Projects', hobby: '1', studio: 'Unlimited', enterprise: 'Unlimited' },
      { id: 'seats', feature: 'Seats', hobby: '1', studio: '10', enterprise: 'Unlimited' },
      { id: 'studio', feature: 'Theme Studio', hobby: '—', studio: 'check', enterprise: 'check' },
      { id: 'blocks', feature: 'Blocks library', hobby: '—', studio: 'check', enterprise: 'check' },
      { id: 'sso', feature: 'SSO', hobby: '—', studio: '—', enterprise: 'check' },
      { id: 'audit', feature: 'Audit log', hobby: '—', studio: '—', enterprise: 'check' },
      { id: 'support', feature: 'Support', hobby: 'Forum', studio: 'Email', enterprise: 'Dedicated Slack' },
      { id: 'onprem', feature: 'On-prem compiler', hobby: '—', studio: '—', enterprise: 'check' },
    ]
    return { compare }
  })
</controller>

<partial compare-cell props: CellProps>
  <if {value === 'check'}>
    <icon name="check" size="sm"></icon>
  {else =>
    {value}
  }
  </if>
</partial>

<column gap="5">
  <column gap="2" align="center">
    <h2>Compare plans</h2>
    <p>Same compiler. The seats and the catalog change.</p>
  </column>
  <scroll-area orientation="horizontal">
    <table sticky-header bordered>
      <thead>
        <tr>
          <th>Feature</th>
          <th>Hobby</th>
          <th>Studio</th>
          <th>Enterprise</th>
        </tr>
      </thead>
      <tbody>
        <for {compare as row} key={row.id}>
          <tr>
            <td>{row.feature}</td>
            <td><compare-cell value={row.hobby} /></td>
            <td><compare-cell value={row.studio} /></td>
            <td><compare-cell value={row.enterprise} /></td>
          </tr>
        </for>
      </tbody>
    </table>
  </scroll-area>
</column>
