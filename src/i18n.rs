//! Tiny translation shim. v1 ships Hungarian only; future locales drop
//! in by extending [`translate`] without touching call sites. Use the
//! `t!` macro for compile-time validation that the key is a string
//! literal.

#[macro_export]
macro_rules! t {
    ($key:literal) => {
        $crate::i18n::translate($key)
    };
}

pub fn translate(key: &'static str) -> &'static str {
    match key {
        "menu.title" => "Betűk",
        "menu.play" => "Játék",
        "menu.tier" => "Szint",
        "menu.locked" => "Még zárva",
        "menu.parent_zone" => "Szülői beállítások",
        "menu.reset" => "Előrehaladás törlése",
        "menu.reset_confirm" => "Biztos törlöd?",
        "menu.reset_yes" => "Igen",
        "menu.reset_no" => "Mégse",
        "level_select.back" => "Vissza",
        "level_select.title" => "Válassz szót",
        "puzzle.next" => "Következő",
        "puzzle.home" => "Főmenü",
        "puzzle.progress" => "Haladás",
        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_keys_translate_to_hungarian_strings() {
        assert_eq!(translate("puzzle.next"), "Következő");
        assert_eq!(translate("menu.play"), "Játék");
        assert_eq!(translate("menu.title"), "Betűk");
    }

    #[test]
    fn unknown_keys_fall_back_to_the_key_itself() {
        // Useful in development: a missing key surfaces as the key name
        // in the rendered UI rather than an empty string.
        assert_eq!(translate("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn macro_resolves_known_key() {
        assert_eq!(t!("puzzle.next"), "Következő");
    }
}
