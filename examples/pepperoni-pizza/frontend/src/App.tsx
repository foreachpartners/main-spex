import { useEffect, useState } from "react";
import { ordersApi, pizzasApi } from "./api";
import type { Order, Pizza } from "./types";

export default function App() {
  const [orders, setOrders] = useState<Order[]>([]);
  const [pizzas, setPizzas] = useState<Pizza[]>([]);
  const [customer, setCustomer] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    ordersApi.list().then(setOrders).catch(showError);
    pizzasApi.list().then(setPizzas).catch(showError);
  }, []);

  function showError(err: unknown) {
    setError(err instanceof Error ? err.message : "Something went wrong");
  }

  async function addOrder(event: React.FormEvent) {
    event.preventDefault();
    const trimmed = customer.trim();
    if (!trimmed) return;

    try {
      const created = await ordersApi.create({ customer: trimmed });
      setOrders((current) => [...current, created]);
      setCustomer("");
    } catch (err) {
      showError(err);
    }
  }

  async function toggleOrder(order: Order) {
    try {
      const updated = await ordersApi.update(order.id, {
        delivered: !order.delivered,
      });
      setOrders((current) =>
        current.map((item) => (item.id === updated.id ? updated : item)),
      );
    } catch (err) {
      showError(err);
    }
  }

  async function removeOrder(id: number) {
    try {
      await ordersApi.remove(id);
      setOrders((current) => current.filter((item) => item.id !== id));
    } catch (err) {
      showError(err);
    }
  }

  return (
    <main>
      <h1>Pepperoni Pizza</h1>

      <form onSubmit={addOrder}>
        <input
          value={customer}
          onChange={(event) => setCustomer(event.target.value)}
          placeholder="Who's ordering a pepperoni pizza?"
          aria-label="Customer name"
        />
        <button type="submit">Order</button>
      </form>

      {error && <p>{error}</p>}

      <ul>
        {orders.map((order) => (
          <li key={order.id}>
            <label>
              <input
                type="checkbox"
                checked={order.delivered}
                onChange={() => toggleOrder(order)}
              />
              <span>{order.customer}</span>
            </label>
            <button
              onClick={() => removeOrder(order.id)}
              aria-label={`Cancel order for ${order.customer}`}
            >
              ✕
            </button>
          </li>
        ))}
      </ul>

      {orders.length === 0 && !error && <p>No orders yet.</p>}

      <h2>Menu</h2>

      <ul>
        {pizzas.map((pizza) => (
          <li key={pizza.id}>
            <div>
              <span>{pizza.name}</span>
              <span>${pizza.price.toFixed(2)}</span>
            </div>
            <p>{pizza.description}</p>
          </li>
        ))}
      </ul>

      {pizzas.length === 0 && !error && (
        <p>The menu is empty right now.</p>
      )}
    </main>
  );
}
