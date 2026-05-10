use crate::word::Word;

pub type Pos = (f64, f64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileState {
    Idle,
    Dragging {
        pointer_id: i32,
        pointer: Pos,
        origin_center: Pos,
    },
    Placed {
        slot_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub letter: char,
    pub state: TileState,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Puzzle {
    pub word: Word,
    pub slots: Vec<Option<usize>>,
    pub tiles: Vec<Tile>,
    pub wrong_drops: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DropOutcome {
    Snapped {
        tile_index: usize,
        slot_index: usize,
    },
    SprungBack {
        tile_index: usize,
    },
    /// Pointer never left the tile between pickup and release — a tap,
    /// not a drag. Tile silently returns to Idle; `wrong_drops` is not
    /// incremented. Lets curious "what does this say?" taps not punish.
    Tapped {
        tile_index: usize,
    },
    Ignored,
}

/// Below this many CSS px of pointer travel from the tile's origin, a
/// release counts as a tap rather than a drag. Above typical finger
/// jitter so deliberate drags still register.
pub const TAP_THRESHOLD_PX: f64 = 8.0;

impl Puzzle {
    pub fn new(word: Word, seed: Option<u64>) -> Self {
        let len = word.word.chars().count();
        let mut letters: Vec<char> = word.word.chars().collect();
        shuffle(&mut letters, seed);
        let tiles = letters
            .into_iter()
            .map(|letter| Tile {
                letter,
                state: TileState::Idle,
            })
            .collect();
        Self {
            slots: vec![None; len],
            tiles,
            word,
            wrong_drops: 0,
        }
    }

    pub fn dragging_tile(&self) -> Option<usize> {
        self.tiles
            .iter()
            .position(|t| matches!(t.state, TileState::Dragging { .. }))
    }

    fn dragging_with(&self, pointer_id: i32) -> Option<usize> {
        self.tiles.iter().position(
            |t| matches!(t.state, TileState::Dragging { pointer_id: pid, .. } if pid == pointer_id),
        )
    }

    pub fn is_correct_target(&self, tile_index: usize, slot_index: usize) -> bool {
        let Some(tile) = self.tiles.get(tile_index) else {
            return false;
        };
        let Some(expected) = self.word.word.chars().nth(slot_index) else {
            return false;
        };
        tile.letter == expected
    }

    pub fn is_complete(&self) -> bool {
        !self.slots.is_empty() && self.slots.iter().all(|s| s.is_some())
    }

    pub fn pickup(
        &mut self,
        tile_index: usize,
        pointer_id: i32,
        pointer: Pos,
        origin_center: Pos,
    ) -> bool {
        if self.dragging_tile().is_some() {
            return false;
        }
        let Some(tile) = self.tiles.get_mut(tile_index) else {
            return false;
        };
        if !matches!(tile.state, TileState::Idle) {
            return false;
        }
        tile.state = TileState::Dragging {
            pointer_id,
            pointer,
            origin_center,
        };
        true
    }

    pub fn pointer_move(&mut self, pointer_id: i32, pointer: Pos) -> bool {
        let Some(idx) = self.dragging_with(pointer_id) else {
            return false;
        };
        if let TileState::Dragging {
            pointer: ref mut p, ..
        } = self.tiles[idx].state
        {
            *p = pointer;
            return true;
        }
        false
    }

    pub fn release(
        &mut self,
        pointer_id: i32,
        slot_centers: &[Pos],
        snap_radius: f64,
    ) -> DropOutcome {
        let Some(tile_index) = self.dragging_with(pointer_id) else {
            return DropOutcome::Ignored;
        };
        let TileState::Dragging {
            pointer,
            origin_center,
            ..
        } = self.tiles[tile_index].state
        else {
            return DropOutcome::Ignored;
        };

        // Tap detection: pointer barely moved from where the user touched
        // down. Treat as a tap, not a drag attempt — silent return to
        // Idle, no wrong-drop penalty.
        let tdx = pointer.0 - origin_center.0;
        let tdy = pointer.1 - origin_center.1;
        if tdx * tdx + tdy * tdy < TAP_THRESHOLD_PX * TAP_THRESHOLD_PX {
            self.tiles[tile_index].state = TileState::Idle;
            return DropOutcome::Tapped { tile_index };
        }

        let nearest = nearest_slot_within(pointer, slot_centers, snap_radius);
        let snap_target = nearest.and_then(|slot_index| {
            if self.slots.get(slot_index).copied().flatten().is_some() {
                None
            } else if self.is_correct_target(tile_index, slot_index) {
                Some(slot_index)
            } else {
                None
            }
        });

        match snap_target {
            Some(slot_index) => {
                self.slots[slot_index] = Some(tile_index);
                self.tiles[tile_index].state = TileState::Placed { slot_index };
                DropOutcome::Snapped {
                    tile_index,
                    slot_index,
                }
            }
            None => {
                self.tiles[tile_index].state = TileState::Idle;
                self.wrong_drops += 1;
                DropOutcome::SprungBack { tile_index }
            }
        }
    }

    pub fn cancel(&mut self, pointer_id: i32) -> bool {
        let Some(idx) = self.dragging_with(pointer_id) else {
            return false;
        };
        self.tiles[idx].state = TileState::Idle;
        true
    }
}

fn nearest_slot_within(p: Pos, centers: &[Pos], radius: f64) -> Option<usize> {
    let r2 = radius * radius;
    let mut best: Option<(usize, f64)> = None;
    for (i, c) in centers.iter().enumerate() {
        let dx = p.0 - c.0;
        let dy = p.1 - c.1;
        let d2 = dx * dx + dy * dy;
        if d2 <= r2 && best.is_none_or(|(_, b)| d2 < b) {
            best = Some((i, d2));
        }
    }
    best.map(|(i, _)| i)
}

pub fn shuffle<T>(slice: &mut [T], seed: Option<u64>) {
    if slice.len() < 2 {
        return;
    }
    let s = seed.unwrap_or_else(entropy_seed);
    let mut rng = XorShift64::new(s);
    for i in (1..slice.len()).rev() {
        let j = (rng.next() % (i as u64 + 1)) as usize;
        slice.swap(i, j);
    }
}

struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

fn entropy_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now().to_bits()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xDEAD_BEEF_CAFE_F00D)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(word_str: &str, tier: u32) -> Word {
        Word {
            word: word_str.to_string(),
            emoji: "🍎".to_string(),
            tier,
        }
    }

    fn alma() -> Puzzle {
        Puzzle::new(sample("ALMA", 2), Some(42))
    }

    fn idx_of(p: &Puzzle, letter: char, nth: usize) -> usize {
        p.tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.letter == letter)
            .nth(nth)
            .expect("letter occurrence not found in tiles")
            .0
    }

    fn slot_centers_alma() -> Vec<Pos> {
        (0..4).map(|i| (200.0 + i as f64 * 100.0, 300.0)).collect()
    }

    #[test]
    fn new_creates_one_slot_and_one_tile_per_letter() {
        let p = Puzzle::new(sample("ALMA", 2), Some(42));
        assert_eq!(p.slots.len(), 4);
        assert_eq!(p.tiles.len(), 4);
        assert!(p.slots.iter().all(|s| s.is_none()));
        assert!(p.tiles.iter().all(|t| matches!(t.state, TileState::Idle)));
        assert_eq!(p.wrong_drops, 0);
    }

    #[test]
    fn tiles_contain_same_letters_as_word() {
        let p = Puzzle::new(sample("ALMA", 2), Some(42));
        let mut got: Vec<char> = p.tiles.iter().map(|t| t.letter).collect();
        let mut want: Vec<char> = "ALMA".chars().collect();
        got.sort();
        want.sort();
        assert_eq!(got, want);
    }

    #[test]
    fn pickup_marks_tile_dragging() {
        let mut p = alma();
        let l = idx_of(&p, 'L', 0);
        assert!(p.pickup(l, 7, (10.0, 20.0), (50.0, 60.0)));
        assert!(matches!(
            p.tiles[l].state,
            TileState::Dragging {
                pointer_id: 7,
                pointer: (10.0, 20.0),
                origin_center: (50.0, 60.0),
            }
        ));
        assert_eq!(p.dragging_tile(), Some(l));
    }

    #[test]
    fn second_pointer_is_ignored_during_drag() {
        let mut p = alma();
        let l = idx_of(&p, 'L', 0);
        let m = idx_of(&p, 'M', 0);
        assert!(p.pickup(l, 1, (0.0, 0.0), (0.0, 0.0)));
        // Different tile, different pointer — must be refused.
        assert!(!p.pickup(m, 2, (0.0, 0.0), (0.0, 0.0)));
        assert!(matches!(p.tiles[m].state, TileState::Idle));
        // Same tile, different pointer — must also be refused.
        assert!(!p.pickup(l, 2, (0.0, 0.0), (0.0, 0.0)));
    }

    #[test]
    fn pointer_move_updates_only_the_active_drag() {
        let mut p = alma();
        let l = idx_of(&p, 'L', 0);
        p.pickup(l, 7, (0.0, 0.0), (0.0, 0.0));
        assert!(p.pointer_move(7, (123.0, 456.0)));
        if let TileState::Dragging { pointer, .. } = p.tiles[l].state {
            assert_eq!(pointer, (123.0, 456.0));
        } else {
            panic!("expected Dragging");
        }
        // Wrong pointer id — no update, no error.
        assert!(!p.pointer_move(99, (1.0, 1.0)));
        if let TileState::Dragging { pointer, .. } = p.tiles[l].state {
            assert_eq!(pointer, (123.0, 456.0));
        }
    }

    #[test]
    fn release_on_correct_slot_snaps_and_locks() {
        // ALMA: word position 0 = 'A'. Drop the first 'A' tile near slot 0.
        let mut p = alma();
        let a0 = idx_of(&p, 'A', 0);
        p.pickup(a0, 1, (200.0, 300.0), (50.0, 600.0));
        let centers = slot_centers_alma();
        let outcome = p.release(1, &centers, 40.0);
        assert_eq!(
            outcome,
            DropOutcome::Snapped {
                tile_index: a0,
                slot_index: 0,
            }
        );
        assert_eq!(p.slots[0], Some(a0));
        assert!(matches!(
            p.tiles[a0].state,
            TileState::Placed { slot_index: 0 }
        ));
        assert_eq!(p.wrong_drops, 0);
    }

    #[test]
    fn release_on_wrong_slot_springs_back_and_counts() {
        // ALMA position 1 = 'L'. Pick an 'A' tile, drop on slot 1 → wrong.
        let mut p = alma();
        let a0 = idx_of(&p, 'A', 0);
        p.pickup(a0, 1, (300.0, 300.0), (50.0, 600.0));
        let centers = slot_centers_alma();
        let outcome = p.release(1, &centers, 40.0);
        assert_eq!(outcome, DropOutcome::SprungBack { tile_index: a0 });
        assert_eq!(p.slots[1], None);
        assert!(matches!(p.tiles[a0].state, TileState::Idle));
        assert_eq!(p.wrong_drops, 1);
    }

    #[test]
    fn release_in_empty_space_springs_back_and_counts() {
        let mut p = alma();
        let l = idx_of(&p, 'L', 0);
        p.pickup(l, 1, (10.0, 10.0), (0.0, 0.0));
        let centers = slot_centers_alma();
        let outcome = p.release(1, &centers, 40.0);
        assert_eq!(outcome, DropOutcome::SprungBack { tile_index: l });
        assert_eq!(p.wrong_drops, 1);
        assert!(matches!(p.tiles[l].state, TileState::Idle));
    }

    #[test]
    fn release_with_no_active_drag_is_ignored() {
        let mut p = alma();
        let centers = slot_centers_alma();
        assert_eq!(p.release(1, &centers, 40.0), DropOutcome::Ignored);
        assert_eq!(p.wrong_drops, 0);
    }

    #[test]
    fn already_placed_tile_cannot_be_picked_up() {
        let mut p = alma();
        let a0 = idx_of(&p, 'A', 0);
        p.pickup(a0, 1, (200.0, 300.0), (0.0, 0.0));
        p.release(1, &slot_centers_alma(), 40.0);
        assert!(matches!(
            p.tiles[a0].state,
            TileState::Placed { slot_index: 0 }
        ));
        // Try to pick it up again.
        assert!(!p.pickup(a0, 2, (0.0, 0.0), (0.0, 0.0)));
        assert!(matches!(
            p.tiles[a0].state,
            TileState::Placed { slot_index: 0 }
        ));
    }

    #[test]
    fn drop_on_filled_slot_springs_back() {
        let mut p = alma();
        // Place 'A' (first one) into slot 0.
        let a0 = idx_of(&p, 'A', 0);
        p.pickup(a0, 1, (200.0, 300.0), (0.0, 0.0));
        p.release(1, &slot_centers_alma(), 40.0);
        // Try to drop the second 'A' onto slot 0 (already filled). Slot 3 is
        // also 'A' and empty — but the drop is targeted at slot 0's center.
        let a1 = idx_of(&p, 'A', 1);
        p.pickup(a1, 2, (200.0, 300.0), (0.0, 0.0));
        let outcome = p.release(2, &slot_centers_alma(), 40.0);
        assert_eq!(outcome, DropOutcome::SprungBack { tile_index: a1 });
        assert_eq!(p.slots[0], Some(a0));
        assert_eq!(p.wrong_drops, 1);
    }

    #[test]
    fn cancel_returns_to_idle_without_counting_wrong_drop() {
        let mut p = alma();
        let l = idx_of(&p, 'L', 0);
        p.pickup(l, 1, (0.0, 0.0), (0.0, 0.0));
        assert!(p.cancel(1));
        assert!(matches!(p.tiles[l].state, TileState::Idle));
        assert_eq!(p.wrong_drops, 0);
        // Cancel with no active drag is a no-op, returns false.
        assert!(!p.cancel(1));
    }

    #[test]
    fn snap_radius_is_inclusive_at_boundary() {
        // Exactly at radius — should snap.
        let mut p = alma();
        let a0 = idx_of(&p, 'A', 0);
        p.pickup(a0, 1, (240.0, 300.0), (0.0, 0.0)); // slot 0 center is (200,300)
        let outcome = p.release(1, &slot_centers_alma(), 40.0);
        assert!(matches!(outcome, DropOutcome::Snapped { .. }));
    }

    #[test]
    fn just_outside_snap_radius_springs_back() {
        let mut p = alma();
        let a0 = idx_of(&p, 'A', 0);
        // 41 px away from any slot — outside radius.
        p.pickup(a0, 1, (241.0, 300.0), (0.0, 0.0));
        let outcome = p.release(1, &slot_centers_alma(), 40.0);
        assert!(matches!(outcome, DropOutcome::SprungBack { .. }));
    }

    #[test]
    fn is_complete_true_only_when_all_slots_filled() {
        let mut p = Puzzle::new(sample("MA", 1), Some(42));
        assert!(!p.is_complete());
        let m = idx_of(&p, 'M', 0);
        let a = idx_of(&p, 'A', 0);
        // slot centers for "MA": index 0 is 'M', index 1 is 'A'.
        let centers = vec![(0.0, 0.0), (100.0, 0.0)];
        // Origin is offset from the slot so the pointer travels far
        // enough to register as a drag (above TAP_THRESHOLD_PX).
        p.pickup(m, 1, (0.0, 0.0), (50.0, 50.0));
        assert_eq!(
            p.release(1, &centers, 10.0),
            DropOutcome::Snapped {
                tile_index: m,
                slot_index: 0
            }
        );
        assert!(!p.is_complete());
        p.pickup(a, 2, (100.0, 0.0), (50.0, 50.0));
        assert_eq!(
            p.release(2, &centers, 10.0),
            DropOutcome::Snapped {
                tile_index: a,
                slot_index: 1
            }
        );
        assert!(p.is_complete());
    }

    #[test]
    fn release_at_origin_is_a_tap_not_a_wrong_drop() {
        // User report 2026-05-10: tapping a letter on the kid's phone
        // briefly highlighted it then dropped back to Idle, which felt
        // like the tile "disappeared" and counted unfairly as a wrong
        // drop. The fix is to detect a tap (pointer never left origin)
        // and emit `Tapped` instead of `SprungBack`.
        let mut p = alma();
        let l = idx_of(&p, 'L', 0);
        // Pointer == origin: the user touched and lifted without moving.
        p.pickup(l, 1, (100.0, 200.0), (100.0, 200.0));
        let outcome = p.release(1, &slot_centers_alma(), 40.0);
        assert_eq!(outcome, DropOutcome::Tapped { tile_index: l });
        assert!(matches!(p.tiles[l].state, TileState::Idle));
        assert_eq!(p.wrong_drops, 0, "tap must not count as a wrong drop");
        assert!(p.slots.iter().all(|s| s.is_none()));
    }

    #[test]
    fn release_within_tap_threshold_is_a_tap() {
        // Tiny finger jitter (within TAP_THRESHOLD_PX) still counts as
        // a tap. Threshold is 8 px; (5, 5) → 7.07 px from origin.
        let mut p = alma();
        let l = idx_of(&p, 'L', 0);
        p.pickup(l, 1, (5.0, 5.0), (0.0, 0.0));
        let outcome = p.release(1, &slot_centers_alma(), 40.0);
        assert_eq!(outcome, DropOutcome::Tapped { tile_index: l });
        assert_eq!(p.wrong_drops, 0);
    }

    #[test]
    fn release_just_past_tap_threshold_is_a_drag() {
        // 9 px on the x-axis is over the 8 px threshold — a deliberate
        // drag, not a tap. With no slot in range, springs back as before.
        let mut p = alma();
        let l = idx_of(&p, 'L', 0);
        p.pickup(l, 1, (9.0, 0.0), (0.0, 0.0));
        let outcome = p.release(1, &slot_centers_alma(), 40.0);
        assert_eq!(outcome, DropOutcome::SprungBack { tile_index: l });
        assert_eq!(p.wrong_drops, 1);
    }

    #[test]
    fn drag_then_return_to_origin_is_still_a_drag_not_a_tap() {
        // Once the pointer has left origin, the user has expressed drag
        // intent. If they happen to release back near where they started,
        // the model uses the *current* pointer position — if that's
        // within the threshold of origin, it's a tap; if not, it's a
        // drag. This test pins down the boundary at the release moment.
        let mut p = alma();
        let l = idx_of(&p, 'L', 0);
        p.pickup(l, 1, (0.0, 0.0), (0.0, 0.0));
        // Drag well away from origin.
        assert!(p.pointer_move(1, (100.0, 100.0)));
        // Then drift back to origin before lifting.
        assert!(p.pointer_move(1, (0.0, 0.0)));
        let outcome = p.release(1, &slot_centers_alma(), 40.0);
        // Released at origin → tap (no penalty), even though there was
        // intervening movement. This mirrors how a user who hesitates
        // and pulls back gets a free retry.
        assert_eq!(outcome, DropOutcome::Tapped { tile_index: l });
        assert_eq!(p.wrong_drops, 0);
    }

    #[test]
    fn shuffle_with_same_seed_is_deterministic() {
        let mut a: Vec<u32> = (0..10).collect();
        let mut b: Vec<u32> = (0..10).collect();
        shuffle(&mut a, Some(42));
        shuffle(&mut b, Some(42));
        assert_eq!(a, b);
    }

    #[test]
    fn shuffle_actually_permutes() {
        let mut v: Vec<u32> = (0..20).collect();
        let original = v.clone();
        shuffle(&mut v, Some(42));
        assert_ne!(
            v, original,
            "shuffle with seed 42 should not produce identity for n=20"
        );
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, original, "shuffle must be a permutation");
    }

    #[test]
    fn shuffle_different_seeds_differ() {
        let mut a: Vec<u32> = (0..20).collect();
        let mut b: Vec<u32> = (0..20).collect();
        shuffle(&mut a, Some(1));
        shuffle(&mut b, Some(2));
        assert_ne!(a, b);
    }

    #[test]
    fn shuffle_zero_seed_is_safe() {
        let mut v: Vec<u32> = (0..10).collect();
        shuffle(&mut v, Some(0));
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, (0..10).collect::<Vec<_>>());
    }
}
