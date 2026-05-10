import { defineConfig, devices } from "@playwright/test";

// The bundled site lives at `../dist/public/` (relative to this file).
// We serve it on port 4173 with Python's stdlib http.server — no extra
// npm dep, ready in well under a second on both macOS and Ubuntu CI.
const PORT = Number(process.env.E2E_PORT ?? 4173);
const BASE_URL = `http://127.0.0.1:${PORT}`;
const DIST_DIR = "../dist/public";

const isCI = !!process.env.CI;

export default defineConfig({
  testDir: "tests",
  fullyParallel: false,
  forbidOnly: isCI,
  retries: isCI ? 1 : 0,
  workers: 1,
  reporter: isCI ? [["list"], ["html", { open: "never" }]] : "list",
  outputDir: "test-results",
  timeout: 30_000,
  expect: { timeout: 5_000 },

  use: {
    baseURL: BASE_URL,
    trace: "retain-on-failure",
    video: "retain-on-failure",
  },

  // iPhone 13 (WebKit, mobile viewport, hasTouch) is the primary target;
  // Pixel 5 (Chromium) covers Android. Firefox skipped per task spec.
  projects: [
    {
      name: "iphone-13-webkit",
      use: { ...devices["iPhone 13"] },
    },
    {
      name: "pixel-5-chromium",
      use: { ...devices["Pixel 5"] },
    },
  ],

  webServer: {
    command: `python3 -m http.server ${PORT} --directory ${DIST_DIR} --bind 127.0.0.1`,
    url: BASE_URL,
    reuseExistingServer: !isCI,
    timeout: 30_000,
    stdout: "ignore",
    stderr: "pipe",
  },
});
