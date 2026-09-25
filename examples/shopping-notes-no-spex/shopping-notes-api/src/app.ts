import Fastify from "fastify";
import cors from "@fastify/cors";

export function createApp() {
  const app = Fastify();

  app.register(cors);

  app.get("/", async () => "hello from api");

  return app;
}
