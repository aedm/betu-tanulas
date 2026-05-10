// Reported by the user on 2026-05-10 from device verification of the kid's
// phone: "Ha rányomok egy betűre, azonnal eltűnik, nem tudom arrébb húzni"
// — "If I press a letter, it immediately disappears, I can't drag it."
//
// A "tap" — pointerdown + pointerup at the same client coordinates, with no
// pointermove in between — must NOT make the tile vanish. The expected
// behavior:
//   1. Tile remains visible (data-placed="false") so the user can try again.
//   2. Tile is no longer in the dragging state at the end.
//   3. wrong_drops does NOT increment — a static tap is not a "wrong drop"
//      attempt; it's the user exploring (often: tapping to hear the letter).
//
// We synthesize the events the same way the helpers already do for drag
// (PointerEvent dispatch via page.evaluate). On both iPhone-13 WebKit and
// Pixel-5 Chromium projects.

import { type Locator, type Page, expect, test } from "@playwright/test";
import { seedProgress, tapPlay, waitForApp } from "./helpers";

async function dispatchPointer(
  loc: Locator,
  type: "pointerdown" | "pointerup",
  client: { x: number; y: number },
  pointerId = 1,
): Promise<void> {
  await loc.evaluate(
    (el, init) => {
      const evt = new PointerEvent(init.type, {
        pointerId: init.pointerId,
        pointerType: "touch",
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
    { type, pointerId, x: client.x, y: client.y },
  );
}

async function centerOf(loc: Locator): Promise<{ x: number; y: number }> {
  const box = await loc.boundingBox();
  if (!box) throw new Error(`element has no bounding box: ${loc}`);
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

async function firstTile(screen: Locator): Promise<Locator> {
  const tile = screen.locator(".betu-tile").first();
  await expect(tile).toBeVisible();
  return tile;
}

test("tapping a tile (no pointermove) keeps it visible and not placed", async ({
  page,
}) => {
  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null);

  const screen = await tapPlay(page);
  const tile = await firstTile(screen);

  // Sanity: tile starts visible and not placed.
  await expect(tile).toHaveAttribute("data-placed", "false");
  await expect(tile).toHaveAttribute("data-dragging", "false");
  await expect(screen).toHaveAttribute("data-wrong-drops", "0");

  const tileIndex = await tile.getAttribute("data-tile-index");
  const tileLetter = (await tile.textContent())?.trim() ?? "";
  const c = await centerOf(tile);

  // The reported scenario: pointerdown then pointerup at the SAME spot,
  // no pointermove. A static tap.
  await dispatchPointer(tile, "pointerdown", c);
  await dispatchPointer(tile, "pointerup", c);

  // The same tile DOM element must still be there, still visible.
  const sameTile = screen.locator(
    `.betu-tile[data-tile-index="${tileIndex}"]`,
  );
  await expect(sameTile).toBeVisible();
  await expect(sameTile).toHaveAttribute("data-placed", "false");
  await expect(sameTile).toHaveAttribute("data-dragging", "false");
  await expect(sameTile).toHaveText(tileLetter);

  // No slot should have been filled by a tap on a tile that's far from
  // any slot center.
  const filled = await screen
    .locator(".betu-slot[data-filled='true']")
    .count();
  expect(filled).toBe(0);

  // A tap is not a wrong-drop attempt. The counter must stay at zero.
  await expect(screen).toHaveAttribute("data-wrong-drops", "0");
});

test("tapping multiple tiles in a row still keeps them all visible", async ({
  page,
}) => {
  await page.goto("/");
  await waitForApp(page);
  await seedProgress(page, null);

  const screen = await tapPlay(page);

  const tiles = screen.locator(".betu-tile");
  const count = await tiles.count();
  expect(count).toBeGreaterThanOrEqual(3);

  for (let i = 0; i < count; i++) {
    const tile = tiles.nth(i);
    const c = await centerOf(tile);
    await dispatchPointer(tile, "pointerdown", c, i + 1);
    await dispatchPointer(tile, "pointerup", c, i + 1);
  }

  // After tapping every tile in turn, none should be placed and the
  // wrong-drops counter must still read 0.
  for (let i = 0; i < count; i++) {
    await expect(tiles.nth(i)).toHaveAttribute("data-placed", "false");
    await expect(tiles.nth(i)).toHaveAttribute("data-dragging", "false");
  }
  await expect(screen).toHaveAttribute("data-wrong-drops", "0");
});
