//! `Game` is the top-level model: dictionary + persisted `Progress` +
//! current tier + a shuffled queue of upcoming words within that tier +
//! the active `Puzzle`. The puzzle screen reads from and mutates this
//! struct via a Dioxus signal; on win + Next, [`Game::advance_to_next`]
//! records the completion, recomputes tier unlock, and rotates to the
//! next word.

use crate::progress::Progress;
use crate::puzzle::{Puzzle, shuffle};
use crate::screen::Screen;
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
    pub screen: Screen,
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
            screen: Screen::Menu,
            seed,
        }
    }

    pub fn current_word(&self) -> &Word {
        &self.current_puzzle.word
    }

    pub fn is_won(&self) -> bool {
        self.current_puzzle.is_complete()
    }

    /// Words available in `tier`, in their dictionary order (stable for UI
    /// listing). Returned by reference so callers can index without cloning.
    pub fn words_in_tier(&self, tier: u32) -> Vec<&Word> {
        self.words.iter().filter(|w| w.tier == tier).collect()
    }

    /// Has this word ever been completed?
    pub fn is_completed(&self, word: &str) -> bool {
        self.progress.completed.iter().any(|w| w == word)
    }

    /// Tap from the main menu's tier button. No-op if `tier` is locked or
    /// out of range — buttons should already be `disabled`, but the model
    /// double-checks so test sequences and accidental taps stay safe.
    pub fn enter_tier(&mut self, tier: u32) {
        if tier == 0 || tier > self.progress.tier_unlocked {
            return;
        }
        self.screen = Screen::LevelSelect { tier };
    }

    /// Tap on a specific word tile in the level-select grid. Replaces the
    /// active puzzle with a fresh one for the chosen word, builds a queue
    /// of the *other* words in the same tier (so post-win advance still
    /// works), and switches the screen.
    pub fn start_word(&mut self, word_str: &str) {
        let Some(target) = self.words.iter().find(|w| w.word == word_str).cloned() else {
            return;
        };
        if target.tier > self.progress.tier_unlocked {
            return;
        }
        self.current_tier = target.tier;
        self.progress.current_tier = target.tier;

        self.queue = self
            .words
            .iter()
            .filter(|w| w.tier == target.tier && w.word != target.word)
            .cloned()
            .collect();
        shuffle(&mut self.queue, Some(self.seed));
        self.seed = self.seed.wrapping_add(1);

        self.current_puzzle = Puzzle::new(target, Some(self.seed));
        self.seed = self.seed.wrapping_add(1);

        self.screen = Screen::Puzzle;
    }

    /// Big "Play" button on the main menu. Resumes the active puzzle if
    /// one is in flight; otherwise starts a fresh tier-1 word. Either way,
    /// switches to the puzzle screen.
    pub fn resume_play(&mut self) {
        // The constructor always seeds `current_puzzle` with a tier word,
        // so resume just switches screen. If state somehow drifted, the
        // user can still tap a tier button to enter level-select.
        self.screen = Screen::Puzzle;
    }

    /// Tap on the in-game home icon. Drops back to the main menu without
    /// disturbing the active puzzle (so the kid can resume where they
    /// were).
    pub fn go_to_menu(&mut self) {
        self.screen = Screen::Menu;
    }

    /// Hidden parent action from the main menu. Wipes progress and
    /// rewinds the current puzzle to a fresh tier-1 word so resume picks
    /// up cleanly. Caller is responsible for persisting via
    /// `progress::save` after.
    pub fn reset_progress(&mut self) {
        self.progress = Progress::default();
        self.current_tier = 1;

        self.queue = self.words.iter().filter(|w| w.tier == 1).cloned().collect();
        shuffle(&mut self.queue, Some(self.seed));
        self.seed = self.seed.wrapping_add(1);
        if !self.queue.is_empty() {
            let first = self.queue.remove(0);
            self.current_puzzle = Puzzle::new(first, Some(self.seed));
            self.seed = self.seed.wrapping_add(1);
        }
        self.screen = Screen::Menu;
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
            volume: crate::audio::VOLUME_DEFAULT,
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
            volume: crate::audio::VOLUME_DEFAULT,
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
    fn new_starts_on_menu_screen() {
        let g = Game::new(dict(), Progress::default(), Some(42));
        assert_eq!(g.screen, Screen::Menu);
    }

    #[test]
    fn enter_tier_switches_to_level_select_when_unlocked() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        g.enter_tier(1);
        assert_eq!(g.screen, Screen::LevelSelect { tier: 1 });
    }

    #[test]
    fn enter_tier_is_a_no_op_when_locked() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        g.enter_tier(2);
        assert_eq!(
            g.screen,
            Screen::Menu,
            "tier 2 is locked at default; menu must not change"
        );
    }

    #[test]
    fn enter_tier_is_a_no_op_for_tier_zero() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        g.enter_tier(0);
        assert_eq!(g.screen, Screen::Menu);
    }

    #[test]
    fn start_word_loads_chosen_word_into_puzzle_and_switches_screen() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        g.start_word("AC");
        assert_eq!(g.screen, Screen::Puzzle);
        assert_eq!(g.current_word().word, "AC");
        assert!(!g.queue.iter().any(|w| w.word == "AC"));
    }

    #[test]
    fn start_word_refuses_word_in_locked_tier() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        // BAB is tier 2; default unlocked is 1.
        g.start_word("BAB");
        assert_eq!(g.screen, Screen::Menu);
        assert_ne!(g.current_word().word, "BAB");
    }

    #[test]
    fn resume_play_switches_to_puzzle_without_resetting_active_puzzle() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        let active = g.current_word().word.clone();
        g.resume_play();
        assert_eq!(g.screen, Screen::Puzzle);
        assert_eq!(g.current_word().word, active);
    }

    #[test]
    fn go_to_menu_returns_to_menu_without_disturbing_puzzle() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        g.start_word("AB");
        let active = g.current_word().word.clone();
        g.go_to_menu();
        assert_eq!(g.screen, Screen::Menu);
        assert_eq!(g.current_word().word, active);
    }

    #[test]
    fn reset_progress_wipes_completion_and_returns_to_tier_one() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        for _ in 0..3 {
            solve_current_word(&mut g);
            g.advance_to_next();
        }
        assert_eq!(g.progress.completed.len(), 3);
        g.reset_progress();
        assert!(g.progress.completed.is_empty());
        assert_eq!(g.progress.tier_unlocked, 1);
        assert_eq!(g.current_tier, 1);
        assert_eq!(g.screen, Screen::Menu);
    }

    #[test]
    fn is_completed_reflects_progress_completed_list() {
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        let first = g.current_word().word.clone();
        assert!(!g.is_completed(&first));
        solve_current_word(&mut g);
        g.advance_to_next();
        assert!(g.is_completed(&first));
    }

    #[test]
    fn words_in_tier_filters_by_tier() {
        let g = Game::new(dict(), Progress::default(), Some(42));
        let tier1 = g.words_in_tier(1);
        assert_eq!(tier1.len(), 6);
        assert!(tier1.iter().all(|w| w.tier == 1));
        let tier2 = g.words_in_tier(2);
        assert_eq!(tier2.len(), 2);
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

    #[test]
    fn volume_change_round_trips_through_progress_json() {
        // The parent dialog mutates `g.progress.volume` and then calls
        // `progress::save`. Serialization is the persistence boundary; if
        // the field doesn't survive a JSON round-trip, the slider won't
        // either.
        let mut g = Game::new(dict(), Progress::default(), Some(42));
        g.progress.volume = 25;
        let saved = g.progress.to_json();
        let restored = Progress::from_json(&saved).expect("must parse");
        assert_eq!(restored.volume, 25);
    }
}
