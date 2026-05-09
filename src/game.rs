//! `Game` is the top-level model: dictionary + persisted `Progress` +
//! current tier + a shuffled queue of upcoming words within that tier +
//! the active `Puzzle`. The puzzle screen reads from and mutates this
//! struct via a Dioxus signal; on win + Next, [`Game::advance_to_next`]
//! records the completion, recomputes tier unlock, and rotates to the
//! next word.

use crate::progress::Progress;
use crate::puzzle::{Puzzle, shuffle};
use crate::word::Word;

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub words: Vec<Word>,
    pub progress: Progress,
    pub current_tier: u32,
    /// Upcoming words for the current tier, in shuffled order. The active
    /// puzzle's word is *not* in this queue. When empty, `advance_to_next`
    /// reshuffles the tier's full word list.
    pub queue: Vec<Word>,
    pub current_puzzle: Puzzle,
    seed: u64,
}

impl Game {
    /// Build a `Game` from the dictionary and persisted progress. `seed`
    /// is `Some(_)` in tests for determinism; in production we pass `None`
    /// and consume entropy via the same path the puzzle shuffler uses.
    pub fn new(words: Vec<Word>, mut progress: Progress, seed: Option<u64>) -> Self {
        // Clamp current_tier into [1, tier_unlocked]: if a future schema
        // change ever leaves these inconsistent, fall back to 1.
        if progress.tier_unlocked < 1 {
            progress.tier_unlocked = 1;
        }
        if progress.current_tier < 1 || progress.current_tier > progress.tier_unlocked {
            progress.current_tier = 1;
        }
        let current_tier = progress.current_tier;
        let mut seed = seed.unwrap_or_else(default_seed);

        let mut queue: Vec<Word> = words
            .iter()
            .filter(|w| w.tier == current_tier)
            .cloned()
            .collect();
        assert!(
            !queue.is_empty(),
            "no words for current_tier={current_tier}; dictionary must contain at least one word per unlocked tier"
        );
        shuffle(&mut queue, Some(seed));
        seed = seed.wrapping_add(1);
        let first = queue.remove(0);
        let puzzle = Puzzle::new(first, Some(seed));
        seed = seed.wrapping_add(1);

        Self {
            words,
            progress,
            current_tier,
            queue,
            current_puzzle: puzzle,
            seed,
        }
    }

    pub fn current_word(&self) -> &Word {
        &self.current_puzzle.word
    }

    pub fn is_won(&self) -> bool {
        self.current_puzzle.is_complete()
    }

    /// Record the current word as completed (if won) and rotate to the
    /// next word in the current tier. When the tier's queue is exhausted,
    /// reshuffle the whole tier — per DESIGN.md §8 free-play rule.
    pub fn advance_to_next(&mut self) {
        if self.is_won() {
            let solved = self.current_word().word.clone();
            self.progress.record_completion(&solved);
            self.progress.recompute_tier_unlock(&self.words);
        }

        if self.queue.is_empty() {
            self.queue = self
                .words
                .iter()
                .filter(|w| w.tier == self.current_tier)
                .cloned()
                .collect();
            shuffle(&mut self.queue, Some(self.seed));
            self.seed = self.seed.wrapping_add(1);
        }

        let next = self.queue.remove(0);
        self.current_puzzle = Puzzle::new(next, Some(self.seed));
        self.seed = self.seed.wrapping_add(1);
        self.progress.current_tier = self.current_tier;
    }
}

#[cfg(target_arch = "wasm32")]
fn default_seed() -> u64 {
    js_sys::Date::now().to_bits()
}

#[cfg(not(target_arch = "wasm32"))]
fn default_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0xDEAD_BEEF_CAFE_F00D)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::N_UNLOCK;
    use crate::puzzle::DropOutcome;

    fn w(word: &str, tier: u32) -> Word {
        Word {
            word: word.to_string(),
            emoji: "🐱".to_string(),
            tier,
        }
    }

    fn dict() -> Vec<Word> {
        vec![
            w("AB", 1),
            w("AC", 1),
            w("AD", 1),
            w("AE", 1),
            w("AF", 1),
            w("AG", 1),
            w("BAB", 2),
            w("BAC", 2),
        ]
    }

    fn solve_current_word(g: &mut Game) {
        // Drive the Puzzle through correct snaps in word order.
        let word = g.current_word().word.clone();
        let centers: Vec<(f64, f64)> = (0..word.chars().count())
            .map(|i| (100.0 + i as f64 * 100.0, 200.0))
            .collect();
        for (pid, (slot_idx, expected)) in (100i32..).zip(word.chars().enumerate()) {
            // Pick the first idle tile whose letter matches and isn't placed.
            let tile_idx = g
                .current_puzzle
                .tiles
                .iter()
                .enumerate()
                .find(|(_, t)| {
                    t.letter == expected && matches!(t.state, crate::puzzle::TileState::Idle)
                })
                .map(|(i, _)| i)
                .expect("expected matching idle tile");
            let center = centers[slot_idx];
            assert!(g.current_puzzle.pickup(tile_idx, pid, center, (0.0, 0.0)));
            let outcome = g.current_puzzle.release(pid, &centers, 40.0);
            assert!(matches!(outcome, DropOutcome::Snapped { .. }));
        }
    }

    #[test]
    fn new_picks_a_tier_one_word_when_progress_is_default() {
        let g = Game::new(dict(), Progress::default(), Some(42));
        assert_eq!(g.current_tier, 1);
        assert_eq!(g.current_word().tier, 1);
        // 6 tier-1 words: 1 in puzzle, 5 in queue.
        assert_eq!(g.queue.len(), 5);
    }

    #[test]
    fn new_clamps_current_tier_to_tier_unlocked() {
        // Misconfigured progress: current_tier = 3 but only tier 1 unlocked.
        let p = Progress {
            completed: Vec::new(),
            current_tier: 3,
            tier_unlocked: 1,
        };
        let g = Game::new(dict(), p, Some(42));
        assert_eq!(g.current_tier, 1);
    }

    #[test]
    fn new_uses_persisted_tier_when_unlocked() {
        let p = Progress {
            completed: Vec::new(),
            current_tier: 2,
            tier_unlocked: 2,
        };
        let g = Game::new(dict(), p, Some(42));
        assert_eq!(g.current_tier, 2);
        assert_eq!(g.current_word().tier, 2);
    }

    #[test]
    fn solving_the_puzzle_transitions_game_to_win_state() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        assert!(!g.is_won());
        solve_current_word(&mut g);
        assert!(g.is_won());
    }

    #[test]
    fn advance_to_next_picks_a_different_word_in_the_same_tier() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        let first = g.current_word().word.clone();
        solve_current_word(&mut g);
        g.advance_to_next();
        assert_eq!(g.current_tier, 1);
        assert_eq!(g.current_word().tier, 1);
        assert_ne!(g.current_word().word, first);
        assert!(!g.is_won(), "new puzzle starts unsolved");
    }

    #[test]
    fn advance_records_completion_in_progress() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        let first = g.current_word().word.clone();
        solve_current_word(&mut g);
        g.advance_to_next();
        assert!(g.progress.completed.contains(&first));
    }

    #[test]
    fn advance_without_winning_does_not_record_anything() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        // No solving — just rotate.
        g.advance_to_next();
        assert!(g.progress.completed.is_empty());
    }

    #[test]
    fn five_completions_unlock_tier_two() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        for _ in 0..(N_UNLOCK as usize) {
            solve_current_word(&mut g);
            g.advance_to_next();
        }
        assert_eq!(g.progress.tier_unlocked, 2);
        assert_eq!(g.progress.completed.len(), N_UNLOCK as usize);
    }

    #[test]
    fn no_word_repeats_until_tier_session_exhausts() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        let mut seen = vec![g.current_word().word.clone()];
        // 6 tier-1 words; rotate 5 more times to see them all once.
        for _ in 0..5 {
            g.advance_to_next();
            let cur = g.current_word().word.clone();
            assert!(!seen.contains(&cur), "word {cur} repeated within session");
            seen.push(cur);
        }
        assert_eq!(seen.len(), 6, "all 6 tier-1 words seen in one session");
        // After the 6th word, the queue is empty; the 7th advance reshuffles.
        g.advance_to_next();
        assert!(seen.contains(&g.current_word().word));
    }

    #[test]
    fn progress_survives_a_round_trip_through_persistent_state() {
        // Solve two words, advance, then "save" → "load" → resume.
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        solve_current_word(&mut g);
        g.advance_to_next();
        solve_current_word(&mut g);
        g.advance_to_next();
        let saved = g.progress.to_json();

        let restored = Progress::from_json(&saved).expect("must parse");
        assert_eq!(restored.completed.len(), 2);

        // A new Game built from the restored progress respects it.
        let g2 = Game::new(dict(), restored.clone(), Some(99));
        assert_eq!(g2.progress.completed, restored.completed);
        assert_eq!(g2.progress.tier_unlocked, restored.tier_unlocked);
    }
}
