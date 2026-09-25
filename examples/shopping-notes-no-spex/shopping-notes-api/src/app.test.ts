import { describe, expect, it } from "vitest";
import { createApp } from "./app.js";

describe("GET /", () => {
  it("returns 200 with hello from api", async () => {
    const app = createApp();

    const response = await app.inject({ method: "GET", url: "/" });

    expect(response.statusCode).toBe(200);
    expect(response.body).toBe("hello from api");
  });
});
