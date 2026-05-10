//! End-to-end test of the navigation state machine: Menu → LevelSelect
//! → Puzzle → Menu, plus the full reset-progress path. Drives `Game`
//! methods directly (the same calls the UI handlers issue).

use betu_tanulas::{DropOutcome, Game, Progress, Screen, TileState, load_words};

fn solve(g: &mut Game) {
    let word = g.current_word().word.clone();
    let centers: Vec<(f64, f64)> = (0..word.chars().count())
        .map(|i| (100.0 + i as f64 * 100.0, 200.0))
        .collect();
    for (pid, (slot_idx, expected)) in (1000i32..).zip(word.chars().enumerate()) {
        let tile_idx = g
            .current_puzzle
            .tiles
            .iter()
            .enumerate()
            .find(|(_, t)| t.letter == expected && matches!(t.state, TileState::Idle))
            .map(|(i, _)| i)
            .expect("idle matching tile");
        assert!(
            g.current_puzzle
                .pickup(tile_idx, pid, centers[slot_idx], (0.0, 0.0))
        );
        let outcome = g.current_puzzle.release(pid, &centers, 40.0);
        assert!(matches!(outcome, DropOutcome::Snapped { .. }));
    }
}

#[test]
fn fresh_game_starts_on_the_menu_screen() {
    let g = Game::new(load_words(), Progress::default(), Some(1));
    assert_eq!(g.screen, Screen::Menu);
}

#[test]
fn play_button_dispatches_to_puzzle_screen() {
    let mut g = Game::new(load_words(), Progress::default(), Some(1));
    g.resume_play();
    assert_eq!(g.screen, Screen::Puzzle);
}

#[test]
fn tap_unlocked_tier_routes_to_level_select() {
    let mut g = Game::new(load_words(), Progress::default(), Some(1));
    g.enter_tier(1);
    assert_eq!(g.screen, Screen::LevelSelect { tier: 1 });
}

#[test]
fn tap_locked_tier_does_not_navigate() {
    let mut g = Game::new(load_words(), Progress::default(), Some(1));
    g.enter_tier(2);
    assert_eq!(g.screen, Screen::Menu);
    g.enter_tier(3);
    assert_eq!(g.screen, Screen::Menu);
}

#[test]
fn tap_word_in_level_select_starts_puzzle_for_that_word() {
    let mut g = Game::new(load_words(), Progress::default(), Some(1));
    g.enter_tier(1);
    g.start_word("APA");
    assert_eq!(g.screen, Screen::Puzzle);
    assert_eq!(g.current_word().word, "APA");
}

#[test]
fn home_icon_returns_to_menu_without_resetting_active_puzzle() {
    let mut g = Game::new(load_words(), Progress::default(), Some(1));
    g.start_word("APA");
    let active = g.current_word().word.clone();
    g.go_to_menu();
    assert_eq!(g.screen, Screen::Menu);
    assert_eq!(
        g.current_word().word,
        active,
        "going home should preserve the in-flight puzzle"
    );
}

#[test]
fn full_round_trip_menu_tier_word_solve_advance() {
    let mut g = Game::new(load_words(), Progress::default(), Some(99));
    g.enter_tier(1);
    assert!(matches!(g.screen, Screen::LevelSelect { tier: 1 }));

    let first_word = "APA";
    g.start_word(first_word);
    assert_eq!(g.screen, Screen::Puzzle);
    solve(&mut g);
    assert!(g.is_won());

    g.advance_to_next();
    assert_eq!(g.screen, Screen::Puzzle);
    assert!(g.progress.completed.contains(&first_word.to_string()));
    assert_ne!(g.current_word().word, first_word);

    g.go_to_menu();
    assert_eq!(g.screen, Screen::Menu);
}

#[test]
fn reset_progress_clears_completion_and_returns_to_menu() {
    let mut g = Game::new(load_words(), Progress::default(), Some(99));
    g.start_word("APA");
    solve(&mut g);
    g.advance_to_next();
    assert!(!g.progress.completed.is_empty());
    g.reset_progress();
    assert!(g.progress.completed.is_empty());
    assert_eq!(g.progress.tier_unlocked, 1);
    assert_eq!(g.screen, Screen::Menu);
}
