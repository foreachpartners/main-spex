import type { NewOrder, Order, Pizza, UpdateOrder } from "./types";

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

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json() as Promise<T>;
}

export const ordersApi = {
  list: () => request<Order[]>("/orders"),

  create: (order: NewOrder) =>
    request<Order>("/orders", {
      method: "POST",
      body: JSON.stringify(order),
    }),

  update: (id: number, changes: UpdateOrder) =>
    request<Order>(`/orders/${id}`, {
      method: "PUT",
      body: JSON.stringify(changes),
    }),

  remove: (id: number) =>
    request<void>(`/orders/${id}`, {
      method: "DELETE",
    }),
};

// @spec impl BIZ-LIST-PIZZA-MENU
export const pizzasApi = {
  list: () => request<Pizza[]>("/pizzas"),
};
