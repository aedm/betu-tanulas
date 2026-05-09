//! Validates assets/words.json against the v1 hard constraints:
//! uppercase A-Z only, no accented characters, no Hungarian digraph
//! sequences, tier N => N+2 letters, every entry has an emoji.

use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Entry {
    word: String,
    emoji: String,
    tier: u32,
}

const ALLOWED: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGRAPHS: &[&str] = &["DZS", "CS", "DZ", "GY", "LY", "NY", "SZ", "TY", "ZS"];

fn load() -> Vec<Entry> {
    let raw = fs::read_to_string("assets/words.json")
        .expect("assets/words.json must exist at repo root");
    serde_json::from_str(&raw).expect("assets/words.json must be valid JSON")
}

#[test]
fn all_words_uppercase_ascii() {
    for e in load() {
        for c in e.word.chars() {
            assert!(
                ALLOWED.contains(c),
                "{:?}: char {:?} not in allowed alphabet (uppercase A-Z, no accents)",
                e.word,
                c
            );
        }
    }
}

#[test]
fn no_digraph_sequences() {
    for e in load() {
        for d in DIGRAPHS {
            assert!(
                !e.word.contains(d),
                "{:?}: contains forbidden digraph sequence {:?}",
                e.word,
                d
            );
        }
    }
}

#[test]
fn tier_letter_count_matches_word_length() {
    for e in load() {
        let expected = (e.tier + 2) as usize;
        let actual = e.word.chars().count();
        assert_eq!(
            actual, expected,
            "{:?}: tier {} expects {} letters, got {}",
            e.word, e.tier, expected, actual
        );
    }
}

#[test]
fn each_entry_has_emoji() {
    for e in load() {
        assert!(!e.emoji.is_empty(), "{:?}: missing emoji", e.word);
    }
}

#[test]
fn no_duplicate_words() {
    let entries = load();
    let mut seen = std::collections::HashSet::new();
    for e in &entries {
        assert!(seen.insert(e.word.clone()), "duplicate word {:?}", e.word);
    }
}
