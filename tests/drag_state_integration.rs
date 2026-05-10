//! End-to-end exercise of the Puzzle drag state machine through the same
//! public API the pointer-event handlers use. This is the "integration test"
//! the betu-06 spec asks for; we cannot drive real DOM events from a
//! non-wasm `cargo test`, so we drive the model directly with the same
//! sequence of calls (`pickup` → `pointer_move` → `release`) that the
//! `onpointerdown` / `onpointermove` / `onpointerup` handlers in
//! `puzzle_screen.rs` issue.

use betu_tanulas::{DropOutcome, Puzzle, TileState, Word};

fn word(word: &str, emoji: &str, tier: u32) -> Word {
    Word {
        word: word.to_string(),
        emoji: emoji.to_string(),
        tier,
    }
}

fn slot_centers_horizontal(n: usize) -> Vec<(f64, f64)> {
    (0..n).map(|i| (200.0 + i as f64 * 100.0, 400.0)).collect()
}

fn first_tile_with_letter(p: &Puzzle, letter: char, nth: usize) -> usize {
    p.tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| t.letter == letter)
        .nth(nth)
        .map(|(i, _)| i)
        .expect("expected tile not found")
}

#[test]
fn complete_a_three_letter_word_end_to_end() {
    // CICA → 4 letters: C-I-C-A. Drop in order, finishing the puzzle.
    let mut p = Puzzle::new(word("CICA", "🐱", 1), Some(7));
    let centers = slot_centers_horizontal(4);
    let snap = 40.0;

    let drops = [
        (0, first_tile_with_letter(&p, 'C', 0), 1),
        (1, first_tile_with_letter(&p, 'I', 0), 2),
        (2, first_tile_with_letter(&p, 'C', 1), 3),
        (3, first_tile_with_letter(&p, 'A', 0), 4),
    ];

    for (slot_index, tile_index, pid) in drops {
        let center = centers[slot_index];
        // Pointer touches down somewhere distant.
        assert!(p.pickup(tile_index, pid, (10.0, 10.0), (0.0, 0.0)));
        // The finger drags toward the slot.
        p.pointer_move(pid, (center.0 - 60.0, center.1 - 30.0));
        p.pointer_move(pid, (center.0 - 5.0, center.1));
        // Release exactly on the slot.
        let outcome = p.release(pid, &centers, snap);
        assert_eq!(
            outcome,
            DropOutcome::Snapped {
                tile_index,
                slot_index,
            }
        );
        assert_eq!(p.slots[slot_index], Some(tile_index));
    }

    assert!(p.is_complete());
    assert_eq!(p.wrong_drops, 0);
}

#[test]
fn wrong_drop_then_correct_drop_keeps_word_completable() {
    // ALMA: try the wrong tile in slot 0, spring back, then the right one.
    let mut p = Puzzle::new(word("ALMA", "🍎", 2), Some(7));
    let centers = slot_centers_horizontal(4);
    let snap = 40.0;

    let l_tile = first_tile_with_letter(&p, 'L', 0);
    // Try L in slot 0 (which expects A).
    p.pickup(l_tile, 1, (centers[0].0, centers[0].1), (0.0, 0.0));
    let outcome = p.release(1, &centers, snap);
    assert_eq!(outcome, DropOutcome::SprungBack { tile_index: l_tile });
    assert!(matches!(p.tiles[l_tile].state, TileState::Idle));
    assert_eq!(p.wrong_drops, 1);
    assert!(p.slots.iter().all(|s| s.is_none()));

    // Now drop the first 'A' tile into slot 0.
    let a_tile = first_tile_with_letter(&p, 'A', 0);
    p.pickup(a_tile, 2, (centers[0].0, centers[0].1), (0.0, 0.0));
    let outcome = p.release(2, &centers, snap);
    assert_eq!(
        outcome,
        DropOutcome::Snapped {
            tile_index: a_tile,
            slot_index: 0,
        }
    );
    assert_eq!(p.slots[0], Some(a_tile));
    // wrong_drops stays at 1 — only the failed attempt counts.
    assert_eq!(p.wrong_drops, 1);
}

#[test]
fn second_pointer_does_not_disturb_active_drag() {
    let mut p = Puzzle::new(word("MA", "🤲", 1), Some(7));
    let centers = vec![(100.0, 100.0), (200.0, 100.0)];
    let snap = 30.0;
    let m = first_tile_with_letter(&p, 'M', 0);
    let a = first_tile_with_letter(&p, 'A', 0);

    // Primary pointer 1 picks up M and starts moving.
    assert!(p.pickup(m, 1, (50.0, 50.0), (0.0, 0.0)));
    p.pointer_move(1, (80.0, 80.0));

    // Stray second pointer 2 tries to pick up A — must be refused.
    assert!(!p.pickup(a, 2, (200.0, 100.0), (0.0, 0.0)));
    assert!(matches!(p.tiles[a].state, TileState::Idle));

    // Stray second pointer 2 tries to "move" — no-op, primary keeps its
    // position.
    assert!(!p.pointer_move(2, (1000.0, 1000.0)));
    if let TileState::Dragging { pointer, .. } = p.tiles[m].state {
        assert_eq!(pointer, (80.0, 80.0));
    } else {
        panic!("primary drag must still be live");
    }

    // Release on slot 0 (M's correct slot).
    let outcome = p.release(1, &centers, snap);
    assert_eq!(
        outcome,
        DropOutcome::Snapped {
            tile_index: m,
            slot_index: 0,
        }
    );
}

#[test]
fn tapping_a_tile_does_not_disappear_or_count_as_wrong_drop() {
    // Regression for the user's 2026-05-10 device-test report:
    // "Ha rányomok egy betűre, azonnal eltűnik, nem tudom arrébb húzni."
    // A tap (pointerdown + pointerup at the same client point, no
    // pointermove) used to flip the tile Dragging → Idle and bump
    // wrong_drops, which felt punitive. The model now distinguishes
    // a tap from a drag and silently returns the tile to Idle.
    let mut p = Puzzle::new(word("CICA", "🐱", 1), Some(7));
    let centers = slot_centers_horizontal(4);
    let snap = 40.0;

    let tile_index = first_tile_with_letter(&p, 'C', 0);
    let tile_origin = (160.0, 600.0); // tile center, far from any slot

    // Tap: pickup at the tile center, release at the tile center,
    // no pointermove in between.
    assert!(p.pickup(tile_index, 1, tile_origin, tile_origin));
    let outcome = p.release(1, &centers, snap);

    assert_eq!(outcome, DropOutcome::Tapped { tile_index });
    assert!(matches!(p.tiles[tile_index].state, TileState::Idle));
    assert_eq!(
        p.wrong_drops, 0,
        "a static tap must not be counted as a wrong drop"
    );
    assert!(
        p.slots.iter().all(|s| s.is_none()),
        "no slot should have been filled by a tap"
    );
}

#[test]
fn tap_then_real_drag_to_correct_slot_still_works() {
    // The tap path must not interfere with the normal drag path on
    // subsequent attempts. After a tap, the user can still pick up the
    // same tile and drag it to its slot.
    let mut p = Puzzle::new(word("MA", "🤲", 1), Some(7));
    let centers = vec![(100.0, 100.0), (200.0, 100.0)];
    let snap = 40.0;
    let m = first_tile_with_letter(&p, 'M', 0);

    // 1) Tap: same point pickup + release.
    assert!(p.pickup(m, 1, (50.0, 400.0), (50.0, 400.0)));
    assert_eq!(
        p.release(1, &centers, snap),
        DropOutcome::Tapped { tile_index: m }
    );
    assert!(matches!(p.tiles[m].state, TileState::Idle));
    assert_eq!(p.wrong_drops, 0);

    // 2) Real drag to slot 0 (M's correct slot).
    assert!(p.pickup(m, 2, (50.0, 400.0), (50.0, 400.0)));
    p.pointer_move(2, (100.0, 100.0));
    assert_eq!(
        p.release(2, &centers, snap),
        DropOutcome::Snapped {
            tile_index: m,
            slot_index: 0,
        }
    );
    assert_eq!(p.slots[0], Some(m));
}

#[test]
fn pointer_cancel_returns_tile_home_without_penalty() {
    let mut p = Puzzle::new(word("ALMA", "🍎", 2), Some(7));
    let l_tile = first_tile_with_letter(&p, 'L', 0);
    p.pickup(l_tile, 1, (10.0, 10.0), (0.0, 0.0));
    p.pointer_move(1, (50.0, 50.0));
    assert!(p.cancel(1));
    assert!(matches!(p.tiles[l_tile].state, TileState::Idle));
    assert_eq!(p.wrong_drops, 0);
}
