// Scenario 6: after ~10 s of no input on the puzzle screen, the word
// audio replays automatically (DESIGN.md §3 idle replay rule). We can't
// hear audio in headless browsers, but the model bumps a thin DOM
// counter `data-idle-replays` on `.betu-screen` every time it fires —
// that's the e2e read-channel.
//
// Threshold is 10 s on production (`crate::idle::IDLE_REPLAY_THRESHOLD_MS`);
// the wasm timer polls every 1 s, so we wait 12 s to be safely past the
// boundary on slow CI hardware.

import { test, expect } from "@playwright/test";
import { seedProgress, tapPlay, waitForApp } from "./helpers";

const IDLE_WAIT_MS = 12_000;

test("after ~10 s of no input the word audio replays and the counter ticks", async ({
  page,
}) => {
  test.setTimeout(30_000);
  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null);

  const screen = await tapPlay(page);
  await expect(screen).toHaveAttribute("data-idle-replays", "0");

  await page.waitForTimeout(IDLE_WAIT_MS);

  // The counter should have ticked at least once. We don't assert the
  // exact value (CI throttling can fire it twice if the polling cycle
  // straddles two thresholds), but ≥ 1 is the meaningful signal.
  const replays = Number(
    await screen.getAttribute("data-idle-replays"),
  );
  expect(replays).toBeGreaterThanOrEqual(1);
});

test("idle replay is suppressed while the tab is hidden", async ({ page }) => {
  // When the kid's phone goes to sleep or the parent switches apps,
  // `document.hidden` flips to true. On iOS WebKit, an HTMLAudioElement
  // .play() call from a hidden tab can queue and play minutes later
  // when visibility returns — startling, not helpful. The wasm timer
  // checks document.hidden() and short-circuits when hidden.
  test.setTimeout(30_000);
  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null);

  const screen = await tapPlay(page);
  await expect(screen).toHaveAttribute("data-idle-replays", "0");

  // Spoof the Page Visibility API: override `document.hidden` to true
  // and dispatch the visibilitychange event. The wasm callback reads
  // `document.hidden()` once per tick, so subsequent ticks see hidden.
  await page.evaluate(() => {
    Object.defineProperty(document, "hidden", {
      configurable: true,
      get: () => true,
    });
    Object.defineProperty(document, "visibilityState", {
      configurable: true,
      get: () => "hidden",
    });
    document.dispatchEvent(new Event("visibilitychange"));
  });

  // Wait the same 12 s the visible-replay test waits — long enough for
  // multiple polls past the 10 s threshold.
  await page.waitForTimeout(IDLE_WAIT_MS);

  await expect(screen).toHaveAttribute("data-idle-replays", "0");
});

test("a pointer-down on a tile resets the idle clock", async ({ page }) => {
  test.setTimeout(30_000);
  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null);

  const screen = await tapPlay(page);
  await expect(screen).toHaveAttribute("data-idle-replays", "0");

  // Wait 6 s (under threshold), then dispatch a pointerdown on the
  // first tile to reset the clock. After another 6 s of inactivity
  // total wait is 12 s but only 6 s since input — replay must NOT
  // have fired yet.
  await page.waitForTimeout(6_000);
  const tile = screen.locator(".betu-tile").first();
  await tile.dispatchEvent("pointerdown", {
    pointerId: 99,
    pointerType: "touch",
    clientX: 0,
    clientY: 0,
    isPrimary: true,
  });
  // Immediately release the tile so the puzzle isn't stuck mid-drag
  // (the screen-level cancel handler is gated on dragging_tile, so
  // release via pointercancel for safety).
  await screen.dispatchEvent("pointercancel", {
    pointerId: 99,
    pointerType: "touch",
    clientX: 0,
    clientY: 0,
    isPrimary: true,
  });
  await page.waitForTimeout(6_000);

  await expect(screen).toHaveAttribute("data-idle-replays", "0");
});
