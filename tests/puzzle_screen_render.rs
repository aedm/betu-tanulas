//! SSR-renders PuzzleScreen for a known word and asserts the DOM contains
//! the emoji, N empty slots, and N tiles whose letters are a permutation of
//! the word. Uses dioxus-ssr (no browser).

use betu_tanulas::{Game, Progress, PuzzleScreen, Word, load_words};
use dioxus::prelude::*;

fn game_for(word: Word) -> Game {
    let progress = Progress {
        completed: Vec::new(),
        current_tier: word.tier,
        tier_unlocked: word.tier,
    };
    Game::new(vec![word], progress, Some(7))
}

fn render_for(word: Word) -> String {
    let mut dom =
        VirtualDom::new_with_props(ScreenHarness, ScreenHarnessProps { word: word.clone() });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn ScreenHarness(word: Word) -> Element {
    let game = use_signal(|| game_for(word));
    rsx! { PuzzleScreen { game } }
}

#[test]
fn renders_emoji_and_correct_slot_and_tile_count_for_alma() {
    let alma = load_words()
        .into_iter()
        .find(|w| w.word == "ALMA")
        .expect("ALMA must be in words.json");
    let expected_len = alma.word.chars().count();
    let html = render_for(alma);

    assert!(
        html.contains("🍎"),
        "expected emoji 🍎 in rendered HTML; got {html}"
    );
    assert_eq!(
        html.matches("data-filled=").count(),
        expected_len,
        "expected {expected_len} slot cells; html: {html}"
    );
    assert_eq!(
        html.matches("data-placed=").count(),
        expected_len,
        "expected {expected_len} tile cells; html: {html}"
    );
    for c in "ALMA".chars() {
        assert!(
            html.contains(c.to_string().as_str()),
            "expected letter {c:?} somewhere in rendered HTML; got {html}"
        );
    }
}

#[test]
fn slots_render_empty_initially() {
    let cica = load_words()
        .into_iter()
        .find(|w| w.word == "CICA")
        .expect("CICA must be in words.json");
    let html = render_for(cica);
    let empty_count = html.matches(r#"data-filled="false""#).count();
    assert_eq!(
        empty_count, 4,
        "all 4 CICA slots should start empty; html: {html}"
    );
}

#[test]
fn renders_for_a_5_letter_word_without_panicking() {
    let labda = load_words()
        .into_iter()
        .find(|w| w.word == "LABDA")
        .expect("LABDA must be in words.json");
    let html = render_for(labda);
    assert_eq!(html.matches("data-filled=").count(), 5);
    assert_eq!(html.matches("data-placed=").count(), 5);
    assert!(html.contains("⚽"));
}

#[test]
fn unsolved_screen_does_not_render_win_overlay_or_next_button() {
    let alma = load_words()
        .into_iter()
        .find(|w| w.word == "ALMA")
        .expect("ALMA must be in words.json");
    let html = render_for(alma);
    assert!(
        html.contains(r#"data-won="false""#),
        "expected data-won=\"false\" in initial render; got {html}"
    );
    assert!(
        !html.contains("betu-next"),
        "expected no Next button before win; got {html}"
    );
    assert!(
        !html.contains("betu-emoji-rain"),
        "expected no confetti rain before win; got {html}"
    );
}
