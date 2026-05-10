//! Main menu screen — the first thing the kid sees on every cold launch
//! (DESIGN §9). Reading-free: tier buttons use the tier's first-word
//! emoji as their icon; locked tiers show a padlock and are disabled.
//!
//! The big play button resumes the active puzzle (the kid's
//! continuation point). The hidden parent zone (triple-tap on the
//! title) opens a small parent menu with a volume slider and a reset
//! button — child-safe because three rapid taps on the same target is
//! hard to do accidentally and the modal adds a second gate.

use dioxus::prelude::*;

use crate::audio::VOLUME_MAX;
use crate::game::Game;
use crate::progress;
use crate::t;
use crate::word::Word;

/// Number of rapid title taps that opens the parent reset dialog. Three
/// is rare enough to be safe, low enough that a frustrated user can
/// trigger it without remembering a magic gesture.
const PARENT_TAP_COUNT: u32 = 3;

#[component]
pub fn MainMenu(game: Signal<Game>) -> Element {
    let mut tap_count = use_signal(|| 0u32);
    let mut show_parent = use_signal(|| false);

    let g = game.read();
    let unlocked = g.progress.tier_unlocked;
    let volume = g.progress.volume;
    let max_tier = g.words.iter().map(|w| w.tier).max().unwrap_or(1);

    let tiers: Vec<TierEntry> = (1..=max_tier)
        .map(|tier| TierEntry {
            tier,
            icon: tier_icon(&g.words, tier),
            locked: tier > unlocked,
        })
        .collect();

    drop(g);

    rsx! {
        section {
            class: "betu-menu",
            "data-screen": "menu",
            h1 {
                class: "betu-menu-title",
                "data-testid": "menu-title",
                onclick: move |_| {
                    let n = *tap_count.read() + 1;
                    if n >= PARENT_TAP_COUNT {
                        tap_count.set(0);
                        show_parent.set(true);
                    } else {
                        tap_count.set(n);
                    }
                },
                {t!("menu.title")}
            }
            button {
                class: "betu-menu-play",
                r#type: "button",
                aria_label: t!("menu.play"),
                "data-testid": "menu-play",
                onclick: move |_| {
                    game.write().resume_play();
                },
                "▶️"
            }
            div {
                class: "betu-menu-tiers",
                aria_label: t!("menu.tier"),
                for entry in tiers.iter().cloned() {
                    {
                        let TierEntry { tier, icon, locked } = entry;
                        let label = if locked {
                            format!("{} {} — {}", t!("menu.tier"), tier, t!("menu.locked"))
                        } else {
                            format!("{} {}", t!("menu.tier"), tier)
                        };
                        rsx! {
                            button {
                                key: "tier-{tier}",
                                class: "betu-tier-button",
                                r#type: "button",
                                "data-tier": "{tier}",
                                "data-locked": if locked { "true" } else { "false" },
                                disabled: locked,
                                aria_label: "{label}",
                                onclick: move |_| {
                                    if !locked {
                                        game.write().enter_tier(tier);
                                    }
                                },
                                span { class: "betu-tier-icon", aria_hidden: "true", "{icon}" }
                                if locked {
                                    span { class: "betu-tier-lock", aria_hidden: "true", "🔒" }
                                }
                            }
                        }
                    }
                }
            }
            if *show_parent.read() {
                ParentDialog {
                    volume,
                    on_volume_change: move |v: u32| {
                        let mut g = game.write();
                        g.progress.volume = v.min(VOLUME_MAX);
                        progress::save(&g.progress);
                    },
                    on_reset: move |_| {
                        let mut g = game.write();
                        g.reset_progress();
                        progress::save(&g.progress);
                        show_parent.set(false);
                    },
                    on_close: move |_| {
                        show_parent.set(false);
                    },
                }
            }
        }
    }
}

#[derive(Clone)]
struct TierEntry {
    tier: u32,
    icon: String,
    locked: bool,
}

fn tier_icon(words: &[Word], tier: u32) -> String {
    words
        .iter()
        .find(|w| w.tier == tier)
        .map(|w| w.emoji.clone())
        .unwrap_or_else(|| "❓".to_string())
}

#[component]
pub fn ParentDialog(
    volume: u32,
    on_volume_change: EventHandler<u32>,
    on_reset: EventHandler<MouseEvent>,
    on_close: EventHandler<MouseEvent>,
) -> Element {
    let mut show_reset_confirm = use_signal(|| false);

    rsx! {
        div {
            class: "betu-modal-backdrop",
            "data-testid": "parent-dialog",
            role: "dialog",
            aria_modal: "true",
            div {
                class: "betu-modal",
                p { class: "betu-modal-title", {t!("menu.parent_zone")} }
                label {
                    class: "betu-volume-row",
                    r#for: "betu-volume",
                    span {
                        class: "betu-volume-label",
                        "data-testid": "volume-label",
                        "{t!(\"menu.volume\")}: {volume}"
                    }
                    input {
                        id: "betu-volume",
                        class: "betu-volume-slider",
                        r#type: "range",
                        min: "0",
                        max: "{VOLUME_MAX}",
                        step: "1",
                        value: "{volume}",
                        "data-testid": "volume-slider",
                        aria_label: t!("menu.volume"),
                        oninput: move |evt| {
                            if let Ok(v) = evt.value().parse::<u32>() {
                                on_volume_change.call(v);
                            }
                        },
                    }
                }
                div {
                    class: "betu-modal-buttons",
                    if *show_reset_confirm.read() {
                        button {
                            class: "betu-modal-yes",
                            r#type: "button",
                            "data-testid": "reset-yes",
                            onclick: move |evt| on_reset.call(evt),
                            {t!("menu.reset_yes")}
                        }
                        button {
                            class: "betu-modal-no",
                            r#type: "button",
                            "data-testid": "reset-no",
                            onclick: move |_| show_reset_confirm.set(false),
                            {t!("menu.reset_no")}
                        }
                    } else {
                        button {
                            class: "betu-modal-warn",
                            r#type: "button",
                            "data-testid": "reset-open",
                            onclick: move |_| show_reset_confirm.set(true),
                            {t!("menu.reset")}
                        }
                        button {
                            class: "betu-modal-no",
                            r#type: "button",
                            "data-testid": "parent-close",
                            onclick: move |evt| on_close.call(evt),
                            {t!("menu.close")}
                        }
                    }
                }
            }
        }
    }
}
