import Fastify from "fastify";
import cors from "@fastify/cors";
import { ItemStore } from "./item-store.js";

export interface CreateAppOptions {
  dataFile?: string;
}

export function createApp(options: CreateAppOptions = {}) {
  const app = Fastify();
  const items = new ItemStore(options.dataFile);

  app.register(cors, { methods: ["GET", "HEAD", "POST", "PUT", "PATCH", "DELETE"] });

  app.get("/", async () => "hello from api");

  // @spec impl api-healthcheck#health-endpoint-returns-ok
  // @spec impl api-healthcheck#health-endpoint-no-auth
  // @spec impl api-healthcheck#health-endpoint-no-dependency-checks
  app.get("/health", async () => ({ status: "ok" }));

  // @spec impl api-view-shopping-list#list-items
  // @spec impl api-view-shopping-list#empty-list-when-no-items
  app.get("/items", async () => ({ items: items.list() }));

  app.post("/items", async (request, reply) => {
    const body = request.body as { name?: unknown };

    // @spec impl api-add-shopping-item#reject-empty-name
    if (typeof body?.name !== "string" || body.name.length === 0) {
      reply.code(422);
      return { message: "name must be a non-empty string" };
    }

    // @spec impl api-add-shopping-item#create-item-with-name
    const item = items.create(body.name);
    reply.code(201);
    return item;
  });

  app.delete("/items", async (_request, reply) => {
    // @spec impl api-clear-shopping-list#delete-all-items
    items.clear();
    reply.code(204);
  });

  app.patch("/items/:item_id", async (request, reply) => {
    const { item_id: itemId } = request.params as { item_id: string };
    const { bought } = request.body as { bought: boolean };

    // @spec impl api-mark-item-bought#update-bought-status
    const item = items.update(itemId, bought);
    if (!item) {
      // @spec impl api-mark-item-bought#unknown-item-not-found
      reply.code(404);
      return { message: `item ${itemId} not found` };
    }

    return item;
  });

  app.delete("/items/:item_id", async (request, reply) => {
    const { item_id: itemId } = request.params as { item_id: string };

    // @spec impl api-delete-shopping-item#delete-item-by-id
    const deleted = items.delete(itemId);
    if (!deleted) {
      // @spec impl api-delete-shopping-item#unknown-item-not-found
      reply.code(404);
      return { message: `item ${itemId} not found` };
    }

    reply.code(204);
  });

  return app;
}
