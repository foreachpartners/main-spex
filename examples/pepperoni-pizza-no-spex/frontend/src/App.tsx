import { useEffect, useState } from "react";
import { pizzaApi } from "./api";
import type { Delivery, Pizza } from "./types";

export default function App() {
  const [menu, setMenu] = useState<Pizza[]>([]);
  const [cart, setCart] = useState<Pizza[]>([]);
  const [deliveries, setDeliveries] = useState<Delivery[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    pizzaApi.listPizzas().then(setMenu).catch(showError);
  }, []);

  // Poll the API every 2 seconds so the delivering window clears itself
  // once the API drops a delivery (5 seconds after it was ordered).
  useEffect(() => {
    function refresh() {
      pizzaApi.listDeliveries().then(setDeliveries).catch(showError);
    }
    refresh();
    const timer = setInterval(refresh, 2000);
    return () => clearInterval(timer);
  }, []);

  function showError(err: unknown) {
    setError(err instanceof Error ? err.message : "Something went wrong");
  }

  function addToCart(pizza: Pizza) {
    setCart((current) => [...current, pizza]);
  }

  async function placeOrder() {
    try {
      await pizzaApi.createOrder(cart.map((pizza) => pizza.id));
      setCart([]);
      setDeliveries(await pizzaApi.listDeliveries());
    } catch (err) {
      showError(err);
    }
  }

  return (
    <main>
      <h1>Pepperoni Pizza</h1>

      {error && <p>{error}</p>}

      <section>
        <h2>Menu</h2>
        <ul>
          {menu.map((pizza) => (
            <li key={pizza.id}>
              <span>
                {pizza.name} — ${pizza.price.toFixed(2)}
              </span>
              <button onClick={() => addToCart(pizza)}>Add</button>
            </li>
          ))}
        </ul>
      </section>

      <section>
        <h2>Cart</h2>
        {cart.length === 0 ? (
          <p>Your cart is empty.</p>
        ) : (
          <>
            <ul>
              {cart.map((pizza, index) => (
                <li key={index}>{pizza.name}</li>
              ))}
            </ul>
            <button onClick={placeOrder}>Order</button>
          </>
        )}
      </section>

      <section>
        <h2>Delivering</h2>
        {deliveries.length === 0 ? (
          <p>Nothing is being delivered.</p>
        ) : (
          <ul>
            {deliveries.map((delivery) => (
              <li key={delivery.id}>{delivery.items.join(", ")}</li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}
