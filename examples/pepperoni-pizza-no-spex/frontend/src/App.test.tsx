import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { pizzaApi } from "./api";
import type { Delivery, Pizza } from "./types";

vi.mock("./api", () => ({
  pizzaApi: {
    listPizzas: vi.fn(),
    createOrder: vi.fn(),
    listDeliveries: vi.fn(),
  },
}));

const menu: Pizza[] = [
  { id: 1, name: "Margherita", price: 8.5 },
  { id: 2, name: "Pepperoni", price: 9.5 },
  { id: 5, name: "Four Cheese", price: 11.0 },
];

beforeEach(() => {
  vi.mocked(pizzaApi.listPizzas).mockResolvedValue(menu);
  vi.mocked(pizzaApi.listDeliveries).mockResolvedValue([]);
});

describe("App", () => {
  it("shows the pizza menu fetched from the API", async () => {
    render(<App />);

    expect(await screen.findByText(/Margherita/)).toBeInTheDocument();
    expect(screen.getByText(/Four Cheese/)).toBeInTheDocument();
  });

  it("adds a pizza to the cart and reveals the order button", async () => {
    render(<App />);

    const addButtons = await screen.findAllByRole("button", { name: "Add" });
    await userEvent.click(addButtons[0]);

    expect(screen.getByRole("button", { name: "Order" })).toBeInTheDocument();
  });

  it("places an order, clears the cart, and shows the delivery", async () => {
    const delivery: Delivery = { id: 1, items: ["Margherita"] };
    vi.mocked(pizzaApi.createOrder).mockResolvedValue(delivery);
    vi.mocked(pizzaApi.listDeliveries)
      .mockResolvedValueOnce([])
      .mockResolvedValue([delivery]);
    render(<App />);

    const addButtons = await screen.findAllByRole("button", { name: "Add" });
    await userEvent.click(addButtons[0]);
    await userEvent.click(screen.getByRole("button", { name: "Order" }));

    expect(pizzaApi.createOrder).toHaveBeenCalledWith([1]);
    await waitFor(() =>
      expect(screen.getByText("Your cart is empty.")).toBeInTheDocument(),
    );
    expect(await screen.findByText("Margherita")).toBeInTheDocument();
  });

  it("shows an error message when loading fails", async () => {
    vi.mocked(pizzaApi.listPizzas).mockRejectedValue(new Error("API is down"));
    render(<App />);

    await waitFor(() =>
      expect(screen.getByText("API is down")).toBeInTheDocument(),
    );
  });
});
