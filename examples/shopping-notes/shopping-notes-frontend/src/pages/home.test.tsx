import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Home from "./home";

interface Item {
  id: string;
  name: string;
  bought: boolean;
}

function jsonResponse(body: unknown) {
  return { json: () => Promise.resolve(body) };
}

function mockFetch(initialItems: Item[]) {
  const items = [...initialItems];

  const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    const method = init?.method ?? "GET";

    if (url === "http://localhost:3000/items" && method === "GET") {
      return jsonResponse({ items: [...items] });
    }

    if (url === "http://localhost:3000/items" && method === "POST") {
      const { name } = JSON.parse(init!.body as string) as { name: string };
      const created: Item = { id: `${items.length + 1}`, name, bought: false };
      items.push(created);
      return jsonResponse(created);
    }

    if (url === "http://localhost:3000/items" && method === "DELETE") {
      items.length = 0;
      return jsonResponse({});
    }

    const itemMatch = url.match(/^http:\/\/localhost:3000\/items\/(.+)$/);
    if (itemMatch && method === "PATCH") {
      const { bought } = JSON.parse(init!.body as string) as { bought: boolean };
      const id = itemMatch[1];
      const existing = items.find((item) => item.id === id)!;
      existing.bought = bought;
      return jsonResponse(existing);
    }

    if (itemMatch && method === "DELETE") {
      const id = itemMatch[1];
      const index = items.findIndex((item) => item.id === id);
      items.splice(index, 1);
      return jsonResponse({});
    }

    throw new Error(`Unhandled fetch: ${method} ${url}`);
  });

  vi.stubGlobal("fetch", fetchMock);

  return fetchMock;
}

describe("Home", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("displays each item's name and bought status", async () => {
    // @spec test fe-view-shopping-list#display-item-list
    mockFetch([
      { id: "1", name: "Milk", bought: false },
      { id: "2", name: "Bread", bought: true },
    ]);

    render(<Home />);

    expect(await screen.findByText("Milk")).toBeInTheDocument();
    expect(await screen.findByLabelText("Mark Milk as bought")).not.toBeChecked();
    expect(screen.getByText("Bread")).toBeInTheDocument();
    expect(screen.getByLabelText("Mark Bread as bought")).toBeChecked();
  });

  it("loads the shopping list from the list-items endpoint when the view is opened", async () => {
    // @spec test fe-view-shopping-list#load-list-from-api
    const fetchMock = mockFetch([{ id: "1", name: "Milk", bought: false }]);

    render(<Home />);

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith("http://localhost:3000/items"),
    );
  });

  it("shows an empty state when the shopping list has no items", async () => {
    // @spec test fe-view-shopping-list#empty-state
    mockFetch([]);

    render(<Home />);

    expect(await screen.findByText("No items yet")).toBeInTheDocument();
  });

  it("lets the user enter an item name and submit it to add it to the shopping list", async () => {
    // @spec test fe-add-shopping-item#add-item-input
    mockFetch([]);
    const user = userEvent.setup();

    render(<Home />);
    await screen.findByText("No items yet");

    await user.type(screen.getByPlaceholderText("Add an item"), "Eggs");
    await user.click(screen.getByRole("button", { name: "Add" }));

    expect(await screen.findByText("Eggs")).toBeInTheDocument();
  });

  it("calls the create-item endpoint when the user submits a new item name", async () => {
    // @spec test fe-add-shopping-item#call-create-item-endpoint
    const fetchMock = mockFetch([]);
    const user = userEvent.setup();

    render(<Home />);
    await screen.findByText("No items yet");

    await user.type(screen.getByPlaceholderText("Add an item"), "Eggs");
    await user.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "http://localhost:3000/items",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ name: "Eggs" }),
        }),
      ),
    );
  });

  it("does not submit an add when the input is empty", async () => {
    // @spec test fe-add-shopping-item#reject-empty-submit
    const fetchMock = mockFetch([]);
    const user = userEvent.setup();

    render(<Home />);
    await screen.findByText("No items yet");
    fetchMock.mockClear();

    await user.click(screen.getByRole("button", { name: "Add" }));

    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("clears the input field after the item is added", async () => {
    // @spec test fe-add-shopping-item#clear-input-after-add
    mockFetch([]);
    const user = userEvent.setup();

    render(<Home />);
    await screen.findByText("No items yet");

    const input = screen.getByPlaceholderText("Add an item") as HTMLInputElement;
    await user.type(input, "Eggs");
    await user.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => expect(input.value).toBe(""));
  });

  it("lets the user mark an item as bought from the shopping list with a single action", async () => {
    // @spec test fe-mark-item-bought#mark-bought-action
    mockFetch([{ id: "1", name: "Milk", bought: false }]);
    const user = userEvent.setup();

    render(<Home />);
    const checkbox = await screen.findByLabelText("Mark Milk as bought");

    await user.click(checkbox);

    await waitFor(() => expect(checkbox).toBeChecked());
  });

  it("calls the mark-item-bought endpoint when the user marks an item as bought", async () => {
    // @spec test fe-mark-item-bought#call-mark-bought-endpoint
    const fetchMock = mockFetch([{ id: "1", name: "Milk", bought: false }]);
    const user = userEvent.setup();

    render(<Home />);
    const checkbox = await screen.findByLabelText("Mark Milk as bought");

    await user.click(checkbox);

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "http://localhost:3000/items/1",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({ bought: true }),
        }),
      ),
    );
  });

  it("visually distinguishes bought items from unbought items in the list", async () => {
    // @spec test fe-mark-item-bought#distinguish-bought-items
    mockFetch([
      { id: "1", name: "Milk", bought: false },
      { id: "2", name: "Bread", bought: true },
    ]);

    render(<Home />);
    await screen.findByText("Milk");

    expect(screen.getByText("Bread")).toHaveStyle({ textDecoration: "line-through" });
    expect(screen.getByText("Milk")).not.toHaveStyle({ textDecoration: "line-through" });
  });

  it("lets the user delete an item from the shopping list with a single action", async () => {
    // @spec test fe-delete-shopping-item#delete-item-action
    mockFetch([{ id: "1", name: "Milk", bought: false }]);
    const user = userEvent.setup();

    render(<Home />);
    await screen.findByText("Milk");

    await user.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => expect(screen.queryByText("Milk")).not.toBeInTheDocument());
  });

  it("calls the delete-item endpoint when the user deletes an item", async () => {
    // @spec test fe-delete-shopping-item#call-delete-item-endpoint
    const fetchMock = mockFetch([{ id: "1", name: "Milk", bought: false }]);
    const user = userEvent.setup();

    render(<Home />);
    await screen.findByText("Milk");

    await user.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "http://localhost:3000/items/1",
        expect.objectContaining({ method: "DELETE" }),
      ),
    );
  });

  it("removes the item from the displayed list immediately after it is deleted", async () => {
    // @spec test fe-delete-shopping-item#remove-item-from-view
    mockFetch([
      { id: "1", name: "Milk", bought: false },
      { id: "2", name: "Bread", bought: false },
    ]);
    const user = userEvent.setup();

    render(<Home />);
    await screen.findByText("Milk");

    await user.click(screen.getAllByRole("button", { name: "Delete" })[0]);

    await waitFor(() => expect(screen.queryByText("Milk")).not.toBeInTheDocument());
    expect(screen.getByText("Bread")).toBeInTheDocument();
  });

  it("lets the user clear all items from the shopping list with a single action", async () => {
    // @spec test fe-clear-shopping-list#clear-list-action
    mockFetch([{ id: "1", name: "Milk", bought: false }]);
    vi.spyOn(window, "confirm").mockReturnValue(true);
    const user = userEvent.setup();

    render(<Home />);
    await screen.findByText("Milk");

    await user.click(screen.getByRole("button", { name: "Clear all" }));

    await waitFor(() => expect(screen.queryByText("Milk")).not.toBeInTheDocument());
  });

  it("calls the clear-list endpoint when the user clears the shopping list", async () => {
    // @spec test fe-clear-shopping-list#call-clear-list-endpoint
    const fetchMock = mockFetch([{ id: "1", name: "Milk", bought: false }]);
    vi.spyOn(window, "confirm").mockReturnValue(true);
    const user = userEvent.setup();

    render(<Home />);
    await screen.findByText("Milk");

    await user.click(screen.getByRole("button", { name: "Clear all" }));

    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        "http://localhost:3000/items",
        expect.objectContaining({ method: "DELETE" }),
      ),
    );
  });

  it("asks the user to confirm before clearing all items", async () => {
    // @spec test fe-clear-shopping-list#confirm-before-clearing
    const fetchMock = mockFetch([{ id: "1", name: "Milk", bought: false }]);
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    const user = userEvent.setup();

    render(<Home />);
    await screen.findByText("Milk");
    fetchMock.mockClear();

    await user.click(screen.getByRole("button", { name: "Clear all" }));

    expect(confirmSpy).toHaveBeenCalled();
    expect(fetchMock).not.toHaveBeenCalledWith(
      "http://localhost:3000/items",
      expect.objectContaining({ method: "DELETE" }),
    );
    expect(screen.getByText("Milk")).toBeInTheDocument();
  });
});
