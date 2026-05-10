//! SSR-renders the parent zone modal in isolation. The dialog is hidden
//! behind a triple-tap on the menu title in production; tests render it
//! directly so the slider markup, default-state buttons, and reset
//! confirmation can be asserted without driving DOM events.
//!
//! What's not covered here (intentionally):
//! - The triple-tap entry — covered by inspecting MainMenu's initial
//!   render in `menu_render.rs` (slider must be hidden) and by the
//!   `navigation_flow.rs` reset-progress path (drives `Game` directly).
//! - The actual `<input type="range">` interaction — pointer/keyboard
//!   driving belongs to the `betu-10` Playwright suite.

use betu_tanulas::ParentDialog;
use betu_tanulas::audio::{VOLUME_DEFAULT, VOLUME_MAX};
use dioxus::prelude::*;

fn render(volume: u32) -> String {
    let mut dom = VirtualDom::new_with_props(Harness, HarnessProps { volume });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn Harness(volume: u32) -> Element {
    rsx! {
        ParentDialog {
            volume,
            on_volume_change: |_v: u32| {},
            on_reset: |_| {},
            on_close: |_| {},
        }
    }
}

#[test]
fn renders_with_a_volume_slider_bound_to_progress_volume() {
    let html = render(VOLUME_DEFAULT);
    assert!(
        html.contains(r#"data-testid="parent-dialog""#),
        "dialog wrapper must be present; got {html}"
    );
    assert!(
        html.contains(r#"data-testid="volume-slider""#),
        "volume slider must be present in parent dialog; got {html}"
    );
    assert!(
        html.contains(r#"type="range""#),
        "slider should be a range input; got {html}"
    );
    assert!(
        html.contains(r#"min="0""#) && html.contains(&format!(r#"max="{VOLUME_MAX}""#)),
        "slider bounds must be 0..={VOLUME_MAX}; got {html}"
    );
    assert!(
        html.contains(&format!(r#"value="{VOLUME_DEFAULT}""#)),
        "slider value must reflect the passed-in volume ({VOLUME_DEFAULT}); got {html}"
    );
}

#[test]
fn renders_volume_label_text_with_localized_caption() {
    let html = render(35);
    // The label text is "Hangerő: 35" (Hungarian "Volume: 35").
    assert!(
        html.contains("Hangerő"),
        "expected localized volume label; got {html}"
    );
    assert!(
        html.contains(": 35"),
        "expected current volume value beside the label; got {html}"
    );
}

#[test]
fn renders_at_volume_zero_without_panicking() {
    let html = render(0);
    assert!(
        html.contains(r#"value="0""#),
        "slider must render at zero volume; got {html}"
    );
}

#[test]
fn renders_reset_open_button_not_the_confirm_pair_initially() {
    let html = render(VOLUME_DEFAULT);
    assert!(
        html.contains(r#"data-testid="reset-open""#),
        "the dialog opens with a single 'reset' entry-point button; got {html}"
    );
    assert!(
        !html.contains(r#"data-testid="reset-yes""#),
        "the destructive yes-button only appears after the user taps reset; got {html}"
    );
    assert!(
        html.contains(r#"data-testid="parent-close""#),
        "the dialog must offer a way to close without resetting; got {html}"
    );
}

#[test]
fn dialog_title_is_localized_to_hungarian_parent_zone() {
    let html = render(VOLUME_DEFAULT);
    // i18n key menu.parent_zone -> "Szülői beállítások"
    assert!(
        html.contains("Szülői beállítások"),
        "dialog must be titled with the localized parent-zone label; got {html}"
    );
}
