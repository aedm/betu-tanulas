//! SSR-renders LevelSelect against the real dictionary. The grid must
//! produce one tile per word in the chosen tier, completed words show
//! `data-completed="true"` (and their emoji), and never-completed
//! words are rendered with `?` so the kid keeps a sense of mystery.

use betu_tanulas::audio::VOLUME_DEFAULT;
use betu_tanulas::{Game, LevelSelect, Progress, load_words};
use dioxus::prelude::*;

fn render(progress: Progress, tier: u32) -> String {
    let mut dom = VirtualDom::new_with_props(
        LevelSelectHarness,
        LevelSelectHarnessProps { progress, tier },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn LevelSelectHarness(progress: Progress, tier: u32) -> Element {
    let game = use_signal(|| Game::new(load_words(), progress, Some(7)));
    rsx! { LevelSelect { game, tier } }
}

#[test]
fn renders_one_tile_per_word_in_the_chosen_tier() {
    let html = render(Progress::default(), 1);
    let count = html.matches(r#"class="betu-word-tile""#).count();
    assert_eq!(count, 12, "expected 12 tier-1 word tiles; html: {html}");
    assert!(
        html.contains(r#"data-tier="1""#),
        "level-select must mark its tier; html: {html}"
    );
}

#[test]
fn never_completed_words_show_question_mark() {
    let html = render(Progress::default(), 1);
    let q_count = html.matches('❓').count();
    assert!(
        q_count >= 12,
        "expected at least 12 question marks (one per tile); html: {html}"
    );
}

#[test]
fn completed_words_show_their_emoji_and_completed_attr() {
    // APA (apa = "father") is the first tier-1 word in words.json.
    let progress = Progress {
        completed: vec!["APA".to_string()],
        current_tier: 1,
        tier_unlocked: 1,
        volume: VOLUME_DEFAULT,
    };
    let html = render(progress, 1);
    assert!(
        html.contains(r#"data-word="APA" data-completed="true""#)
            || html.contains(r#"data-completed="true" data-word="APA""#),
        "expected APA tile marked completed; html: {html}"
    );
    assert!(
        html.contains("👨"),
        "expected APA emoji 👨 because completed; html: {html}"
    );
    assert!(
        html.matches('❓').count() >= 11,
        "11 other tier-1 words still uncompleted; html: {html}"
    );
}

#[test]
fn back_button_is_present_and_localized() {
    let html = render(Progress::default(), 1);
    assert!(
        html.contains(r#"data-testid="level-select-back""#),
        "back button must be present; got {html}"
    );
    assert!(
        html.contains("⬅️"),
        "back button uses arrow emoji; got {html}"
    );
}

#[test]
fn tier_two_grid_renders_when_unlocked() {
    let progress = Progress {
        completed: Vec::new(),
        current_tier: 1,
        tier_unlocked: 2,
        volume: VOLUME_DEFAULT,
    };
    let html = render(progress, 2);
    let count = html.matches(r#"class="betu-word-tile""#).count();
    assert_eq!(count, 16, "expected 16 tier-2 word tiles; html: {html}");
}
