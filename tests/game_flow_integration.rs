//! Drives `Game` through the same call sequence the UI issues — solve a
//! word via puzzle drag/release, observe `is_won` flip, tap "Next" via
//! `advance_to_next`, observe a different word loaded — without any DOM.

use betu_tanulas::{DropOutcome, Game, Progress, TileState, Word, load_words};

fn solve_current_word(g: &mut Game) {
    let word = g.current_word().word.clone();
    let centers: Vec<(f64, f64)> = (0..word.chars().count())
        .map(|i| (100.0 + i as f64 * 100.0, 200.0))
        .collect();
    for (pid, (slot_idx, expected)) in (100i32..).zip(word.chars().enumerate()) {
        let tile_idx = g
            .current_puzzle
            .tiles
            .iter()
            .enumerate()
            .find(|(_, t)| t.letter == expected && matches!(t.state, TileState::Idle))
            .map(|(i, _)| i)
            .expect("expected matching idle tile");
        let center = centers[slot_idx];
        assert!(g.current_puzzle.pickup(tile_idx, pid, center, (0.0, 0.0)));
        let outcome = g.current_puzzle.release(pid, &centers, 40.0);
        assert!(matches!(outcome, DropOutcome::Snapped { .. }));
    }
}

#[test]
fn full_loop_solve_advance_solve_advance() {
    let mut g = Game::new(load_words(), Progress::default(), Some(123));
    let first = g.current_word().word.clone();

    assert!(!g.is_won());
    solve_current_word(&mut g);
    assert!(g.is_won(), "after all letters placed, game must be won");

    g.advance_to_next();
    assert!(!g.is_won(), "advance_to_next clears the won state");
    assert_ne!(g.current_word().word, first);
    assert!(g.progress.completed.contains(&first));

    let second = g.current_word().word.clone();
    solve_current_word(&mut g);
    g.advance_to_next();
    assert!(g.progress.completed.contains(&second));
    assert_eq!(g.progress.completed.len(), 2);
}

#[test]
fn five_completions_unlock_tier_two_against_real_dictionary() {
    let mut g = Game::new(load_words(), Progress::default(), Some(456));
    for _ in 0..5 {
        solve_current_word(&mut g);
        g.advance_to_next();
    }
    assert_eq!(g.progress.tier_unlocked, 2);
    assert!(g.progress.completed.iter().all(|w| {
        load_words()
            .iter()
            .find(|x| &x.word == w)
            .map(|x| x.tier == 1)
            .unwrap_or(false)
    }));
}

#[test]
fn restored_progress_keeps_unlocked_tier_and_completed_list() {
    // Simulate: kid plays one session, advances 6 words, saves; next session
    // starts from the same persisted state via JSON.
    let mut g = Game::new(load_words(), Progress::default(), Some(789));
    for _ in 0..3 {
        solve_current_word(&mut g);
        g.advance_to_next();
    }
    let saved_json = g.progress.to_json();

    let restored = Progress::from_json(&saved_json).expect("must round-trip");
    let g2 = Game::new(load_words(), restored.clone(), Some(789));
    assert_eq!(g2.progress.completed, restored.completed);
    assert_eq!(g2.progress.tier_unlocked, restored.tier_unlocked);
    // Different seed counter would advance differently — verify by seed.
    assert_eq!(g2.current_word().tier, restored.current_tier);

    // The kid can still keep playing from the new Game. The just-solved
    // word ends up in completed (set semantics: dedup if already there,
    // which is legal — a fresh session reshuffles all tier-1 words,
    // including ones the kid has solved in a prior session).
    let mut g2 = g2;
    let solved = g2.current_word().word.clone();
    solve_current_word(&mut g2);
    g2.advance_to_next();
    assert!(
        g2.progress.completed.contains(&solved),
        "expected just-solved word {solved} to be in completed: {:?}",
        g2.progress.completed
    );
}

#[test]
fn tier_unlocks_persist_across_a_save_load_cycle() {
    // Same as above but specifically asserts the tier_unlocked bit.
    let mut p = Progress::default();
    let words: Vec<Word> = load_words().into_iter().filter(|w| w.tier == 1).collect();
    for w in words.iter().take(5) {
        p.record_completion(&w.word);
    }
    p.recompute_tier_unlock(&load_words());
    assert_eq!(p.tier_unlocked, 2);

    let json = p.to_json();
    let restored = Progress::from_json(&json).expect("must round-trip");
    assert_eq!(restored.tier_unlocked, 2);
}
