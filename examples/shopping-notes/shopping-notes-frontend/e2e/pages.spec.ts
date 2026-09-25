import { expect, test } from "@playwright/test";

test("home page shows the shopping notes title", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByText("Shopping notes")).toBeVisible();
});

test("api-test page shows the api response", async ({ page }) => {
  await page.goto("/api-test");

  await expect(page.getByText("hello from api")).toBeVisible();
});
