import { type Page, test } from "@playwright/test";
import { expectMatchesMockup, openMockup } from "./mockup";

const showItems = (
  page: Page,
  items: { id: string; name: string; bought: boolean }[],
) =>
  page.route("http://localhost:3000/items", (route) =>
    route.fulfill({ json: { items } }),
  );

test("not found page matches its mockup", async ({
  page,
  context,
}, testInfo) => {
  await page.goto("/missing-page");

  const mockup = await openMockup(context, "not-found");
  await expectMatchesMockup(
    page.locator(".not-found"),
    mockup.locator('[data-state="default"] .not-found'),
    "not-found-default",
    testInfo,
  );
});

test("empty shopping list matches its mockup", async ({
  page,
  context,
}, testInfo) => {
  await showItems(page, []);
  await page.goto("/");
  await page.getByText("No items yet").waitFor();

  const mockup = await openMockup(context, "shopping-list");
  await expectMatchesMockup(
    page.locator(".shopping-list"),
    mockup.locator('[data-state="empty"] .shopping-list'),
    "shopping-list-empty",
    testInfo,
  );
});

test("shopping list with items matches its mockup", async ({
  page,
  context,
}, testInfo) => {
  await showItems(page, [
    { id: "1", name: "Milk", bought: false },
    { id: "2", name: "Eggs", bought: true },
  ]);
  await page.goto("/");
  await page.getByText("Eggs").waitFor();

  const mockup = await openMockup(context, "shopping-list");
  await expectMatchesMockup(
    page.locator(".shopping-list"),
    mockup.locator('[data-state="with-items"] .shopping-list'),
    "shopping-list-with-items",
    testInfo,
  );
});
