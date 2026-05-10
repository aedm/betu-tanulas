pub mod audio;
pub mod game;
pub mod i18n;
pub mod level_select;
pub mod menu;
pub mod progress;
pub mod puzzle;
pub mod puzzle_screen;
pub mod screen;
pub mod word;

use dioxus::prelude::*;

pub use game::Game;
pub use level_select::LevelSelect;
pub use menu::{MainMenu, ParentDialog};
pub use progress::Progress;
pub use puzzle::{DropOutcome, Puzzle, Tile, TileState, shuffle};
pub use puzzle_screen::PuzzleScreen;
pub use screen::Screen;
pub use word::{Word, load_words};

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn App() -> Element {
    let game = use_signal(|| Game::new(load_words(), progress::load(), None));

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        main {
            class: "betu-app",
            {
                let screen = game.read().screen;
                match screen {
                    Screen::Menu => rsx! { MainMenu { game } },
                    Screen::LevelSelect { tier } => rsx! { LevelSelect { game, tier } },
                    Screen::Puzzle => rsx! { PuzzleScreen { game } },
                }
            }
        }
    }
}
