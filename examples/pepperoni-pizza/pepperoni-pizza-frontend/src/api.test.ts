import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { pizzaApi } from "./api";
import type { Delivery, DeliveryEstimate, Pizza } from "./types";

function mockFetch(body: unknown, init: { status?: number } = {}) {
  const status = init.status ?? 200;
  const fetchMock = vi.fn().mockResolvedValue({
    ok: status >= 200 && status < 300,
    status,
    json: async () => body,
  } as Response);
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

beforeEach(() => {
  vi.restoreAllMocks();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("pizzaApi", () => {
  it("lists pizzas from GET /pizzas", async () => {
    const pizzas: Pizza[] = [{ id: 1, name: "Margherita", price: 8.5 }];
    const fetchMock = mockFetch(pizzas);

    const result = await pizzaApi.listPizzas();

    expect(result).toEqual(pizzas);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:3000/pizzas",
      expect.objectContaining({ headers: { "Content-Type": "application/json" } }),
    );
  });

  it("creates an order with a POST request and pizza ids", async () => {
    const delivery: Delivery = {
      id: 1,
      items: ["Margherita"],
      estimatedSeconds: 25,
      remainingSeconds: 25,
    };
    const fetchMock = mockFetch(delivery);

    const result = await pizzaApi.createOrder([1]);

    expect(result).toEqual(delivery);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:3000/orders",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ pizzaIds: [1] }),
      }),
    );
  });

  it("lists deliveries from GET /deliveries", async () => {
    const deliveries: Delivery[] = [
      { id: 1, items: ["Margherita"], estimatedSeconds: 25, remainingSeconds: 25 },
    ];
    const fetchMock = mockFetch(deliveries);

    const result = await pizzaApi.listDeliveries();

    expect(result).toEqual(deliveries);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:3000/deliveries",
      expect.anything(),
    );
  });

  // @spec test DELIVERY-TIME
  it("estimates the delivery time with a POST request and pizza ids", async () => {
    const estimate: DeliveryEstimate = { estimatedSeconds: 25 };
    const fetchMock = mockFetch(estimate);

    const result = await pizzaApi.estimateOrderDeliveryTime([1, 2]);

    expect(result).toEqual(estimate);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:3000/order-estimate",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ pizzaIds: [1, 2] }),
      }),
    );
  });

  it("throws the server message when a request fails", async () => {
    mockFetch({ message: "order must contain at least one pizza" }, { status: 400 });

    await expect(pizzaApi.createOrder([])).rejects.toThrow(
      "order must contain at least one pizza",
    );
  });
});
