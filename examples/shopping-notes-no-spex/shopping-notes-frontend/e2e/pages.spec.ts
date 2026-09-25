import { expect, test } from "@playwright/test";

test("home page shows hello from frontend", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByText("hello from frontend")).toBeVisible();
});

test("api-test page shows the api response", async ({ page }) => {
  await page.goto("/api-test");

  await expect(page.getByText("hello from api")).toBeVisible();
});
