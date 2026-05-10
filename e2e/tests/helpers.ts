// Test helpers for the betu-tanulas drag-and-drop puzzle.
//
// The app is a Dioxus wasm SPA that uses Pointer Events with
// setPointerCapture. To exercise drag deterministically across mobile
// projects (iPhone-13 WebKit + Pixel-5 Chromium) we synthesize pointer
// events via Element.dispatchEvent — this hits the same Dioxus handlers
// that real touches do, without depending on touch-vs-mouse routing or
// pointer-capture behavior in headless browsers.

import { type Locator, type Page, expect } from "@playwright/test";

export const STORAGE_KEY = "betu/progress/v1";

/** Wait for the wasm to mount and a known top-level element to render. */
export async function waitForApp(page: Page): Promise<void> {
  await expect(page.locator(".betu-app")).toBeVisible({ timeout: 15_000 });
  // The first screen is always the menu (Game::screen defaults to Menu
  // and only changes after user input).
  await expect(page.locator("[data-testid='menu-title']")).toBeVisible();
}

/**
 * Replace localStorage with the given progress payload (or clear it),
 * then reload so the next page load consumes the seeded state.
 */
export async function seedProgress(
  page: Page,
  payload: object | null,
): Promise<void> {
  await page.evaluate(
    ({ key, value }) => {
      if (value === null) {
        localStorage.removeItem(key);
      } else {
        localStorage.setItem(key, JSON.stringify(value));
      }
    },
    { key: STORAGE_KEY, value: payload },
  );
  await page.reload();
  await waitForApp(page);
}

/** Read the persisted progress JSON from localStorage (or null). */
export async function readProgress(
  page: Page,
): Promise<Record<string, unknown> | null> {
  return await page.evaluate((key) => {
    const raw = localStorage.getItem(key);
    if (!raw) return null;
    try {
      return JSON.parse(raw) as Record<string, unknown>;
    } catch {
      return null;
    }
  }, STORAGE_KEY);
}

/**
 * Tap the play button on the main menu and wait for the puzzle screen
 * to render. Returns the locator for the puzzle `<section>`.
 */
export async function tapPlay(page: Page): Promise<Locator> {
  await page.locator("[data-testid='menu-play']").click();
  const screen = page.locator(".betu-screen");
  await expect(screen).toBeVisible();
  await expect(screen).toHaveAttribute("data-word", /^[A-Z]+$/);
  return screen;
}

/** The currently-displayed word on the puzzle screen, e.g. "ALMA". */
export async function getCurrentWord(screen: Locator): Promise<string> {
  const word = await screen.getAttribute("data-word");
  if (!word) throw new Error("puzzle screen missing data-word");
  return word;
}

/** Center coordinates of an element in viewport space. */
async function centerOf(loc: Locator): Promise<{ x: number; y: number }> {
  const box = await loc.boundingBox();
  if (!box) throw new Error(`element has no bounding box: ${loc}`);
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

interface PointerOpts {
  pointerId?: number;
  pointerType?: "touch" | "mouse" | "pen";
}

/**
 * Dispatch a synthetic PointerEvent on the given locator. We use
 * `dispatchEvent` rather than Playwright's `mouse`/`touchscreen` APIs so
 * the app's onpointerdown/move/up handlers fire identically across
 * browsers and isMobile contexts.
 */
async function dispatchPointer(
  loc: Locator,
  type: "pointerdown" | "pointermove" | "pointerup" | "pointercancel",
  client: { x: number; y: number },
  opts: PointerOpts = {},
): Promise<void> {
  const { pointerId = 1, pointerType = "touch" } = opts;
  await loc.evaluate(
    (el, init) => {
      const evt = new PointerEvent(init.type, {
        pointerId: init.pointerId,
        pointerType: init.pointerType,
        clientX: init.x,
        clientY: init.y,
        isPrimary: true,
        button: 0,
        buttons: init.type === "pointerup" ? 0 : 1,
        bubbles: true,
        cancelable: true,
      });
      el.dispatchEvent(evt);
    },
    { type, pointerId, pointerType, x: client.x, y: client.y },
  );
}

/**
 * Drag the tile that contains `letter` onto the slot at `slotIndex`.
 * Skips tiles that are already placed. Throws if no eligible tile is
 * found (would mean the test setup is wrong, not a flaky failure).
 */
export async function dragLetterToSlot(
  page: Page,
  screen: Locator,
  letter: string,
  slotIndex: number,
  pointerId = 1,
): Promise<void> {
  const tile = screen
    .locator(
      `.betu-tile[data-placed="false"]:has-text("${letter}")`,
    )
    .first();
  await expect(tile).toBeVisible();

  const slot = screen.locator(
    `.betu-slot[data-slot-index="${slotIndex}"]`,
  );
  await expect(slot).toBeVisible();

  const tileCenter = await centerOf(tile);
  const slotCenter = await centerOf(slot);

  await dispatchPointer(tile, "pointerdown", tileCenter, { pointerId });
  // Two moves: first a small jitter so the model leaves the origin
  // pixel-perfectly, then to the slot center for the release hit-test.
  await dispatchPointer(screen, "pointermove",
    { x: tileCenter.x + 4, y: tileCenter.y + 4 },
    { pointerId });
  await dispatchPointer(screen, "pointermove", slotCenter, { pointerId });
  await dispatchPointer(screen, "pointerup", slotCenter, { pointerId });

  // The model snaps synchronously inside the pointerup handler. Wait for
  // the slot to flip to data-filled="true".
  await expect(slot).toHaveAttribute("data-filled", "true");
}

/**
 * Drop a tile somewhere far from any slot — used to test wrong-drop
 * spring-back. Picks the bottom-left of the screen for the release.
 */
export async function dragLetterToWaste(
  page: Page,
  screen: Locator,
  letter: string,
  pointerId = 2,
): Promise<void> {
  const tile = screen
    .locator(
      `.betu-tile[data-placed="false"]:has-text("${letter}")`,
    )
    .first();
  await expect(tile).toBeVisible();

  const tileCenter = await centerOf(tile);
  const screenBox = await screen.boundingBox();
  if (!screenBox) throw new Error("screen has no bounding box");
  const wastePoint = {
    x: screenBox.x + 12,
    y: screenBox.y + screenBox.height - 12,
  };

  await dispatchPointer(tile, "pointerdown", tileCenter, { pointerId });
  await dispatchPointer(screen, "pointermove", wastePoint, { pointerId });
  await dispatchPointer(screen, "pointerup", wastePoint, { pointerId });

  // Tile must spring back to Idle (no longer dragging, not placed).
  await expect(tile).toHaveAttribute("data-dragging", "false");
  await expect(tile).toHaveAttribute("data-placed", "false");
}

/** Solve the puzzle currently shown by dragging each letter to its slot. */
export async function solveCurrentPuzzle(
  page: Page,
  screen: Locator,
): Promise<string> {
  const word = await getCurrentWord(screen);
  for (let i = 0; i < word.length; i++) {
    await dragLetterToSlot(page, screen, word[i], i, i + 1);
  }
  await expect(screen).toHaveAttribute("data-won", "true");
  return word;
}

/** After winning, click ➡️ to advance and persist completion. */
export async function tapNext(page: Page): Promise<void> {
  await page.locator("[data-testid='betu-next']").click();
}
