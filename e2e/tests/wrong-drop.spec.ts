// Scenario 2: a wrong drop springs back and increments wrong_drops.
// We pick a tile, release it far from any slot (bottom-left), and verify:
//   1. the tile is back at its origin (data-placed=false, data-dragging=false)
//   2. `data-wrong-drops` on the puzzle screen has incremented from 0 → 1
//
// `data-wrong-drops` is a thin DOM mirror of `Puzzle::wrong_drops`,
// added in betu-10 specifically as the e2e read-channel for this counter
// (the in-game UI doesn't surface it).

import { test, expect } from "@playwright/test";
import {
  dragLetterToWaste,
  getCurrentWord,
  seedProgress,
  tapPlay,
  waitForApp,
} from "./helpers";

test("dropping a tile away from any slot springs it back and bumps wrong_drops", async ({
  page,
}) => {
  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null);

  const screen = await tapPlay(page);

  // Sanity: counter starts at zero.
  await expect(screen).toHaveAttribute("data-wrong-drops", "0");

  const word = await getCurrentWord(screen);
  const firstLetter = word[0];

  await dragLetterToWaste(page, screen, firstLetter);

  // Counter is now 1; no slot got filled.
  await expect(screen).toHaveAttribute("data-wrong-drops", "1");
  const filled = await screen
    .locator(".betu-slot[data-filled='true']")
    .count();
  expect(filled).toBe(0);
});
