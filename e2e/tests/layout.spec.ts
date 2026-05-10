// Scenario 5: mobile-viewport layout sanity at 390×844 (iPhone 13).
//
// The `betu-10` task asked for a visual-snapshot regression. We chose
// **structural** assertions over pixel-diff because:
//   - WebKit/Chromium emoji and font rendering differs sharply between
//     Linux CI and macOS dev machines, so pixel snapshots would either
//     fail constantly or need per-platform baselines maintained by hand.
//   - The actual regressions we worry about (tiles overflowing the
//     viewport, sub-touch-target cells, slot row missing tiles) are
//     measurable in the DOM and far more stable than pixel diffs.
//
// What we assert:
//   1. Puzzle screen + emoji + slot row + tile row all visible.
//   2. Cells (`.betu-cell`) are at least 56 px on the short edge — the
//      floor documented in DESIGN.md §16 (the standard 64-px touch-target
//      goal *can* fall to ~59 px at 375 px wide; 56 is the hard minimum).
//   3. The slot row fits within the visible viewport horizontally (no
//      horizontal scrollbar, no overflow). This is the "regression-y"
//      property: layout should never push tiles off-screen.
//
// Re-baseline only with explicit acknowledgment in PR (per the task).

import { test, expect } from "@playwright/test";
import { seedProgress, tapPlay, waitForApp } from "./helpers";

test("puzzle screen fits the mobile viewport without horizontal overflow", async ({
  page,
}) => {
  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null);

  const screen = await tapPlay(page);

  // Emoji, slot row, tile row all rendered.
  await expect(screen.locator(".betu-emoji")).toBeVisible();
  await expect(screen.locator(".betu-slots")).toBeVisible();
  await expect(screen.locator(".betu-tiles")).toBeVisible();

  // Every cell meets the touch-target floor.
  const cellSizes = await screen
    .locator(".betu-cell")
    .evaluateAll((els) =>
      els.map((el) => {
        const r = el.getBoundingClientRect();
        return { w: r.width, h: r.height };
      }),
    );
  expect(cellSizes.length).toBeGreaterThan(0);
  for (const size of cellSizes) {
    expect(size.w, `cell width ≥ 56 px (got ${size.w})`).toBeGreaterThanOrEqual(
      56,
    );
    expect(
      size.h,
      `cell height ≥ 56 px (got ${size.h})`,
    ).toBeGreaterThanOrEqual(56);
  }

  // No horizontal overflow on the puzzle screen.
  const viewport = page.viewportSize();
  expect(viewport).not.toBeNull();
  const slotsBox = await screen.locator(".betu-slots").boundingBox();
  const tilesBox = await screen.locator(".betu-tiles").boundingBox();
  expect(slotsBox).not.toBeNull();
  expect(tilesBox).not.toBeNull();
  expect(slotsBox!.x).toBeGreaterThanOrEqual(0);
  expect(slotsBox!.x + slotsBox!.width).toBeLessThanOrEqual(viewport!.width);
  expect(tilesBox!.x).toBeGreaterThanOrEqual(0);
  expect(tilesBox!.x + tilesBox!.width).toBeLessThanOrEqual(viewport!.width);

  // No horizontal scrollbar on <html>.
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow, "no horizontal page overflow").toBeLessThanOrEqual(0);
});
