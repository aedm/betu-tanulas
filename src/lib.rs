pub mod puzzle;
pub mod puzzle_screen;
pub mod word;

use dioxus::prelude::*;

pub use puzzle::{Puzzle, Tile, shuffle};
pub use puzzle_screen::PuzzleScreen;
pub use word::{Word, load_words};

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn App() -> Element {
    let words = use_signal(load_words);
    let words_read = words.read();
    let first = words_read.first().cloned();

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        main {
            class: "betu-app",
            if let Some(w) = first {
                PuzzleScreen { word: w }
            } else {
                p { "Nincs szó." }
            }
        }
    }
}
