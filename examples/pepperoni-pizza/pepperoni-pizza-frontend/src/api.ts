import type { Delivery, DeliveryEstimate, Pizza } from "./types";

const BASE_URL = "http://localhost:3000";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE_URL}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...init,
  });

  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new Error(body?.message ?? `Request failed: ${response.status}`);
  }

  return response.json() as Promise<T>;
}

export const pizzaApi = {
  listPizzas: () => request<Pizza[]>("/pizzas"),

  createOrder: (pizzaIds: number[]) =>
    request<Delivery>("/orders", {
      method: "POST",
      body: JSON.stringify({ pizzaIds }),
    }),

  listDeliveries: () => request<Delivery[]>("/deliveries"),

  // @spec impl delivery-time#cart-estimate-display
  estimateOrderDeliveryTime: (pizzaIds: number[]) =>
    request<DeliveryEstimate>("/order-estimate", {
      method: "POST",
      body: JSON.stringify({ pizzaIds }),
    }),
};
