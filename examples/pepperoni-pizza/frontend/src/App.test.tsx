import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { ordersApi, pizzasApi } from "./api";
import type { Order, Pizza } from "./types";

vi.mock("./api", () => ({
  ordersApi: {
    list: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    remove: vi.fn(),
  },
  pizzasApi: {
    list: vi.fn(),
  },
}));

const menu: Pizza[] = [
  { id: "pepperoni", name: "Pepperoni", description: "Spicy classic", price: 12 },
];

beforeEach(() => {
  vi.mocked(ordersApi.list).mockResolvedValue([]);
  vi.mocked(pizzasApi.list).mockResolvedValue(menu);
});

describe("App", () => {
  // @spec test BIZ-LIST-PIZZA-MENU
  it("renders the pizza menu returned by the api", async () => {
    render(<App />);

    expect(await screen.findByText("Pepperoni")).toBeInTheDocument();
    expect(screen.getByText("$12.00")).toBeInTheDocument();
  });

  it("adds an order through the form", async () => {
    const created: Order = { id: 1, customer: "Ada", delivered: false };
    vi.mocked(ordersApi.create).mockResolvedValue(created);
    render(<App />);

    await userEvent.type(
      screen.getByLabelText("Customer name"),
      "Ada",
    );
    await userEvent.click(screen.getByRole("button", { name: "Order" }));

    expect(ordersApi.create).toHaveBeenCalledWith({ customer: "Ada" });
    expect(await screen.findByText("Ada")).toBeInTheDocument();
  });

  it("shows an error message when loading fails", async () => {
    vi.mocked(pizzasApi.list).mockRejectedValue(new Error("API is down"));
    render(<App />);

    await waitFor(() =>
      expect(screen.getByText("API is down")).toBeInTheDocument(),
    );
  });
});
