// Scenario 7: tapping anywhere in the slot row plays the word audio
// (DESIGN.md §7 "repeat instruction" cue). Audio playback is silent in
// headless browsers, so we instead read the `data-slot-replays` counter
// the model bumps on each tap. The counter is reset on each new puzzle
// so a per-test count starting at 0 is a reliable invariant.

import { test, expect } from "@playwright/test";
import { seedProgress, tapPlay, waitForApp } from "./helpers";

test("tapping a slot bumps the slot-replay counter without filling it", async ({
  page,
}) => {
  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null);

  const screen = await tapPlay(page);
  await expect(screen).toHaveAttribute("data-slot-replays", "0");

  const firstSlot = screen.locator(".betu-slot[data-slot-index='0']");
  await expect(firstSlot).toHaveAttribute("data-filled", "false");
  await firstSlot.click();

  await expect(screen).toHaveAttribute("data-slot-replays", "1");
  // The slot must remain empty — a tap is read-only.
  await expect(firstSlot).toHaveAttribute("data-filled", "false");

  // Tapping again increments further.
  await firstSlot.click();
  await expect(screen).toHaveAttribute("data-slot-replays", "2");
});
