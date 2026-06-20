import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ordersApi, pizzasApi } from "./api";
import type { Order, Pizza } from "./types";

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

// @spec test BIZ-LIST-PIZZA-MENU
describe("pizzasApi.list", () => {
  it("returns the pizza menu from GET /pizzas", async () => {
    const menu: Pizza[] = [
      { id: "margherita", name: "Margherita", description: "Classic", price: 9.5 },
    ];
    const fetchMock = mockFetch(menu);

    const result = await pizzasApi.list();

    expect(result).toEqual(menu);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:3000/pizzas",
      expect.objectContaining({ headers: { "Content-Type": "application/json" } }),
    );
  });

  it("returns an empty list when the menu has no pizzas", async () => {
    mockFetch([]);

    await expect(pizzasApi.list()).resolves.toEqual([]);
  });
});

describe("ordersApi", () => {
  it("creates an order with a POST request and JSON body", async () => {
    const created: Order = { id: 1, customer: "Ada", delivered: false };
    const fetchMock = mockFetch(created);

    const result = await ordersApi.create({ customer: "Ada" });

    expect(result).toEqual(created);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:3000/orders",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ customer: "Ada" }),
      }),
    );
  });

  it("throws the server message when a request fails", async () => {
    mockFetch({ message: "Customer is required" }, { status: 400 });

    await expect(ordersApi.create({ customer: "" })).rejects.toThrow(
      "Customer is required",
    );
  });
});
