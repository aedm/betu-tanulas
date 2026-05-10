//! Persistent progress (`localStorage` on wasm, no-op on native) per
//! DESIGN.md §5: `betu/progress/v1` schema with `completed`, `current_tier`,
//! `tier_unlocked`. Tier unlock rule: a kid who has completed `N_UNLOCK`
//! words from tier `N` unlocks tier `N+1`.

use serde::{Deserialize, Serialize};

use crate::audio::VOLUME_DEFAULT;
use crate::word::Word;

pub const STORAGE_KEY: &str = "betu/progress/v1";
pub const N_UNLOCK: u32 = 5;

fn default_volume() -> u32 {
    VOLUME_DEFAULT
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    pub completed: Vec<String>,
    #[serde(rename = "currentTier")]
    pub current_tier: u32,
    #[serde(rename = "tierUnlocked")]
    pub tier_unlocked: u32,
    /// Master volume, `0..=100`. Stored alongside progress so the parent
    /// only sets it once. `#[serde(default)]` keeps pre-`betu-09` saves
    /// loadable: missing field falls back to [`VOLUME_DEFAULT`].
    #[serde(default = "default_volume")]
    pub volume: u32,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            completed: Vec::new(),
            current_tier: 1,
            tier_unlocked: 1,
            volume: VOLUME_DEFAULT,
        }
    }
}

impl Progress {
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn record_completion(&mut self, word: &str) {
        if !self.completed.iter().any(|w| w == word) {
            self.completed.push(word.to_string());
        }
    }

    /// Set `tier_unlocked` to the highest `N+1` for which `>= N_UNLOCK`
    /// tier-`N` words appear in `completed`. Monotonic: never decreases.
    pub fn recompute_tier_unlock(&mut self, words: &[Word]) {
        let max_tier = words.iter().map(|w| w.tier).max().unwrap_or(1);
        for tier in 1..=max_tier {
            let in_tier = words.iter().filter(|w| w.tier == tier);
            let done = in_tier
                .filter(|w| self.completed.iter().any(|c| c == &w.word))
                .count() as u32;
            if done >= N_UNLOCK {
                self.tier_unlocked = self.tier_unlocked.max(tier + 1);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn load() -> Progress {
    let Some(window) = web_sys::window() else {
        return Progress::default();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return Progress::default();
    };
    match storage.get_item(STORAGE_KEY) {
        Ok(Some(s)) => Progress::from_json(&s).unwrap_or_default(),
        _ => Progress::default(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> Progress {
    Progress::default()
}

#[cfg(target_arch = "wasm32")]
pub fn save(p: &Progress) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let _ = storage.set_item(STORAGE_KEY, &p.to_json());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save(_p: &Progress) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::VOLUME_MAX;

    fn w(word: &str, tier: u32) -> Word {
        Word {
            word: word.to_string(),
            emoji: "🐱".to_string(),
            tier,
        }
    }

    #[test]
    fn default_starts_at_tier_one_with_only_tier_one_unlocked() {
        let p = Progress::default();
        assert_eq!(p.current_tier, 1);
        assert_eq!(p.tier_unlocked, 1);
        assert!(p.completed.is_empty());
    }

    #[test]
    fn json_roundtrip_preserves_all_fields() {
        let p = Progress {
            completed: vec!["CICA".into(), "ALMA".into()],
            current_tier: 2,
            tier_unlocked: 2,
            volume: 42,
        };
        let json = p.to_json();
        let back = Progress::from_json(&json).expect("must parse own output");
        assert_eq!(p, back);
    }

    #[test]
    fn json_uses_camel_case_keys_compatible_with_design_doc() {
        let p = Progress {
            completed: vec!["CICA".into()],
            current_tier: 2,
            tier_unlocked: 3,
            volume: VOLUME_DEFAULT,
        };
        let json = p.to_json();
        assert!(
            json.contains("\"currentTier\":2"),
            "expected camelCase currentTier in {json}"
        );
        assert!(
            json.contains("\"tierUnlocked\":3"),
            "expected camelCase tierUnlocked in {json}"
        );
    }

    #[test]
    fn pre_betu_09_save_loads_with_default_volume() {
        // A localStorage payload written before the volume field existed
        // must still parse, with volume falling back to VOLUME_DEFAULT so
        // the audio cues stay audible after the upgrade.
        let legacy = r#"{"completed":["CICA"],"currentTier":1,"tierUnlocked":2}"#;
        let p = Progress::from_json(legacy).expect("legacy v1 saves must still parse");
        assert_eq!(p.volume, VOLUME_DEFAULT);
        assert_eq!(p.completed, vec!["CICA".to_string()]);
        assert_eq!(p.tier_unlocked, 2);
    }

    #[test]
    fn default_volume_is_audible_default() {
        let p = Progress::default();
        assert_eq!(p.volume, VOLUME_DEFAULT);
        assert!(p.volume > 0, "default must be audible, not muted");
        assert!(p.volume <= VOLUME_MAX, "default must be in 0..=VOLUME_MAX");
    }

    #[test]
    fn record_completion_dedups() {
        let mut p = Progress::default();
        p.record_completion("CICA");
        p.record_completion("CICA");
        p.record_completion("ALMA");
        assert_eq!(p.completed, vec!["CICA".to_string(), "ALMA".to_string()]);
    }

    #[test]
    fn recompute_unlocks_tier_two_after_n_unlock_tier_one_completions() {
        let words = vec![
            w("AB", 1),
            w("AC", 1),
            w("AD", 1),
            w("AE", 1),
            w("AF", 1),
            w("AG", 1),
        ];
        let mut p = Progress {
            completed: vec!["AB".into(), "AC".into(), "AD".into(), "AE".into()],
            current_tier: 1,
            tier_unlocked: 1,
            volume: VOLUME_DEFAULT,
        };
        p.recompute_tier_unlock(&words);
        assert_eq!(p.tier_unlocked, 1, "4 < 5: still locked");
        p.completed.push("AF".into());
        p.recompute_tier_unlock(&words);
        assert_eq!(p.tier_unlocked, 2, "5 reached: tier 2 unlocks");
    }

    #[test]
    fn recompute_is_monotonic_never_relocks() {
        let words = vec![w("AB", 1)];
        let mut p = Progress {
            completed: vec![],
            current_tier: 1,
            tier_unlocked: 3, // already unlocked further (e.g. via past play)
            volume: VOLUME_DEFAULT,
        };
        p.recompute_tier_unlock(&words);
        assert_eq!(
            p.tier_unlocked, 3,
            "must not relock previously unlocked tiers"
        );
    }

    #[test]
    fn malformed_json_returns_default_via_load_path() {
        // We can't invoke wasm `load` here, but we can exercise the same
        // fallback: from_json returns None, caller falls back to default.
        let parsed = Progress::from_json("{ this is not json").unwrap_or_default();
        assert_eq!(parsed, Progress::default());
    }
}
