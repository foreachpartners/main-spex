import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  // a page is compared with a screenshot of its mockup taken in the same run, kept out of git
  snapshotPathTemplate: "{testDir}/.snapshots/{arg}{ext}",
  // threshold 0 sees a one-digit color change; 40 pixels absorb the 1-unit antialiasing noise (up to about 25) on rounded corners
  expect: { toHaveScreenshot: { threshold: 0, maxDiffPixels: 40 } },
  webServer: [
    {
      command: "npm start",
      url: "http://localhost:5173",
      reuseExistingServer: !process.env.CI,
    },
    {
      command: "npm start",
      cwd: "../shopping-notes-api",
      url: "http://localhost:3000",
      reuseExistingServer: !process.env.CI,
    },
  ],
  use: {
    baseURL: "http://localhost:5173",
    viewport: { width: 1280, height: 720 },
  },
});
