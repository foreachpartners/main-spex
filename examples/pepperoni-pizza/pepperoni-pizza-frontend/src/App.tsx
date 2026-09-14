import { useEffect, useState } from "react";
import { pizzaApi } from "./api";
import type { Delivery, Pizza } from "./types";

export default function App() {
  const [menu, setMenu] = useState<Pizza[]>([]);
  const [cart, setCart] = useState<Pizza[]>([]);
  const [deliveries, setDeliveries] = useState<Delivery[]>([]);
  const [estimate, setEstimate] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    pizzaApi.listPizzas().then(setMenu).catch(showError);
  }, []);

  // Poll the API every 2 seconds so the delivering window clears itself
  // once the API drops a delivery (when its estimated delivery time elapses).
  // @spec impl delivery-time#in-progress-countdown-display
  useEffect(() => {
    function refresh() {
      pizzaApi.listDeliveries().then(setDeliveries).catch(showError);
    }
    refresh();
    const timer = setInterval(refresh, 2000);
    return () => clearInterval(timer);
  }, []);

  // Fetch the live delivery estimate whenever the cart changes; reset to null
  // once the cart is empty so the cart shows no stale estimate.
  // @spec impl delivery-time#cart-estimate-display
  useEffect(() => {
    if (cart.length === 0) {
      setEstimate(null);
      return;
    }
    pizzaApi
      .estimateOrderDeliveryTime(cart.map((pizza) => pizza.id))
      .then((result) => setEstimate(result.estimatedSeconds))
      .catch(showError);
  }, [cart]);

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
            {estimate !== null && <p>Estimated delivery: {estimate} s</p>}
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
              <li key={delivery.id}>
                {delivery.items.join(", ")} — {delivery.remainingSeconds} s
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}
