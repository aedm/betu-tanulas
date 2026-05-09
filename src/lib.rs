pub mod game;
pub mod progress;
pub mod puzzle;
pub mod puzzle_screen;
pub mod word;

use dioxus::prelude::*;

pub use game::Game;
pub use progress::Progress;
pub use puzzle::{DropOutcome, Puzzle, Tile, TileState, shuffle};
pub use puzzle_screen::PuzzleScreen;
pub use word::{Word, load_words};

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn App() -> Element {
    let game = use_signal(|| Game::new(load_words(), progress::load(), None));

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        main {
            class: "betu-app",
            PuzzleScreen { game }
        }
    }
}
