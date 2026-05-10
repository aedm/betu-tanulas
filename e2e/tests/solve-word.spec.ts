// Scenario 1: golden-path solve.
// Open `/`, tap play, drag each letter to its slot, assert win celebration
// + progress saved to localStorage.

import { test, expect } from "@playwright/test";
import {
  readProgress,
  seedProgress,
  solveCurrentPuzzle,
  tapNext,
  tapPlay,
  waitForApp,
} from "./helpers";

test("solving a word raises the win overlay and persists completion", async ({
  page,
}) => {
  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null); // fresh storage

  const screen = await tapPlay(page);
  const word = await solveCurrentPuzzle(page, screen);

  // Win artifacts are present.
  await expect(page.locator("[data-testid='betu-next']")).toBeVisible();
  await expect(page.locator(".betu-emoji-rain")).toBeVisible();

  // Persistence kicks in only on Next; tap it.
  await tapNext(page);

  const progress = await readProgress(page);
  expect(progress).not.toBeNull();
  expect(progress!.completed).toContain(word);
});
