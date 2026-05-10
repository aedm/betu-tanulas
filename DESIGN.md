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

## 16. Layout decisions made in `betu-05` (static puzzle screen)

- **Tile/slot sizing.** `width: min(88px, calc((100vw - 80px) / 5))`
  with a `min-width: 56px` floor. Five tier-3 (5-letter) tiles with
  6 px gaps fit a 375 px viewport without horizontal scroll;
  shorter words inherit the same cell size for visual consistency.
  This **deviates from §3's "≥ 64 × 64 CSS px" floor** at 375 px
  (where the formula yields ~59 px) — still well above the 44 px
  WCAG AAA floor and the 44 pt Apple HIG recommendation. On
  ≥440 px-wide phones (iPhone Pro Max, modern Android), the cell
  is back at the spec'd ≥64 px. Revisit in `betu-11` if the kid's
  finger calibration on the actual device finds 56 px too small.
- **Font bundling deferred.** The `@theme` font stack lists
  `"Atkinson Hyperlegible"` first but no font file is bundled
  yet — browsers fall back to `system-ui, sans-serif`. Uppercase
  Hungarian doesn't have the I/l/1 confusion Atkinson is famous
  for solving, so the visual cost is small for v1. Bundle the
  OFL font file in a follow-up if the user wants it before launch.
- **Word selection.** `App` renders the *first* entry in
  `assets/words.json` (currently `APA`). "Configurable" per the
  task is interpreted as "edit `words.json` to put your desired
  word first" — the proper menu lands in `betu-08`.
- **Crate layout.** Promoted from binary-only to lib + bin so
  `tests/puzzle_screen_render.rs` can SSR-render `PuzzleScreen`
  via `dioxus-ssr` without a browser. `src/lib.rs` exposes the
  `App` component; `src/main.rs` is now one line that delegates
  to `betu_tanulas::App`.
- **Shuffle RNG.** Custom xorshift64 (`src/puzzle.rs`) instead of
  pulling `rand` + `getrandom`-with-`js`. Entropy seed comes from
  `js_sys::Date::now()` on wasm and `SystemTime::now()` on native.
  Tests pin `seed = Some(42)` for determinism. If we ever need
  cryptographic-quality randomness elsewhere we'll switch to
  `rand`; for shuffling a 5-letter row, xorshift is more than
  enough.

## 17. Drag mechanics implementation (`betu-06`)

- **State machine.** `Puzzle` owns `tiles: Vec<Tile>` and
  `slots: Vec<Option<usize>>` (slot index → tile index when
  filled). Each `Tile` has a `TileState` of `Idle`,
  `Dragging { pointer_id, pointer, origin_center }`, or
  `Placed { slot_index }`. The state transitions live as plain
  methods on `Puzzle` — `pickup`, `pointer_move`, `release`,
  `cancel` — so they're testable in pure Rust without any DOM.
  The `tests/drag_state_integration.rs` integration test drives
  the same call sequence the pointer-event handlers issue,
  exercising win conditions and wrong-drop accounting end-to-end.
- **Snap rule.** On `release`, the nearest empty slot whose
  center is within `SNAP_RADIUS_PX = 40` is the snap target; if
  the dragged tile's letter equals `word[slot_index]` the tile
  locks (`Placed`). Otherwise — closest slot already filled,
  letter mismatch, or no slot within radius — the tile springs
  back to `Idle` and `wrong_drops` increments. This counts both
  "wrong slot" and "empty space" drops as wrong drops, as the
  spec requires. (Because v1 has no stars, `wrong_drops` has no
  UX consequence yet — it's bookkeeping for `betu-07`.)
- **Multi-touch lock.** `pickup` refuses while any tile is in
  `Dragging`. `pointer_move` and `release` only act on the tile
  matching the supplied `pointer_id`. A second finger landing
  during a drag is silently ignored.
- **Pointer event wiring.** The screen container holds
  `onpointermove` / `onpointerup` / `onpointercancel`; each tile
  holds `onpointerdown`. We additionally call
  `setPointerCapture(pointer_id)` on the tile's `Element` at
  pickup so the events follow the finger even if it leaves the
  tile mid-drag. `touch-action: none` on the screen disables
  browser-handled gestures so move events fire reliably on iOS
  Safari (no scroll-hijacking on the play surface).
- **Visual transform.** The dragged tile gets an inline
  `transform: translate(dx, dy)` where
  `(dx, dy) = pointer − origin_center` (origin_center is the
  tile's center at pickup, measured via
  `getBoundingClientRect()`). `transition: none` keeps the tile
  glued to the finger while dragging; on release, the inline
  style strips, and the base `.betu-tile` rule's
  `transition: transform 250ms ease-out` provides the spring-back.
  `data-dragging="true"` adds a softly elevated shadow.
- **Slot affordance.** While a drag is live, every empty slot
  whose expected letter matches the dragged tile's letter gets
  `data-target="true"`; the CSS paints a soft success-tinted
  border. Default ON for v1 (research §1) — flip the
  `target_for_drag` predicate to `false` to disable if the
  hint feels too generous on the kid.
- **Slot-center measurement.** At release time we run
  `document.querySelectorAll(".betu-slot")`, read each
  `getBoundingClientRect()` center, and key the resulting array
  by the `data-slot-index` attribute so misordered DOM nodes
  can't desync the array from the model. This avoids the
  ref-tracking dance Dioxus would otherwise need.
- **Real-device verification deferred.** The state machine and
  Tailwind visuals are exercised by 28 in-process tests
  (puzzle unit + SSR + drag integration) plus a desktop browser
  via `dx serve`. Touch-device verification on real iOS / Android
  hardware is the explicit job of `betu-11-real-device-polish.md`
  per the parent plan; this run does *not* claim hardware
  verification.

## 18. Win flow + progression (`betu-07`)

- **`Game` struct** in `src/game.rs` is the new top-level model:
  it owns the dictionary (`Vec<Word>`), the persisted `Progress`,
  the `current_tier`, a shuffled `queue: Vec<Word>` of upcoming
  tier words, and the active `Puzzle`. The puzzle screen reads &
  mutates `Game` via a single `Signal<Game>` rooted in `App` —
  per §12 "single root `AppState` Dioxus signal owns everything".
- **Win detection** is `Game::is_won() = current_puzzle.is_complete()`.
  The PuzzleScreen renders `data-won="true"` on `.betu-screen`,
  which switches in the win-flow visuals via pure CSS (no JS
  timers).
- **Win-flow visuals** (CSS only, in `assets/tailwind.input.css`):
  - 0–600 ms × 2: every slot pulses softly with a green halo
    (`@keyframes betu-pulse` + `animation: 600ms ease-in-out 2`).
    The completed word is readable in solved form throughout.
  - 0–1500 ms: 10 emoji "raindrops" fall from the top of the
    screen, staggered 80 ms apart (`@keyframes betu-rain` on
    `.betu-rain-drop` with `--i` index per drop). The drop emoji
    is the word's own emoji — "the emoji is the celebration" (§6).
  - 800 ms onward: a large round Next button (`➡️`) fades in at
    the bottom-center (`@keyframes betu-next-fadein` runs
    *after* the pulse + most of the rain, so the button
    materializes once the celebration has settled). It is
    `pointer-events: none` until the fade-in completes, so an
    eager finger can't dismiss the celebration prematurely.
- **Manual tap to continue.** No auto-advance — research §3
  flagged auto-advance as feeling rushed; the kid taps Next when
  ready. `betu-11` may revisit if the kid's pattern says
  otherwise.
- **Tile pickup is gated on `won`.** While in the win state,
  `onpointerdown` / `onpointermove` / `onpointerup` early-return
  so already-placed tiles can't accidentally re-trigger drag
  state.
- **`advance_to_next` semantics** (`Game::advance_to_next`):
  1. If `is_won()`, record the current word into
     `progress.completed` (deduped) and recompute
     `progress.tier_unlocked` against the dictionary.
  2. If `queue` is empty, rebuild it from the tier's full word
     list and shuffle (Fisher–Yates with `XorShift64`). This
     gives §8's "free play within a tier — not repeated until
     all words have been seen".
  3. Pop the next word and build a fresh `Puzzle` for it. The
     active `current_puzzle` is replaced; tile state resets to
     all-`Idle`.
  4. Persist via `progress::save(&progress)` — wasm hits
     `localStorage["betu/progress/v1"]`, native is a no-op.
- **Persistence schema** (`src/progress.rs`):
  ```json
  {
    "completed": ["CICA", "ALMA"],
    "currentTier": 1,
    "tierUnlocked": 2
  }
  ```
  Serde uses `#[serde(rename = "currentTier" / "tierUnlocked")]`
  to match the camelCase already documented in §5. Unknown /
  malformed JSON → `Progress::default()` (kid restarts from
  scratch — fail-soft, no flash of an error screen).
  `Game::new` clamps `current_tier` into `[1, tier_unlocked]`
  defensively.
- **Tier unlock rule.** `Progress::recompute_tier_unlock(&words)`
  walks every tier `N` in the dictionary; if `>= N_UNLOCK = 5`
  words from tier `N` appear in `completed`, set
  `tier_unlocked = max(tier_unlocked, N + 1)`. Monotonic — never
  re-locks. This is §5's rule promoted into a tested function.
  N_UNLOCK is a guess per §11(3); revisit during `betu-11`.
- **Tier change from menus deferred** to `betu-08`. For now the
  kid plays in `progress.current_tier` and stays there;
  `advance_to_next` does not auto-jump into a newly-unlocked
  tier. The unlock notification UX lives in `betu-08`'s home
  screen, not here.
- **Tests.** `src/progress.rs` (7 unit tests on JSON round-trip,
  dedup, unlock rule, monotonicity, malformed-JSON fallback);
  `src/game.rs` (10 unit tests on tier clamping, win transition,
  word rotation without repeats, completion recording, 5×
  tier-unlock); `tests/game_flow_integration.rs` (4 integration
  tests driving the same `Puzzle::pickup → release` sequence the
  pointer handlers issue, against the real `words.json`,
  including a save/load round-trip). New SSR test verifies the
  un-won screen renders without the Next button or confetti.
- **Audio + chime** still deferred to `betu-09` per parent plan.
  The success chime is referenced in §7 but not wired up in
  `betu-07`; today the win flow is silent.

## 19. Menus + navigation + in-game progress (`betu-08`)

The kid drives navigation entirely with icons. Three flat screens —
`Screen::{Menu, LevelSelect, Puzzle}` — and the kid is never deeper
than two taps from the menu.

- **`Screen` enum** in `src/screen.rs`. Game owns `screen: Screen`;
  `App` matches on it and renders `MainMenu`, `LevelSelect`, or
  `PuzzleScreen` accordingly. State lives in the existing single
  `Signal<Game>` rooted in `App` — no parallel router state.
- **`MainMenu`** (`src/menu.rs`):
  - Big play button (▶️) → `Game::resume_play()` switches to
    `Screen::Puzzle` without disturbing the active puzzle.
  - One tier button per tier in the dictionary (3 today). The
    button's icon is the tier's first-word emoji per §9. Locked
    tiers carry `data-locked="true"` + 🔒 + `disabled` attribute
    + 0.45 opacity + grayscale, so the kid sees them but cannot
    activate.
  - Hidden parent zone: triple-tap on the title opens a confirm
    dialog with `Igen`/`Mégse` buttons. Confirm calls
    `Game::reset_progress()` + `progress::save()`. Triple-tap +
    confirm is rare-enough-by-design to be child-safe; replaces
    the long-press-on-home-icon idea from §9 because long-press
    is harder to implement reliably across iOS/Android.
- **`LevelSelect`** (`src/level_select.rs`):
  - Header: ⬅️ back button → `Game::go_to_menu()`; tier label;
    a spacer to balance the grid.
  - Grid of word tiles, one per word in `Game::words_in_tier(tier)`.
    Completed words show their emoji; never-completed words show ❓
    so the kid keeps a sense of mystery (DESIGN §9 / betu-08 spec).
  - Tap → `Game::start_word(word)` builds a fresh `Puzzle` for
    that word, populates the queue with the tier's other words
    (so post-win Next still rotates correctly), and switches to
    `Screen::Puzzle`.
- **`PuzzleScreen` additions** (`src/puzzle_screen.rs`):
  - Top-left 🏠 home icon → `Game::go_to_menu()`. Going home does
    *not* reset the puzzle — the kid can resume.
  - Top-right tiny progress chip: `<tier> · <done>/<total>` for
    the current tier. `done` is the count of completed words in
    that tier; `total` is dictionary size. Pill-shaped, dimmed —
    "don't make it a billboard" (betu-08 spec).
- **`Game` API** (`src/game.rs`):
  - `enter_tier(tier)` — no-op when locked or out of range; sets
    `Screen::LevelSelect { tier }`.
  - `start_word(&str)` — looks up word; if its tier is locked,
    no-op; otherwise rebuilds tier queue (excluding the chosen
    word), creates a fresh `Puzzle`, switches to `Screen::Puzzle`.
  - `resume_play()` — switches to `Screen::Puzzle` (puzzle
    survives re-entries to menu).
  - `go_to_menu()` — switches to `Screen::Menu`.
  - `reset_progress()` — wipes `Progress`, rewinds to a tier-1
    puzzle, returns to `Screen::Menu`. Caller persists.
  - `words_in_tier(tier) -> Vec<&Word>`, `is_completed(&str) -> bool`
    are read-only helpers used by both UI and tests.
- **Localization shim** (`src/i18n.rs`): tiny `translate(&'static str)
  -> &'static str` matched against literal keys (e.g. `"menu.play"`,
  `"puzzle.next"`). Macro `t!("…")` enforces literal keys at the
  call site so a typo is a compile error rather than a fallback
  string at runtime. v1 is Hungarian-only; future locales drop in
  by extending the match arms — no call-site changes.
- **Tests added (33 new, 87 total)**:
  - 11 unit tests in `game.rs` covering `enter_tier` (locked/zero/
    happy-path), `start_word` (locked-tier refusal), `resume_play`,
    `go_to_menu`, `reset_progress`, `is_completed`, `words_in_tier`,
    and the new `Screen::Menu` default.
  - 3 unit tests in `i18n.rs` (known/unknown keys + macro).
  - `tests/menu_render.rs` (5 SSR tests) — play button + 3 tier
    buttons render, locked-tier `data-locked="true"` count, title
    localized, reset dialog hidden.
  - `tests/level_select_render.rs` (5 SSR tests) — one tile per
    tier word; uncompleted = ❓; completed = real emoji +
    `data-completed="true"`; back button localized; tier-2 grid
    when unlocked.
  - `tests/navigation_flow.rs` (8 integration tests) — Menu →
    LevelSelect → Puzzle → Menu state machine plus reset path
    against the real dictionary.
  - 1 SSR test added to `puzzle_screen_render.rs` for the in-game
    home icon + progress chip.
- **What's deferred to later subtasks:**
  - Audio cues for Play/locked-tier-tap remain in `betu-09`.
  - Real-device feel (long-press timings, tap-target tuning) in
    `betu-11`.
  - The original §9 "long-press parent menu on home icon" is
    *replaced* by the title triple-tap in v1; revisit if user
    objects.

## 20. Audio + parent-zone volume (`betu-09`)

Implements §7 with the smallest viable wiring: stateless cue helpers,
silent-stub assets for letters/words, synthesized SFX, persisted
volume.

### Module layout

- `src/audio.rs` — public stateless helpers `play_letter / play_word
  / play_snap / play_chime`. Each takes a `volume: u32` (0..=100)
  read from `Game.progress.volume` at call time. URL helpers
  (`letter_url`, `word_url`, `SNAP_URL`, `CHIME_URL`) are pure
  functions — testable on native.
- Wasm path uses `web_sys::HtmlAudioElement::new_with_src` →
  `set_volume(volume_to_unit(v))` → `play()`. The element is
  transient; the browser keeps it alive until playback finishes,
  then it gets GC'd. No element pool, no preloading complexity.
- Native (test) path: every play call is a `_url, _volume` no-op
  with the same signature so tests run without a browser.
- `volume == 0` short-circuits the wasm path before creating the
  `<audio>` element.

### Asset pipeline

- Stubs: `assets/audio/letter/<L>.wav` (26) and
  `assets/audio/word/<WORD>.wav` (41) are 80 ms of silence at
  16 kHz mono 16-bit (~1.3 KB each). Total ~100 KB. The user
  records over them in-place when ready (DESIGN §7's "recording
  responsibility on user").
- SFX: `assets/audio/sfx/snap.wav` (60 ms decaying noise burst)
  and `assets/audio/sfx/chime.wav` (C5–E5–G5 sine arpeggio with
  exponential decay, ~600 ms). Both synthesized programmatically;
  no third-party samples.
- Generator: `tools/gen_audio.py` reads `assets/words.json`,
  writes all of the above. Idempotent (deterministic snap RNG).
  Run via `make audio` after editing words or when bootstrapping
  a fresh checkout.
- Bundle copy: `dx bundle` does not pick up unreferenced files.
  `make bundle` and `.github/workflows/ci.yml`'s `bundle` job
  both copy `assets/audio/` → `dist/public/audio/` after the
  Dioxus bundle step. CI verifies the SFX files are present
  before uploading the artifact.
- Attribution: `assets/audio/ATTRIBUTIONS.md` records every
  shipped file's source and license. Synthesized files = our
  own; silent stubs = placeholders for future user recordings.

### Cue triggers (mapped from §7)

| Cue | Trigger in code | Helper |
| --- | --- | --- |
| Letter phoneme | `onpointerdown` after `Puzzle::pickup` returns true | `audio::play_letter(c, v)` |
| Snap into slot | `onpointerup` when `Puzzle::release` yields `Snapped` | `audio::play_snap(v)` |
| Win chime | `onpointerup` when `Game::is_won()` becomes true | `audio::play_chime(v)` |
| Word pronunciation | Same release path as the chime, fired alongside | `audio::play_word(word, v)` |
| Wrong drop | (no cue) | — |

The win-flow chime + word fire from the same release handler so
they begin in the same audio frame as the snap — the browser
sequences them by element. Silence on wrong drops is per `betu-09`
spec ("don't punish with a buzz") and §7.

§7's "slot tap to repeat instruction" and §3's "idle replay after
~10 s" are deferred — neither blocks the kid playing v1; both
belong to a later device-polish iteration that can tune timings
on real hardware.

### Volume persistence (parent zone)

- `Progress.volume: u32` (0..=100) defaults to 70 on first launch.
- `#[serde(default = "default_volume")]` makes pre-`betu-09`
  saves still parse — old `localStorage["betu/progress/v1"]`
  payloads load with `volume = VOLUME_DEFAULT`.
- The parent dialog (still gated behind a triple-tap on the menu
  title) is now a small panel:
  - A `<input type="range" min=0 max=100 step=1>` slider that
    updates `Progress.volume` on every `oninput` and immediately
    calls `progress::save` so a closed dialog still persists the
    new setting.
  - A primary "Reset progress" button that opens an in-modal
    Igen / Mégse confirm pair before destroying state. The
    confirm UI now lives inside the parent dialog rather than a
    separate "Are you sure?" modal.
  - A "Bezár" (close) button that dismisses the dialog without
    touching state.
- The slider does not duplicate volume into a separate signal —
  the source of truth is `Progress.volume`, which already
  round-trips through `progress::save / load`. A test
  (`game.rs::volume_change_round_trips_through_progress_json`)
  pins this contract.

### iOS Safari notes

- HTMLAudioElement requires a user gesture before the first
  `.play()` resolves on iOS. The kid's first tap on a tile is
  the gesture, so we get this for free; cold launches with no
  interaction simply queue silence (Promise rejection ignored).
- WAV is reliably playable on both iOS Safari and Android
  Chrome. We ship WAV not OGG/MP3 because synthesizing WAV from
  scratch needs no external tooling; size cost (~200 KB total)
  is fine for a static SPA. The user can replace stub WAVs
  with real recordings in any browser-supported codec —
  the URL convention (`<L>.wav`) is the only constraint.

### Tests

- 9 unit tests in `src/audio.rs` (URL formats + volume clamping +
  native no-op smoke).
- 3 integration tests in `tests/audio_assets.rs` proving every
  letter / word in the dictionary has a stub on disk and the
  two SFX exist + non-trivially sized.
- 5 SSR tests in `tests/parent_dialog_render.rs` covering the
  rendered slider markup, default state buttons, and localized
  caption.
- `tests/menu_render.rs` updated to assert the slider + dialog
  are *not* in the initial menu render (i.e., the triple-tap
  gate still works).
- 1 unit test in `src/game.rs` proving volume round-trips
  through `Progress` JSON serialization.
- 1 unit test in `src/progress.rs` proving a pre-`betu-09`
  legacy save (no volume field) loads with the default volume.

108 tests total (87 from `betu-08` + 21 new).

### What's deferred to `betu-11` device polish

- Real-device verification (audio plays, doesn't double, doesn't
  echo, no iOS muted-switch gotchas).
- Idle-replay of the word audio after ~10 s of no progress (§3).
- Slot-tap "repeat instruction" cue (§7).
- Tuning chime timbre / volume against a real living-room
  listening test.

## 21. End-to-end tests (`betu-10`)

Playwright suite under `e2e/`, run against the production bundle
(`dist/public/`) via `python3 -m http.server`. Two projects: WebKit
(iPhone 13 viewport, hasTouch) and Chromium (Pixel 5). Firefox is
skipped — not the audience. Five scenarios:

1. **Solve a word, golden path** (`solve-word.spec.ts`) — drag every
   letter to its slot, assert win celebration + Next button +
   `localStorage["betu/progress/v1"].completed` contains the word.
2. **Wrong drop springs back** (`wrong-drop.spec.ts`) — drop a tile in
   the bottom-left waste area; assert the tile returns to Idle and
   `data-wrong-drops` increments from `0` → `1`.
3. **Progress persists across reload** (`progress-persists.spec.ts`) —
   solve, tap Next, reload, open level-select for tier 1, assert the
   word tile renders with `data-completed="true"`.
4. **Tier unlock** (`tier-unlock.spec.ts`) — solve five tier-1 words
   in a row, return to menu, assert the tier-2 button is no longer
   `disabled` / `data-locked="true"`.
5. **Mobile-viewport layout** (`layout.spec.ts`) — structural checks
   at the iPhone-13 viewport: every `.betu-cell` is ≥ 56 px on each
   edge (DESIGN §16 floor), neither the slot row nor the tile row
   horizontally overflows the viewport, and `<html>` has no
   horizontal scroll. **Deviation from the original `betu-10` task:**
   we chose structural assertions over pixel-diff snapshots because
   WebKit/Chromium emoji + font rendering differs sharply between
   Linux CI and macOS dev machines, which would force per-platform
   baselines maintained by hand. Structural checks catch the actual
   regression we worry about — tiles or slots clipping off the
   viewport — without that maintenance cost.

### Drag synthesis

Headless browsers' touch and mouse APIs disagree on Pointer Events
across `isMobile=true` projects. To exercise the same code path
real fingers do, we dispatch synthetic `PointerEvent`s via
`Element.dispatchEvent` (`pointerdown` on the tile, `pointermove`
+ `pointerup` on the screen container). This hits Dioxus's
`onpointer*` handlers directly and works identically on WebKit and
Chromium. See `e2e/tests/helpers.ts` for the `dispatchPointer`
helper. Pointer capture (`setPointerCapture`) is not required for
correctness because the move/up handlers are wired on the screen,
not the tile.

### Test e2e read-channels

The puzzle screen exposes:

- `data-word="<UPPERCASE>"` — current word (already used by SSR
  tests; e2e reads it to discover what to solve).
- `data-won="true|false"` — win-state mirror.
- `data-wrong-drops="<N>"` — counter mirror, added in `betu-10`
  specifically for scenario 2 (the in-game UI never shows the
  counter).

Stable `data-testid` hooks already in place from `betu-08`/`-09`
(`menu-title`, `menu-play`, `puzzle-home`, `puzzle-progress`,
`betu-next`, `parent-dialog`, `volume-slider`, `level-select-back`)
are reused by the e2e tests.

### CI integration

A new `e2e` job in `ci.yml` runs after `bundle`, downloads the
`dist` artifact, sets up Node 20, restores a Playwright browser
cache (key `playwright-${os}-1.49.1`), runs both projects, and on
failure uploads the Playwright report + traces and posts the
diag-tail comment to the PR (per the diag pattern in
`brain/cloudflare-deploy-vault.md`). The `deploy` job now depends
on `[bundle, e2e]` so a failing e2e blocks production.

Runtime budget: ~30 s for the full suite (10 tests × 2 projects)
on Apple Silicon; CI sees similar with cache hit, ~90 s on cold
cache including browser download. Comfortably under the 2-minute
target.

## 22. Idle replay + slot-tap repeat-instruction (`betu-11`, code-only slice)

`betu-11` is mostly hardware verification (still pending — needs the
user's child's phone). Two items from §3 and §7 were *blind code work*
and landed here without device access.

### §3 idle replay

After 10 s of no `pointerdown` / `pointermove` / `pointerup` /
`pointercancel` / slot-tap on the puzzle screen, the current word's
audio replays. The kid hears the whole word reminded of what they're
solving without having to ask. Tapping or dragging anything resets the
clock; the next replay only fires after another full 10 s of idleness.

### §7 slot-tap "repeat instruction"

Tapping any cell in the slot row (filled or empty) plays the current
word's audio. This is the kid's explicit "say it again" gesture.
Implemented as `onclick` on each `.betu-slot`, gated on
`!is_won() && dragging_tile().is_none()` so a drag-release-on-slot
doesn't double-trigger.

### Implementation

- New module `src/idle.rs` holds `IdleReplay { last_input_ms,
  idle_replays, slot_replays }`. Methods take wall-clock timestamps
  explicitly (`note_input(now)`, `should_replay(now, threshold)`,
  `note_replay(now)`, `note_slot_tap(now)`) so the state machine is
  fully unit-testable on native — no DOM, no `Date::now`.
- Threshold is `IDLE_REPLAY_THRESHOLD_MS = 10_000.0`.
- In `puzzle_screen.rs`, the screen owns `Signal<IdleReplay>` seeded
  with the current wall-clock at first render. Pointer handlers, slot
  clicks, the home icon, and the Next button all bump
  `note_input(now_ms())`. `note_input` is deliberately *not* called
  from the bare `pointermove` path when no drag is active — that
  path already early-returns to avoid 60 Hz re-renders, and a hover
  without contact isn't a meaningful "wakefulness" signal on touch
  devices.
- Wasm-only: `install_idle_replay_timer` registers a 1 s
  `setInterval` on mount and clears it on unmount via
  `dioxus::core::use_hook_with_cleanup`. On each tick it consults
  `should_replay` + `Game::is_won()` + `dragging_tile()`, plays the
  word audio, and calls `note_replay` (which both bumps the counter
  and re-arms the clock).
- The interval is *always* alive while the puzzle screen is mounted;
  navigating to the menu unmounts the screen and `clear_interval`
  fires from the cleanup callback. No leaked timers, no audio plays
  while the kid is on the menu.
- **Page Visibility suppression.** Each tick consults
  `document.hidden` (Page Visibility API) via a small wasm helper
  and short-circuits when the tab is backgrounded. The kid isn't
  watching anyway, and on iOS WebKit an `HTMLAudioElement.play()`
  from a hidden tab can queue and play later when visibility
  returns — startling for a parent pulling the phone out of a
  pocket. The model exposes `IdleReplay::should_fire_replay(now,
  threshold, hidden)` so the suppression logic is unit-testable on
  native (no DOM stub needed); the wasm wiring is one
  `web_sys::Document::hidden()` read.

### E2E read-channels

Two new attributes on `.betu-screen`:

- `data-idle-replays="<N>"` — counter, bumped by the timer when it
  fires the replay.
- `data-slot-replays="<N>"` — counter, bumped on slot-tap.

The headless e2e suite can't *hear* audio but uses these counters
plus the existing `audio/word/<WORD>.wav` request log (visible in
the `python3 -m http.server` access lines) to verify both behaviors.

### Tests

- 8 unit tests in `src/idle.rs` cover boundary, reset, replay rearm,
  slot-tap counter independence, multi-replay sequence, plus the two
  visibility-suppression cases (hidden never fires; visibility-only
  predicate doesn't mutate state).
- 1 SSR test in `tests/puzzle_screen_render.rs` asserts both new
  data attributes start at `0`.
- 4 e2e specs:
  - `idle-replay.spec.ts` — wait 12 s, assert counter ≥ 1.
  - `idle-replay.spec.ts` — spoof `document.hidden = true` via
    `Object.defineProperty` + `visibilitychange`, wait 12 s, assert
    counter still 0 (visibility suppression).
  - `idle-replay.spec.ts` — pointerdown at +6 s + pointercancel,
    wait another 6 s (12 s total but only 6 s since input), assert
    counter still 0 (clock reset works).
  - `slot-tap-repeat.spec.ts` — click slot-0, assert counter goes
    `0 → 1 → 2`, slot stays empty (read-only tap).

### What's still deferred to a real device

Everything else in `tasks/betu-11-real-device-polish.md`:

- iOS muted-switch behavior, double-play, echo on speakers + AirPods.
- Tap responsiveness, 300 ms tap-delay verification.
- Outdoor/sunlight contrast.
- Notch / safe-area clipping.
- Battery / overheat under sustained play.
- PWA install banner + service worker decision.
- Accessibility audit (VoiceOver / TalkBack on letter tiles,
  reduced-motion confetti).

## 23. A11y + safe-area + iOS web-app polish (`betu-11`, code-only slice 2)

A second code-only slice of `betu-11` lands the implementation pieces
of the device-polish list whose hardware concern is *verification*,
not *implementation*. The actual on-device pass is still pending — the
user still has to hold the kid's phone and confirm — but the phone
will now meet the code halfway.

**Custom HTML template (`index.html`).** dx CLI ships a default
`prod.index.html` that hardcodes
`<meta name="viewport" content="width=device-width, initial-scale=1">`
with no `viewport-fit`. We override by dropping a custom `index.html`
at the crate root (per dx 0.7's `prepare_html` lookup at
`<crate-root>/index.html`). Our template adds:

- `viewport-fit=cover` so the browser allows content under the system
  bezels and `env(safe-area-inset-*)` resolves to non-zero on notched
  phones.
- `theme-color="#FBFAF7"` matching `--color-bone` so the iOS status-bar
  area and the Android URL bar pick up the app palette.
- `apple-mobile-web-app-capable=yes` +
  `apple-mobile-web-app-status-bar-style=default` +
  `apple-mobile-web-app-title=Betűk` so adding the page to the home
  screen on iOS launches it full-screen with the right window title
  (no "Safari" chrome wrapping; kids tap the icon and land in the
  app).
- `mobile-web-app-capable=yes` for the Android equivalent.
- `format-detection=telephone=no` so iOS doesn't auto-link the digits
  in the progress chip ("2 · 3/12") as phone numbers.
- `<html lang="hu">` so VoiceOver / TalkBack pick the Hungarian voice
  pack for letter announcements.

The `{app_title}` placeholder + `<div id="main">` mount + `</head>` /
`</body>` markers are preserved; dx's `inject_resources`,
`inject_loading_scripts`, and `replace_template_placeholders` continue
to substitute the wasm + JS asset paths and the title at bundle time.
A guard test (`tests/index_html_template.rs`, 4 tests) compiles the
template via `include_str!` and asserts the meta tags are present — the
failure mode if a future scaffold regenerates the template is caught at
`cargo test`, not on a notched device.

**Safe-area CSS.** `.betu-app` gets
`padding-{top,bottom,left,right}: env(safe-area-inset-*)`. Phones
without a notch report zero on those values and lose nothing; notched
phones gain ~44 px on top + ~34 px on the home-indicator side, which
prevents the puzzle header (home icon + progress chip) from sitting
under the notch and the Next button from landing under the home-bar.
Inner screens (`.betu-screen`, `.betu-menu`, `.betu-level-select`)
already use `clamp()`-based padding that compounds gracefully on top
of the outer shell's safe-area; no change needed inside.

**Reduced-motion preference (`prefers-reduced-motion: reduce`).**
Confetti rain (`.betu-rain-drop`) and slot pulse
(`.betu-screen[data-won="true"] .betu-slot`) are the two motion sources
during the win flow. With reduced-motion on:

- Slot pulse animation is suppressed (`animation: none`).
- Confetti drops swap their 1.5 s falling-rotating animation for a
  single 600 ms `betu-fade-in-soft` (opacity 0 → 0.85, no movement).
  Kids who can't tolerate full-screen falling motion still get a
  visual reward — celebration is part of the contract — just not
  vestibular.
- The Next button's 800 ms delayed fade-in is removed: it's visible
  and tappable immediately on `is_won`, since the delay only existed
  to let the motion settle.

`.betu-tile`'s `transform: 250ms ease-out` spring-back transition is
left alone — it's not decorative, it's a direct response to user input
on a wrong drop, and removing it makes wrong drops feel broken.

**ARIA + screen-reader announcement.**

- Letter tiles get `role="button"` + `aria-label="Betű {L}"` ("letter
  L" in Hungarian). Without the label, VoiceOver announces a single
  letter as a flat character; with it, "betű A" reads naturally.
- Slots get `role="button"` + positional `aria-label="Betűhely {N+1}"`
  ("letter slot N", 1-indexed) so the screen reader can communicate
  which position is which.
- The slots row and tiles row become `role="group"` with localized
  labels (`"Betűhelyek"` / `"Betűcserepek"`); the previous hardcoded
  English `aria-label="slots"` / `"tiles"` was wrong for the
  Hungarian-only audience.

The dialog backdrop, parent-zone slider, and reset modal already
carried correct localized labels from earlier subtasks.

### What's still deferred to a real device after this slice

The device-polish list shrinks to:

- iOS muted-switch behavior, double-play, echo on speakers + AirPods.
- Tap responsiveness — modern viewport + Pointer Events should already
  eliminate the 300 ms tap delay; verification is hardware-only.
- Outdoor/sunlight contrast.
- **Verification** that the safe-area inset CSS lands cleanly on a
  real notched phone (the implementation is in; the visual check is
  hardware).
- Battery / overheat under sustained play.
- Service worker decision (intentionally deferred — caching the wasm
  bundle could mask a bad deploy; `betu-12` decides).
- **Verification** that VoiceOver / TalkBack actually read the new
  Hungarian aria-labels with the `lang="hu"` voice (the labels are
  in; the screen-reader check is hardware).

## 24. PWA install — manifest + icons (`betu-11`, code-only slice 4)

A fourth code-only slice of `betu-11` lands the "PWA install banner"
item from the device-polish list. Adding the kid's app to the home
screen is the *only* way the user wants the kid to launch it (no
chrome, no URL bar, no Safari/Chrome icon disguising what it is) — and
without a manifest + icon set, "Add to Home Screen" produces a fuzzy
auto-generated thumbnail of whatever was on screen at the time of the
add. This slice fixes the cosmetics; the install gesture itself is
hardware.

**`assets/manifest.webmanifest`** ships verbatim to
`dist/public/manifest.webmanifest` via the bundle pipeline (Makefile +
CI both `cp` it after `dx bundle`, alongside the audio-cp step).
Fields:

- `name` + `short_name`: "Betűk" (a single 5-char Hungarian word also
  serves as the home-screen label — under iOS's ~12-char truncation).
- `lang: "hu"` + `dir: "ltr"` so screen readers and Chromium pick the
  right voice / direction in the install prompt.
- `start_url: "/"` and `scope: "/"` — the entire app is one route.
- `display: "standalone"` so the launched window has no Safari /
  Chrome chrome (the whole point of the install).
- `orientation: "portrait"` because the puzzle layout is phone-first
  and the safe-area work in §23 is also portrait-tuned.
- `background_color: "#FBFAF7"` + `theme_color: "#FBFAF7"` — both
  match `--color-bone` so the iOS launch splash and the Android URL
  bar are seamless with the app shell.
- `icons`: 192 × 192 + 512 × 512 PNGs at `/icons/icon-{size}.png`,
  `purpose: "any"`. The 192 is what Chrome's install prompt
  thumbnails; the 512 is what Android uses for the splash; iOS uses
  neither and instead reads `apple-touch-icon` (next item).

**`<link>` tags in `index.html`.** Added inside the same `<head>`
that the Run 177 slice extended:

- `<link rel="manifest" href="/manifest.webmanifest">` — the required
  pointer.
- `<link rel="apple-touch-icon" href="/icons/apple-touch-icon.png">`
  — iOS Safari ignores the manifest's `icons` array for home-screen
  add and reads this `<link>` instead. 180 × 180 is the iOS
  recommendation.
- `<link rel="icon" type="image/png" sizes="32x32" href="/icons/favicon-32.png">`
  + `<link rel="icon" type="image/png" sizes="192x192" href="/icons/icon-192.png">`
  — browser-tab favicon at 32, plus a 192 for higher-DPI tab strips.

**Icon design.** A single uppercase **B** for "Betűk" on the bone
background, with a thin tile-border square framing it — visually
matches the in-app letter tiles at a glance. The letterform sits well
inside both Android's circular mask and iOS's rounded-square (~80%
safe zone), so a future maskable variant is a render-time tweak, not
a redesign. No accents, no digraphs (matches the v1 dictionary's own
constraint, so the icon "looks like" the corpus the kid will see).

**Generator: `tools/gen_icons.py`.** Mirrors the
`tools/gen_audio.py` pattern from `betu-09`: a Python+Pillow script
that renders the 1024 px base and downsamples to 512 / 192 / 180 / 32.
Idempotent and deterministic; PNGs are committed (so CI doesn't need
Pillow). Falls back from Arial Bold → DejaVu Bold → Liberation Sans
Bold → Pillow default; warns on the bitmap fallback.

**Tests.** `tests/index_html_template.rs` gains one new test that
asserts all four `<link>` lines are present in the source template.
`tests/manifest_template.rs` (5 tests) parses `manifest.webmanifest`
via `serde_json` and guards required keys (`name`, `short_name`,
`start_url`, `display`, `icons`), the locale + palette match
(`lang="hu"`, `theme_color`/`background_color` = `#FBFAF7`), the
declared icon size matrix (`192x192` + `512x512`), and that all four
referenced PNGs exist on disk with valid PNG magic. The CI bundle
job's `Verify bundle output` step is also extended to fail loudly if
`manifest.webmanifest` or any of the three load-bearing icon PNGs
fail to land in `dist/public/`.

### What's still deferred to a real device after this slice

The device-polish list shrinks again — *install* itself is hardware:

- iOS muted-switch, double-play, echo on speakers + AirPods.
- Tap responsiveness verification.
- Outdoor/sunlight contrast.
- **Verification** that the safe-area inset CSS lands cleanly on a
  notched phone.
- Battery / overheat under sustained play.
- Service worker decision (still `betu-12`).
- VoiceOver / TalkBack pronunciation pass with the `lang="hu"`
  voice.
- **Verification** that "Add to Home Screen" on iOS Safari and
  Chrome's install prompt on Android both render the new icon
  cleanly, and that the launched window is full-screen (no chrome).
  The manifest + icon files exist; the install UX is hardware.
