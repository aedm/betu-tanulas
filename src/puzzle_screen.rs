use dioxus::prelude::*;

use crate::puzzle::Puzzle;
use crate::word::Word;

#[component]
pub fn PuzzleScreen(word: Word) -> Element {
    let initial = Puzzle::new(word.clone(), None);
    let puzzle = use_signal(|| initial);
    let p = puzzle.read();

    rsx! {
        section {
            class: "betu-screen",
            "data-word": "{p.word.word}",
            div {
                class: "betu-emoji",
                role: "img",
                aria_label: "{p.word.word}",
                "{p.word.emoji}"
            }
            div {
                class: "betu-row betu-slots",
                aria_label: "slots",
                for (idx, slot) in p.slots.iter().enumerate() {
                    div {
                        key: "slot-{idx}",
                        class: "betu-cell betu-slot",
                        "data-filled": if slot.is_some() { "true" } else { "false" },
                        {slot.as_ref().map(|c| c.to_string()).unwrap_or_default()}
                    }
                }
            }
            div {
                class: "betu-row betu-tiles",
                aria_label: "tiles",
                for (idx, tile) in p.tiles.iter().enumerate() {
                    div {
                        key: "tile-{idx}",
                        class: "betu-cell betu-tile",
                        "data-placed": if tile.placed { "true" } else { "false" },
                        "{tile.letter}"
                    }
                }
            }
        }
    }
}
