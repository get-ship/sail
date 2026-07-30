<column gap="4" sx={style.pad(6)}>
  <h1>Orders</h1>
  <label for="filter">Filter orders</label>
  <input id="filter" name="filter" placeholder="Filter orders" />
  <table>
    <thead>
      <tr>
        <th>Order</th>
        <th>Customer</th>
        <th>Status</th>
        <th>Total</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>#1001</td>
        <td>Ada Lovelace</td>
        <td>Shipped</td>
        <td>$120.00</td>
      </tr>
      <tr>
        <td>#1002</td>
        <td>Alan Turing</td>
        <td>Pending</td>
        <td>$64.50</td>
      </tr>
    </tbody>
  </table>
</column>
