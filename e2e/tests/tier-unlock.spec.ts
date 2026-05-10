// Scenario 4: solving N_UNLOCK (5) tier-1 words unlocks tier 2.
// Walks the full gameplay loop end-to-end: solve current puzzle, tap
// Next, repeat 5×, then return to the main menu and assert the tier-2
// button is no longer `disabled` / `data-locked="true"`.

import { test, expect } from "@playwright/test";
import {
  seedProgress,
  solveCurrentPuzzle,
  tapNext,
  tapPlay,
  waitForApp,
} from "./helpers";

const N_UNLOCK = 5;

test("solving 5 tier-1 words unlocks tier 2 in the main menu", async ({
  page,
}) => {
  test.setTimeout(60_000); // 5 puzzles + transitions

  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null);

  // Tier 2 starts locked.
  const tier2 = page.locator(".betu-tier-button[data-tier='2']");
  await expect(tier2).toHaveAttribute("data-locked", "true");
  await expect(tier2).toBeDisabled();

  // Solve N_UNLOCK puzzles in a row.
  let screen = await tapPlay(page);
  for (let i = 0; i < N_UNLOCK; i++) {
    await solveCurrentPuzzle(page, screen);
    await tapNext(page);
    // Next either rotates to a fresh puzzle (still on .betu-screen) or,
    // if the queue ran dry mid-tier, reshuffles and starts a new word.
    // Either way we stay on the puzzle screen.
    await expect(screen).toHaveAttribute("data-won", "false");
  }

  // Back to main menu.
  await page.locator("[data-testid='puzzle-home']").click();
  await expect(page.locator("[data-testid='menu-title']")).toBeVisible();

  // Tier 2 is now unlocked.
  await expect(tier2).toHaveAttribute("data-locked", "false");
  await expect(tier2).toBeEnabled();
});
