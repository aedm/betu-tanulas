//! Audio-asset integration test.
//!
//! Ensures every cue path the runtime can build (one per letter present
//! in any v1 word, one per word, plus the two SFX) actually resolves to
//! a real `.wav` file in `assets/audio/`. The bundler step in
//! `.github/workflows/ci.yml` then copies those files to
//! `dist/public/audio/` so the served URLs match.
//!
//! If this test fails on a fresh checkout, run `python3 tools/gen_audio.py`
//! from the repo root.

use betu_tanulas::audio::{CHIME_URL, SNAP_URL, letter_url, word_url};
use betu_tanulas::load_words;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn asset_for_url(url: &str) -> PathBuf {
    // URLs are root-relative and shaped like "/audio/...". Map to
    // `assets/audio/...` for the on-disk lookup.
    let stripped = url.trim_start_matches('/');
    let stripped = stripped.strip_prefix("audio/").unwrap_or(stripped);
    repo_root().join("assets").join("audio").join(stripped)
}

#[test]
fn every_letter_in_the_dictionary_has_a_pronunciation_stub() {
    let words = load_words();
    let mut letters: BTreeSet<char> = BTreeSet::new();
    for w in &words {
        for c in w.word.chars() {
            letters.insert(c);
        }
    }
    assert!(!letters.is_empty(), "dictionary must yield letters");
    for c in letters {
        let path = asset_for_url(&letter_url(c));
        assert!(
            path.is_file(),
            "missing letter stub for {c}: {}",
            path.display()
        );
    }
}

#[test]
fn every_word_in_the_dictionary_has_a_pronunciation_stub() {
    let words = load_words();
    for w in &words {
        let path = asset_for_url(&word_url(&w.word));
        assert!(
            path.is_file(),
            "missing word stub for {}: {}",
            w.word,
            path.display()
        );
    }
}

#[test]
fn snap_and_chime_sfx_exist_and_are_non_empty() {
    for url in [SNAP_URL, CHIME_URL] {
        let path = asset_for_url(url);
        let meta = std::fs::metadata(&path)
            .unwrap_or_else(|e| panic!("missing SFX {}: {e}", path.display()));
        assert!(meta.is_file(), "{} must be a file", path.display());
        assert!(
            meta.len() >= 1024,
            "{} should be non-trivially sized (synthesized waveform); got {} bytes",
            path.display(),
            meta.len()
        );
    }
}
