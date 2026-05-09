use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Word {
    pub word: String,
    pub emoji: String,
    pub tier: u32,
}

const WORDS_RAW: &str = include_str!("../assets/words.json");

pub fn load_words() -> Vec<Word> {
    serde_json::from_str(WORDS_RAW)
        .expect("assets/words.json must parse; CI words_validation tests guard the schema")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_at_least_one_word_per_tier() {
        let words = load_words();
        for tier in 1..=3 {
            assert!(
                words.iter().any(|w| w.tier == tier),
                "no word found for tier {tier}"
            );
        }
    }
}
