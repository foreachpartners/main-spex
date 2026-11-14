import {
  type BrowserContext,
  expect,
  type Locator,
  type TestInfo,
} from "@playwright/test";
import { mkdir, writeFile } from "node:fs/promises";
import { dirname } from "node:path";

export const mockupUrl = (page: string) =>
  new URL(`../contracts/mockups/pages/${page}.html`, import.meta.url).href;

export const openMockup = async (context: BrowserContext, page: string) => {
  const mockup = await context.newPage();
  await mockup.goto(mockupUrl(page));
  return mockup;
};

// Holds the app to the mockup state rendered in the same run: its structure, then its look
export const expectMatchesMockup = async (
  app: Locator,
  mockupState: Locator,
  name: string,
  testInfo: TestInfo,
) => {
  await expect(app).toMatchAriaSnapshot(await mockupState.ariaSnapshot());

  // toHaveScreenshot reads its expectation from disk, so the mockup state is written there first
  const expectedName = `${name}-${testInfo.repeatEachIndex}.png`;
  const expectedPath = testInfo.snapshotPath(expectedName);
  await mkdir(dirname(expectedPath), { recursive: true });
  await writeFile(
    expectedPath,
    await mockupState.screenshot({
      animations: "disabled",
      caret: "hide",
      scale: "css",
    }),
  );
  await expect(app).toHaveScreenshot(expectedName);
};
