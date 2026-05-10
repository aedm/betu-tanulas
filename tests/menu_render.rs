//! SSR-renders MainMenu against the real dictionary and verifies its
//! reading-free, icon-driven layout: the title, the play button, one
//! tier button per tier, and a `data-locked="true"` for tiers above
//! `tier_unlocked`. The reset dialog stays hidden until the title is
//! triple-tapped (we exercise that path in the navigation flow test
//! by driving model methods directly, not the UI tap).

use betu_tanulas::{Game, MainMenu, Progress, load_words};
use dioxus::prelude::*;

fn render_menu(progress: Progress) -> String {
    let mut dom = VirtualDom::new_with_props(MenuHarness, MenuHarnessProps { progress });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn MenuHarness(progress: Progress) -> Element {
    let game = use_signal(|| Game::new(load_words(), progress, Some(7)));
    rsx! { MainMenu { game } }
}

#[test]
fn menu_renders_play_button_and_tier_buttons() {
    let html = render_menu(Progress::default());
    assert!(
        html.contains("data-screen=\"menu\""),
        "expected menu screen marker; got {html}"
    );
    assert!(
        html.contains("data-testid=\"menu-play\""),
        "expected play button; got {html}"
    );
    assert!(
        html.contains("data-tier=\"1\""),
        "expected tier 1 button; got {html}"
    );
    assert!(
        html.contains("data-tier=\"2\""),
        "expected tier 2 button; got {html}"
    );
    assert!(
        html.contains("data-tier=\"3\""),
        "expected tier 3 button; got {html}"
    );
}

#[test]
fn locked_tiers_render_with_data_locked_true() {
    let html = render_menu(Progress::default());
    // tier 1 unlocked by default; 2 and 3 locked.
    assert!(
        html.contains(r#"data-tier="1" data-locked="false""#)
            || html.contains(r#"data-locked="false" data-tier="1""#),
        "tier 1 must be unlocked; html: {html}"
    );
    let locked_count = html.matches(r#"data-locked="true""#).count();
    assert_eq!(
        locked_count, 2,
        "expected 2 locked tier buttons (2 and 3); html: {html}"
    );
}

#[test]
fn unlocking_tier_two_renders_only_tier_three_locked() {
    let progress = Progress {
        completed: Vec::new(),
        current_tier: 1,
        tier_unlocked: 2,
    };
    let html = render_menu(progress);
    assert_eq!(
        html.matches(r#"data-locked="true""#).count(),
        1,
        "only tier 3 should be locked; html: {html}"
    );
}

#[test]
fn menu_title_is_localized_to_hungarian() {
    let html = render_menu(Progress::default());
    assert!(
        html.contains("Betűk"),
        "expected localized Hungarian title; got {html}"
    );
}

#[test]
fn reset_dialog_is_not_rendered_initially() {
    let html = render_menu(Progress::default());
    assert!(
        !html.contains(r#"data-testid="reset-dialog""#),
        "reset dialog must be hidden until parent gesture; got {html}"
    );
}
