import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";
import { ordersApi } from "./api";
import type { Order } from "./types";

vi.mock("./api", () => ({
  ordersApi: {
    list: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    remove: vi.fn(),
  },
}));

beforeEach(() => {
  vi.mocked(ordersApi.list).mockResolvedValue([]);
});

describe("App", () => {
  it("shows the empty state when there are no orders", async () => {
    render(<App />);

    expect(await screen.findByText("No orders yet.")).toBeInTheDocument();
  });

  it("adds an order through the form", async () => {
    const created: Order = { id: 1, customer: "Ada", delivered: false };
    vi.mocked(ordersApi.create).mockResolvedValue(created);
    render(<App />);

    await userEvent.type(screen.getByLabelText("Customer name"), "Ada");
    await userEvent.click(screen.getByRole("button", { name: "Order" }));

    expect(ordersApi.create).toHaveBeenCalledWith({ customer: "Ada" });
    expect(await screen.findByText("Ada")).toBeInTheDocument();
  });

  it("shows an error message when loading fails", async () => {
    vi.mocked(ordersApi.list).mockRejectedValue(new Error("API is down"));
    render(<App />);

    await waitFor(() =>
      expect(screen.getByText("API is down")).toBeInTheDocument(),
    );
  });
});
