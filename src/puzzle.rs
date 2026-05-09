use crate::word::Word;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tile {
    pub letter: char,
    pub placed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Puzzle {
    pub word: Word,
    pub slots: Vec<Option<char>>,
    pub tiles: Vec<Tile>,
}

impl Puzzle {
    pub fn new(word: Word, seed: Option<u64>) -> Self {
        let len = word.word.chars().count();
        let mut letters: Vec<char> = word.word.chars().collect();
        shuffle(&mut letters, seed);
        let tiles = letters
            .into_iter()
            .map(|letter| Tile {
                letter,
                placed: false,
            })
            .collect();
        Self {
            slots: vec![None; len],
            tiles,
            word,
        }
    }
}

pub fn shuffle<T>(slice: &mut [T], seed: Option<u64>) {
    if slice.len() < 2 {
        return;
    }
    let s = seed.unwrap_or_else(entropy_seed);
    let mut rng = XorShift64::new(s);
    for i in (1..slice.len()).rev() {
        let j = (rng.next() % (i as u64 + 1)) as usize;
        slice.swap(i, j);
    }
}

struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

fn entropy_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now().to_bits()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xDEAD_BEEF_CAFE_F00D)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(word_str: &str, tier: u32) -> Word {
        Word {
            word: word_str.to_string(),
            emoji: "🍎".to_string(),
            tier,
        }
    }

    #[test]
    fn new_creates_one_slot_and_one_tile_per_letter() {
        let p = Puzzle::new(sample("ALMA", 2), Some(42));
        assert_eq!(p.slots.len(), 4);
        assert_eq!(p.tiles.len(), 4);
        assert!(p.slots.iter().all(|s| s.is_none()));
        assert!(p.tiles.iter().all(|t| !t.placed));
    }

    #[test]
    fn tiles_contain_same_letters_as_word() {
        let p = Puzzle::new(sample("ALMA", 2), Some(42));
        let mut got: Vec<char> = p.tiles.iter().map(|t| t.letter).collect();
        let mut want: Vec<char> = "ALMA".chars().collect();
        got.sort();
        want.sort();
        assert_eq!(got, want);
    }

    #[test]
    fn shuffle_with_same_seed_is_deterministic() {
        let mut a: Vec<u32> = (0..10).collect();
        let mut b: Vec<u32> = (0..10).collect();
        shuffle(&mut a, Some(42));
        shuffle(&mut b, Some(42));
        assert_eq!(a, b);
    }

    #[test]
    fn shuffle_actually_permutes() {
        let mut v: Vec<u32> = (0..20).collect();
        let original = v.clone();
        shuffle(&mut v, Some(42));
        assert_ne!(
            v, original,
            "shuffle with seed 42 should not produce identity for n=20"
        );
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, original, "shuffle must be a permutation");
    }

    #[test]
    fn shuffle_different_seeds_differ() {
        let mut a: Vec<u32> = (0..20).collect();
        let mut b: Vec<u32> = (0..20).collect();
        shuffle(&mut a, Some(1));
        shuffle(&mut b, Some(2));
        assert_ne!(a, b);
    }

    #[test]
    fn shuffle_zero_seed_is_safe() {
        let mut v: Vec<u32> = (0..10).collect();
        shuffle(&mut v, Some(0));
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, (0..10).collect::<Vec<_>>());
    }
}
