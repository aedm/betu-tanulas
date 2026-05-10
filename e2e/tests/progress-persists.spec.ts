// Scenario 3: progress persists across reload.
// Solve a word, tap Next (which calls progress::save), reload the page,
// open the level-select for tier 1, and assert the just-solved word now
// renders with `data-completed="true"` (its emoji rather than ❓).

import { test, expect } from "@playwright/test";
import {
  seedProgress,
  solveCurrentPuzzle,
  tapNext,
  tapPlay,
  waitForApp,
} from "./helpers";

test("a solved word stays completed after a page reload", async ({ page }) => {
  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null);

  const screen = await tapPlay(page);
  const word = await solveCurrentPuzzle(page, screen);
  await tapNext(page);

  // Reload — progress::load should rehydrate the completion list.
  await page.reload();
  await waitForApp(page);

  // Open tier-1 level-select.
  await page.locator(".betu-tier-button[data-tier='1']").click();
  await expect(
    page.locator(".betu-level-select[data-tier='1']"),
  ).toBeVisible();

  const tile = page.locator(`.betu-word-tile[data-word='${word}']`);
  await expect(tile).toBeVisible();
  await expect(tile).toHaveAttribute("data-completed", "true");
});
