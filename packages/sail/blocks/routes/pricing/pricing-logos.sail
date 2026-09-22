<controller>
  export default controller(() => {
    const logos = [
      { id: 'harbor', name: 'Harbor' },
      { id: 'north-pier', name: 'North Pier' },
      { id: 'ada-labs', name: 'Ada Labs' },
      { id: 'meridian', name: 'Meridian' },
      { id: 'field', name: 'Field & Co' },
      { id: 'lumen', name: 'Lumen' },
    ]
    return { logos }
  })
</controller>

<column gap="4" align="center">
  <small>Trusted by teams who already shipped</small>
  <row gap="8" align="center" justify="center">
    <for {logos as logo} key={logo.id}>
      <strong>{logo.name}</strong>
    </for>
  </row>
</column>
