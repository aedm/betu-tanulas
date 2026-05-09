# DESIGN.md

The canonical design for the betu-tanulas project. Decisions here are
locked for v1; later subtasks reference *sections* of this document
instead of re-deciding.

Decisions are derived from research notes in
`brain/letter-game-ux-research.md` and the user's parent spec in
`tasks/betu-tanulas.md` (both in the operator's mind directory).

---

## 1. Product summary

A static, mobile-first web app where a 6-year-old drags shuffled
uppercase letters into the empty slots of a short Hungarian word. The
word is hinted by an emoji (e.g. 🐱 for **CICA**). Wrong drops fly
back; right drops snap and lock; finishing the word advances to the
next one. No ads, no IAP, no telemetry, no fail state, no timer.

Target user: one specific 6-year-old. Optimize for *that* kid's
delight, not for a general audience.

## 2. Tech stack

| Concern | Choice | Rationale |
| --- | --- | --- |
| UI framework | **Dioxus 0.7.x** (latest stable as of 2026-05) | User requirement: "Even frontend: Dioxus and Tailwind". Note: design originally specified 0.6.x; 0.7.9 was the latest at scaffold time so that's what we pinned. |
| Target | **WASM + static HTML/CSS/JS**, served from Cloudflare Pages | Fully client-side, offline-capable after first load, no backend needed for v1. |
| Styling | **Tailwind CSS via the standalone Tailwind CLI** (v4) | Avoids Node toolchain dependency; CI invokes `tailwindcss` once before `dx bundle`. v4 uses CSS-first config (`@import "tailwindcss"` + `@theme`); no `tailwind.config.js`. |
| Build | **`dx bundle --platform web`** produces `dist/public/` | Standard Dioxus output, directly publishable to Pages. |
| Deploy | **Cloudflare Pages**, credentials minted via `aedm/cloudflare-deploy` (`pages-basic` preset) | Token never touches operator session. |
| Persistence | **`localStorage`** under key `betu/progress/v1` | No backend; offline-capable; survives browser restarts. |
| Audio | **HTML `<audio>` with pre-recorded WAV/OGG** for letters + chimes | Pre-recorded by a Hungarian native speaker (the user). No TTS — research flagged TTS phonetic errors as a top failure mode. |
| Tests | **Cargo unit tests** for game logic + **Playwright e2e** on a mobile viewport against the built `dist/`, both in CI | Specifics in a later subtask. |

**Out of pipeline:** no Node-side bundler, no Webpack, no service worker
in v1. The app is a static SPA; CF Pages serves it.

## 3. Drag interaction

| Aspect | Decision | Rationale |
| --- | --- | --- |
| Event model | **Pointer Events** (`pointerdown` / `pointermove` / `pointerup` / `pointercancel`) | Single code path for desktop dev + iOS/Android prod. iOS Safari ≥16 reliable. |
| Snap radius | **40 px** from slot center, first cut | Tune **up** on real device if drops feel finicky, never down. |
| Wrong-drop | **Spring back to origin, ~250 ms ease-out** | Avoid disappear / stay-where-dropped — both confuse the age group. |
| Right-drop | **Snap into slot, lock (no further drag), play soft chime** | "Snap + lock" is the universal well-reviewed pattern. |
| Multi-touch | **Ignore second active pointer** while one is dragging | Toddler apps universally lock to single drag. |
| Tile size | **≥ 64 × 64 CSS px**, ≥ 16 px gap, larger if vertical space allows | Exceeds WCAG AAA 44 px; preschool finger calibration. |
| Slot size | **Same dimensions as tile**, plus a soft drop-zone halo equal to snap radius | Visible affordance for where to aim. |
| Pointer capture | **Use `setPointerCapture` on `pointerdown`** | Avoids losing the drag if the finger leaves the tile element mid-drag. |
| Idle behavior | **After ~10 s of no `pointermove`/`pointerdown`, replay the word's audio** | Keeps the kid from staring at a stuck screen. |

**Hit-test order on `pointerup`:** find the nearest empty slot whose
center is within `snap_radius_px`. If multiple, pick the closest. If
the closest empty slot's expected letter equals the dragged tile's
letter → snap+lock. Else → spring back.

## 4. Word & emoji model

`assets/words.json` lives in the repo. Schema:

```json
[
  { "word": "CICA", "emoji": "🐱", "tier": 1 },
  { "word": "ALMA", "emoji": "🍎", "tier": 2 }
]
```

- `word`: uppercase only, no accents, no digraphs (user hard
  constraint). Each character is one tile.
- `emoji`: a single Unicode emoji that visually represents the word.
- `tier`: positive integer. Tier `n` words have `n + 2` letters by
  convention (tier 1 = 3 letters, tier 2 = 4, tier 3 = 5).

**Final word counts:** 12 tier-1 + 16 tier-2 + 13 tier-3 = 41 entries.
No tier deviations needed. Curation rationale lived in the operator's
mind directory at curation time. The `tests/words_validation.rs` test
enforces every constraint above.

**Letter shuffling.** On entry to a word, the letters are shuffled
into the tile row. Shuffle on every retry of the same word so the
kid can't memorize positions instead of letters. Use a uniform
Fisher–Yates with a non-seeded RNG.

**No distractor letters in v1.** Tile count == word length.

## 5. Progress model

`localStorage` key: `betu/progress/v1` (versioned).

```json
{
  "completed": ["CICA", "ALMA"],
  "currentTier": 1,
  "tierUnlocked": 1
}
```

| Field | Meaning |
| --- | --- |
| `completed` | Words the kid has finished at least once (set semantics; we store as array for JSON, dedupe on write). |
| `currentTier` | The tier the kid most recently played in (used to resume). |
| `tierUnlocked` | The highest tier the kid is allowed to play. Starts at 1; advances by the unlock rule below. |

**No stars in v1.** Mixed evidence; some 6-year-olds re-grind for 3
stars and lose breadth. Reduce to "completed / not completed". Easy
to add later.

**Tier unlock rule.** When `completed ∩ tier-N-words ≥ N_UNLOCK`,
set `tierUnlocked = max(tierUnlocked, N+1)`. **N_UNLOCK = 5** in v1
(verify on the kid before locking in).

**Migration.** The `v1` suffix in the key allows future schema
changes (e.g., adding stars) without losing data: a `v2` writer
reads `v1` and migrates once.

## 6. Visual direction

| Aspect | Decision | Rationale |
| --- | --- | --- |
| Letter case | **Uppercase only** | User hard constraint. |
| Letter font | **Atkinson Hyperlegible** (OFL, Braille Institute, free) | Designed for legibility. Fallback: system sans (`-apple-system, Segoe UI, Roboto, sans-serif`). |
| Color palette | **Light background, high-contrast tiles**: bone-white background `#FBFAF7`, tile fill `#FFFFFF` with `#E5E7EB` border, letter color near-black `#1F2937`. Slot empty state: light grey `#F3F4F6` with dashed border. Success accent: warm green `#34D399`. Error accent: soft amber `#F59E0B` (used only briefly in spring-back). | High contrast for legibility; not migraine-bright. |
| Tile shape | Rounded rectangles (`rounded-2xl`), subtle shadow (`shadow-sm`), grow shadow on `pointerdown` (`shadow-md`) to confirm pickup | Affordance; mirrors well-reviewed apps. |
| Emoji size | The emoji at the top of the screen is the largest single element (~30% of viewport height) — emoji is the celebration. | |
| Layout | Portrait-first; landscape works but is not optimized | The kid will use a phone in portrait. |
| Animation | None — use CSS transitions + Dioxus reactive state | Avoid JS animation deps; spring-back is `transition: transform 250ms ease-out`. |

## 7. Audio direction

Ship audio in v1. Audio is not optional.

| Cue | Asset | Trigger |
| --- | --- | --- |
| Letter phoneme | `assets/audio/letter/<L>.ogg` (one per letter present in any v1 word) | Tile `pointerdown` when picked up (not on snap). |
| Word pronunciation | `assets/audio/word/<WORD>.ogg` | Word completion (after final snap+lock) **and** on slot-area tap (= "repeat instruction"). |
| Success chime | `assets/audio/sfx/chime.ogg` | Word completion. |
| Snap tap | `assets/audio/sfx/snap.ogg` | Each correct snap. |
| Idle replay | reuse `word/<WORD>.ogg` | After ~10 s of no input on a word. |

**Recording responsibility.** The user is Hungarian; recording falls
to them. Until recordings exist, ship a **silent stub** (each audio
file is a 50ms silence) so the codepath is exercised in CI; gate
"shippable to the kid" on real recordings being present.

**Phonetic accuracy.** Each `letter/<L>.ogg` must be the **letter
name** as a Hungarian early-reader would pronounce it (e.g., `A` as
/ɒ/, `M` as /ɛm/, `S` as /ɛʃ/). Spot-check each.

## 8. Level structure

- **Free play within a tier.** The kid picks a tier (from those
  unlocked) and is served words from that tier in random order. A
  word is not repeated until all words in the tier have been seen
  in the current session.
- **Tier unlock.** Per §5, after 5 completions in tier N, tier N+1
  unlocks.
- **No fixed level sequence.** No mandatory order, no "level 1 → 2
  → 3" narrative. Sandbox feel.

## 9. Menus

Reading-free (preschoolers can't read menu copy reliably).

- **Home screen.** Shows the unlocked tiers as big buttons. Each
  tier button uses the emoji of the *easiest word in that tier* as
  its icon (e.g., tier 1 → 🐱). Locked tiers show a dimmed icon
  and a soft padlock; tapping plays a short "not yet" cue and
  bounces, no text dialog.
- **In-game header.** Small "home" icon (🏠) at top-left to return
  to home. No text. Tappable area ≥ 44 × 44 px.
- **No settings screen in v1.** No options the kid would change.
  A long-press on the home icon (≥ 1 s) reveals a small parent
  menu with "reset progress" — not advertised, used by the user
  during testing.

## 10. Out of scope for v1 (explicit non-goals)

These are deliberately deferred. Listing them here so a future run
doesn't quietly add them:

- Multiplayer.
- Custom word entry by the parent.
- Distractor letters (extra letters in the row).
- Lowercase / accented letters / digraphs (user hard constraint).
- Time pressure of any kind.
- Stars / leaderboards / streaks.
- Mascot / persistent character.
- Cartoon-illustrating-the-word video on completion (emoji animation
  is enough; cartoons are art-heavy).
- Service worker / installable PWA.
- Telemetry beyond CF Pages defaults.
- Settings screen / parental controls UI.
- Localization beyond Hungarian.

## 11. Documented trade-offs (do not re-litigate)

Each of these is a deliberate v1 simplification. A future iteration
may revisit; do not silently revisit.

1. **Digraphs excluded.** Hungarian early-readers normally learn CS,
   GY, NY, SZ, TY, ZS as **single letters bound to single
   pictures**. Treating them as two tiles would conflict with how
   the kid is being taught at school. The user's "no digraphs" spec
   is a scope simplification, not pedagogy. A future "digraph as
   single tile" mode would re-align with Hungarian curricula.
2. **No stars.** See §5. Mixed evidence; revisit if the kid asks.
3. **N_UNLOCK = 5.** Guess; verify on the kid.
4. **Snap radius 40 px.** Guess; tune up on device if drops feel
   finicky.
5. **Pre-recorded audio, not TTS.** Phonetic accuracy matters; TTS
   for Hungarian preschool letter-names is not reliably good.

## 12. Data flow / state diagram

```
                  ┌──────────────────┐
                  │  localStorage    │
                  │  betu/progress   │
                  └────────┬─────────┘
                           │ load on app boot
                           ▼
                ┌─────────────────────┐
                │  ProgressStore      │
                │  (Dioxus signal)    │
                └───┬─────────────┬───┘
                    │             │
        unlocked tiers           current word
                    │             │
                    ▼             ▼
              ┌──────────┐   ┌──────────────────┐
              │  Home    │──►│  PuzzleScreen    │
              │  Screen  │   │  ┌─────────────┐ │
              └──────────┘   │  │  Emoji      │ │
                  ▲          │  │  Slots[N]   │ │
                  │          │  │  TileRow[N] │ │  pointer events
            home  │          │  └─────────────┘ │  ───────────────►
            icon  │          │       │          │  drag → snap or
                  │          │       ▼          │  spring-back
                  └──────────┤  WordComplete    │
                             │  (chime + audio  │
                             │   + advance)     │
                             └──────────────────┘
                                     │
                          on each completion
                                     ▼
                           write ProgressStore
                                     │
                                     ▼
                           write localStorage
```

**State ownership.** A single root `AppState` Dioxus signal owns
everything: `progress`, `currentScreen`, `currentWord`, `tiles`,
`slots`. Subcomponents read slices and dispatch events; no global
mutability outside the root.

**Render path.** Pure function of `AppState`. Side effects (audio,
localStorage writes) happen in event handlers, not in render.

## 13. Open questions

None worth blocking on. All v1 decisions are reversible in code.

## 14. Deployment

- **Production URL:** <https://betu-tanulas.pages.dev> — first deploy
  confirmed live 2026-05-09 (HTTP 200, serves Dioxus hello page).
- **Project name:** `betu-tanulas` (Cloudflare Pages).
- **Credentials:** minted via `aedm/cloudflare-deploy` workflow
  `mint-token.yml`, preset `pages-basic`. Re-running the workflow
  revokes prior tokens and is therefore safe.
- **Project bootstrap caveat:** Wrangler 3.x's `pages deploy` does not
  auto-create the project; CI runs `pages project create` first with
  `continue-on-error: true` so re-deploys are idempotent.

## 15. Scaffold deviations from the original design plan

Recorded by `betu-04` so future runs don't re-decide silently:

- **Dioxus 0.7.9** picked instead of the design-doc's 0.6.x — 0.7.9
  was the latest stable at scaffold time and the dioxus-cli already
  installed on the operator's system was 0.7.9.
- **Tailwind CSS v4** (standalone CLI v4.3.0) used instead of v3.
  v4's CSS-first config replaces `tailwind.config.js`; the project
  configures Tailwind via `assets/tailwind.input.css` (`@import
  "tailwindcss"` + `@theme { ... }` + `@source "../src/**/*.rs"`).
- **`assets/tailwind.css` is committed** as a stub so plain `cargo
  build` / `cargo test` work without first running `tailwindcss`.
  CI regenerates it before bundling. The `dioxus::asset!` macro
  resolves the path at compile time, so the file must exist on
  disk during compilation.
- Build out is `dist/public/` (Dioxus 0.7 default), not bare
  `dist/`. The CI deploy step uses `dist/public/` as wrangler's
  source dir.
