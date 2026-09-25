import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { createApp } from "./app.js";

function tempDataFile(): { dataFile: string; dir: string } {
  const dir = mkdtempSync(join(tmpdir(), "shopping-notes-api-test-"));
  return { dataFile: join(dir, "items.json"), dir };
}

describe("GET /", () => {
  it("returns 200 with hello from api", async () => {
    const app = createApp();

    const response = await app.inject({ method: "GET", url: "/" });

    expect(response.statusCode).toBe(200);
    expect(response.body).toBe("hello from api");
  });
});

describe("GET /health", () => {
  // @spec test api-healthcheck#health-endpoint-returns-ok
  it("returns 200 with status ok", async () => {
    const app = createApp();

    const response = await app.inject({ method: "GET", url: "/health" });

    expect(response.statusCode).toBe(200);
    expect(response.json()).toEqual({ status: "ok" });
  });

  // @spec test api-healthcheck#health-endpoint-no-auth
  it("responds without requiring any authorization header", async () => {
    const app = createApp();

    const response = await app.inject({ method: "GET", url: "/health" });

    expect(response.statusCode).toBe(200);
  });

  // @spec test api-healthcheck#health-endpoint-no-dependency-checks
  it("returns only the status field, without downstream dependency data", async () => {
    const app = createApp();

    const response = await app.inject({ method: "GET", url: "/health" });

    expect(Object.keys(response.json())).toEqual(["status"]);
  });
});

describe("GET /items", () => {
  let dataFile: string;
  let dir: string;

  afterEach(() => {
    rmSync(dir, { recursive: true, force: true });
  });

  // @spec test api-view-shopping-list#empty-list-when-no-items
  it("returns an empty list when no items exist", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });

    const response = await app.inject({ method: "GET", url: "/items" });

    expect(response.statusCode).toBe(200);
    expect(response.json()).toEqual({ items: [] });
  });

  // @spec test api-view-shopping-list#list-items
  it("returns every item with its id, name, and bought status", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });
    const created = await app.inject({
      method: "POST",
      url: "/items",
      payload: { name: "Milk" },
    });
    const createdItem = created.json();

    const response = await app.inject({ method: "GET", url: "/items" });

    expect(response.statusCode).toBe(200);
    expect(response.json()).toEqual({
      items: [{ id: createdItem.id, name: "Milk", bought: false }],
    });
  });
});

describe("POST /items", () => {
  let dataFile: string;
  let dir: string;

  afterEach(() => {
    rmSync(dir, { recursive: true, force: true });
  });

  // @spec test api-add-shopping-item#create-item-with-name
  it("creates an item with the given name and bought: false", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });

    const response = await app.inject({
      method: "POST",
      url: "/items",
      payload: { name: "Bread" },
    });

    expect(response.statusCode).toBe(201);
    expect(response.json()).toMatchObject({ name: "Bread", bought: false });
  });

  // @spec test api-add-shopping-item#assign-unique-id
  it("assigns each created item a unique id", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });

    const first = await app.inject({ method: "POST", url: "/items", payload: { name: "Bread" } });
    const second = await app.inject({ method: "POST", url: "/items", payload: { name: "Eggs" } });

    const firstId = first.json().id;
    const secondId = second.json().id;
    expect(firstId).toEqual(expect.any(String));
    expect(firstId.length).toBeGreaterThan(0);
    expect(secondId).not.toBe(firstId);
  });

  // @spec test api-add-shopping-item#persist-created-item
  it("persists the created item to the JSON file store so it survives an API restart", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });
    const created = await app.inject({
      method: "POST",
      url: "/items",
      payload: { name: "Bread" },
    });
    const createdItem = created.json();

    const restarted = createApp({ dataFile });
    const response = await restarted.inject({ method: "GET", url: "/items" });

    expect(response.json()).toEqual({ items: [createdItem] });
  });

  // @spec test api-add-shopping-item#reject-empty-name
  it("does not create an item when name is empty", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });

    const response = await app.inject({
      method: "POST",
      url: "/items",
      payload: { name: "" },
    });

    expect(response.statusCode).not.toBe(201);
    const listResponse = await app.inject({ method: "GET", url: "/items" });
    expect(listResponse.json()).toEqual({ items: [] });
  });

  // @spec test api-add-shopping-item#reject-empty-name
  it("does not create an item when name is missing", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });

    const response = await app.inject({
      method: "POST",
      url: "/items",
      payload: {},
    });

    expect(response.statusCode).not.toBe(201);
    const listResponse = await app.inject({ method: "GET", url: "/items" });
    expect(listResponse.json()).toEqual({ items: [] });
  });
});

describe("PATCH /items/:item_id", () => {
  let dataFile: string;
  let dir: string;

  afterEach(() => {
    rmSync(dir, { recursive: true, force: true });
  });

  // @spec test api-mark-item-bought#update-bought-status
  it("sets the item's bought field to the given value", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });
    const created = await app.inject({ method: "POST", url: "/items", payload: { name: "Milk" } });
    const itemId = created.json().id;

    const response = await app.inject({
      method: "PATCH",
      url: `/items/${itemId}`,
      payload: { bought: true },
    });

    expect(response.statusCode).toBe(200);
    expect(response.json()).toMatchObject({ id: itemId, bought: true });
  });

  // @spec test api-mark-item-bought#persist-bought-status
  it("persists the updated bought status so it survives an API restart", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });
    const created = await app.inject({ method: "POST", url: "/items", payload: { name: "Milk" } });
    const itemId = created.json().id;
    await app.inject({ method: "PATCH", url: `/items/${itemId}`, payload: { bought: true } });

    const restarted = createApp({ dataFile });
    const response = await restarted.inject({ method: "GET", url: "/items" });

    expect(response.json()).toEqual({ items: [{ id: itemId, name: "Milk", bought: true }] });
  });

  // @spec test api-mark-item-bought#unknown-item-not-found
  it("returns 404 when no item matches the id", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });

    const response = await app.inject({
      method: "PATCH",
      url: "/items/does-not-exist",
      payload: { bought: true },
    });

    expect(response.statusCode).toBe(404);
  });
});

describe("DELETE /items/:item_id", () => {
  let dataFile: string;
  let dir: string;

  afterEach(() => {
    rmSync(dir, { recursive: true, force: true });
  });

  // @spec test api-delete-shopping-item#delete-item-by-id
  it("removes the item when called with an existing item id", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });
    const created = await app.inject({ method: "POST", url: "/items", payload: { name: "Milk" } });
    const itemId = created.json().id;

    const response = await app.inject({ method: "DELETE", url: `/items/${itemId}` });

    expect(response.statusCode).toBe(204);
    const listResponse = await app.inject({ method: "GET", url: "/items" });
    expect(listResponse.json()).toEqual({ items: [] });
  });

  // @spec test api-delete-shopping-item#persist-deletion
  it("persists the deletion so the item does not reappear after an API restart", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });
    const created = await app.inject({ method: "POST", url: "/items", payload: { name: "Milk" } });
    const itemId = created.json().id;
    await app.inject({ method: "DELETE", url: `/items/${itemId}` });

    const restarted = createApp({ dataFile });
    const response = await restarted.inject({ method: "GET", url: "/items" });

    expect(response.json()).toEqual({ items: [] });
  });

  // @spec test api-delete-shopping-item#unknown-item-not-found
  it("returns 404 when no item matches the id", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });

    const response = await app.inject({ method: "DELETE", url: "/items/does-not-exist" });

    expect(response.statusCode).toBe(404);
  });
});

describe("DELETE /items", () => {
  let dataFile: string;
  let dir: string;

  afterEach(() => {
    rmSync(dir, { recursive: true, force: true });
  });

  // @spec test api-clear-shopping-list#delete-all-items
  it("deletes every item from the JSON file store", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });
    await app.inject({ method: "POST", url: "/items", payload: { name: "Milk" } });
    await app.inject({ method: "POST", url: "/items", payload: { name: "Bread" } });

    const response = await app.inject({ method: "DELETE", url: "/items" });

    expect(response.statusCode).toBe(204);
    const listResponse = await app.inject({ method: "GET", url: "/items" });
    expect(listResponse.json()).toEqual({ items: [] });
  });

  // @spec test api-clear-shopping-list#persist-cleared-list
  it("persists the empty list so cleared items do not reappear after an API restart", async () => {
    ({ dataFile, dir } = tempDataFile());
    const app = createApp({ dataFile });
    await app.inject({ method: "POST", url: "/items", payload: { name: "Milk" } });
    await app.inject({ method: "DELETE", url: "/items" });

    const restarted = createApp({ dataFile });
    const response = await restarted.inject({ method: "GET", url: "/items" });

    expect(response.json()).toEqual({ items: [] });
  });
});
