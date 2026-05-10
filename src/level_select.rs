//! Level-select screen — a flat grid of every word in the chosen tier.
//! Tap a word tile to drop into the puzzle for that word. Completed
//! words show their emoji; never-completed words show a question mark
//! so the kid keeps a sense of mystery (DESIGN §9 / betu-08 spec).

use dioxus::prelude::*;

use crate::game::Game;
use crate::t;

#[component]
pub fn LevelSelect(game: Signal<Game>, tier: u32) -> Element {
    let g = game.read();
    let tier_words: Vec<TileEntry> = g
        .words_in_tier(tier)
        .iter()
        .map(|w| TileEntry {
            word: w.word.clone(),
            emoji: w.emoji.clone(),
            completed: g.is_completed(&w.word),
        })
        .collect();
    drop(g);

    rsx! {
        section {
            class: "betu-level-select",
            "data-screen": "level-select",
            "data-tier": "{tier}",
            div {
                class: "betu-header",
                button {
                    class: "betu-back",
                    r#type: "button",
                    aria_label: t!("level_select.back"),
                    "data-testid": "level-select-back",
                    onclick: move |_| {
                        game.write().go_to_menu();
                    },
                    "⬅️"
                }
                h2 {
                    class: "betu-header-title",
                    "{t!(\"menu.tier\")} {tier}"
                }
                span { class: "betu-header-spacer", aria_hidden: "true" }
            }
            div {
                class: "betu-word-grid",
                aria_label: t!("level_select.title"),
                for entry in tier_words.iter().cloned() {
                    {
                        let TileEntry { word, emoji, completed } = entry;
                        let display = if completed { emoji.clone() } else { "❓".to_string() };
                        let label = if completed {
                            word.clone()
                        } else {
                            format!("{} (?)", t!("level_select.title"))
                        };
                        rsx! {
                            button {
                                key: "word-{word}",
                                class: "betu-word-tile",
                                r#type: "button",
                                "data-word": "{word}",
                                "data-completed": if completed { "true" } else { "false" },
                                aria_label: "{label}",
                                onclick: move |_| {
                                    game.write().start_word(&word);
                                },
                                span { class: "betu-word-emoji", aria_hidden: "true", "{display}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct TileEntry {
    word: String,
    emoji: String,
    completed: bool,
}
