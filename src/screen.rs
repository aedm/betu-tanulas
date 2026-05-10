//! Top-level navigation state. The `App` component switches sub-screens
//! by reading `Game::screen`. Three flat screens cover v1 — menu →
//! level-select → puzzle — and the kid never goes deeper than two taps
//! from the menu (DESIGN §9, betu-08 spec).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Menu,
    LevelSelect {
        tier: u32,
    },
    Puzzle,
}
