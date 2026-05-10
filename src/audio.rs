//! Audio cue playback (DESIGN.md §7 + §20).
//!
//! Stateless helpers that map a game event (tile pickup, snap, win) to a
//! short HTML5 `<audio>` play. On native, every entry point is a no-op so
//! tests run without a browser.
//!
//! Asset URLs are root-relative (`/audio/letter/A.wav`, etc.) and resolved
//! by Cloudflare Pages — the bundler step copies `assets/audio/` to
//! `dist/public/audio/` so this works without any Dioxus `asset!()`
//! wiring per file.
//!
//! Volume comes from [`crate::progress::Progress::volume`] (`0..=100`),
//! converted via [`volume_to_unit`] to the `[0.0, 1.0]` range
//! `HtmlAudioElement::set_volume` expects. `volume == 0` short-circuits:
//! we don't even create the element.

/// Maximum value for the user-facing volume scale.
pub const VOLUME_MAX: u32 = 100;

/// Default volume on first launch — quiet enough that the parent isn't
/// startled, loud enough that the kid hears the snap.
pub const VOLUME_DEFAULT: u32 = 70;

/// Convert a `0..=100` user volume into the `[0.0, 1.0]` HTMLAudioElement
/// scale. Out-of-range inputs clamp to the nearest end.
pub fn volume_to_unit(v: u32) -> f64 {
    let clamped = v.min(VOLUME_MAX) as f64;
    clamped / VOLUME_MAX as f64
}

/// URL for the letter-name pronunciation stub. Letter is uppercased
/// because filenames are uppercase and our word data is uppercase-only.
pub fn letter_url(c: char) -> String {
    let upper: String = c.to_uppercase().collect();
    format!("/audio/letter/{upper}.wav")
}

/// URL for a whole-word pronunciation stub.
pub fn word_url(word: &str) -> String {
    let upper = word.to_uppercase();
    format!("/audio/word/{upper}.wav")
}

/// URL for the soft snap-into-slot click.
pub const SNAP_URL: &str = "/audio/sfx/snap.wav";

/// URL for the win-flow chime.
pub const CHIME_URL: &str = "/audio/sfx/chime.wav";

#[cfg(target_arch = "wasm32")]
fn play_url(url: &str, volume: u32) {
    if volume == 0 {
        return;
    }
    if let Ok(audio) = web_sys::HtmlAudioElement::new_with_src(url) {
        audio.set_volume(volume_to_unit(volume));
        // `play()` returns a promise; we ignore it. iOS Safari may reject
        // before the first user gesture — that's expected on cold start.
        let _ = audio.play();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn play_url(_url: &str, _volume: u32) {}

pub fn play_letter(c: char, volume: u32) {
    play_url(&letter_url(c), volume);
}

pub fn play_word(word: &str, volume: u32) {
    play_url(&word_url(word), volume);
}

pub fn play_snap(volume: u32) {
    play_url(SNAP_URL, volume);
}

pub fn play_chime(volume: u32) {
    play_url(CHIME_URL, volume);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_zero_maps_to_zero_unit() {
        assert_eq!(volume_to_unit(0), 0.0);
    }

    #[test]
    fn volume_max_maps_to_one_unit() {
        assert_eq!(volume_to_unit(VOLUME_MAX), 1.0);
    }

    #[test]
    fn volume_above_max_clamps_to_one_unit() {
        assert_eq!(volume_to_unit(250), 1.0);
    }

    #[test]
    fn volume_default_is_in_audible_range() {
        let unit = volume_to_unit(VOLUME_DEFAULT);
        assert!(
            unit > 0.0 && unit < 1.0,
            "default volume must be audible but not max; got {unit}"
        );
    }

    #[test]
    fn letter_url_format_matches_bundled_assets() {
        assert_eq!(letter_url('A'), "/audio/letter/A.wav");
        assert_eq!(letter_url('Z'), "/audio/letter/Z.wav");
    }

    #[test]
    fn letter_url_uppercases_lowercase_input() {
        // Defense against future call sites that pass tile.letter unchanged
        // even if a future word source switches to lowercase.
        assert_eq!(letter_url('a'), "/audio/letter/A.wav");
    }

    #[test]
    fn word_url_format_matches_bundled_assets() {
        assert_eq!(word_url("CICA"), "/audio/word/CICA.wav");
        assert_eq!(word_url("LABDA"), "/audio/word/LABDA.wav");
    }

    #[test]
    fn word_url_uppercases_lowercase_input() {
        assert_eq!(word_url("alma"), "/audio/word/ALMA.wav");
    }

    #[test]
    fn sfx_urls_are_under_audio_sfx() {
        assert_eq!(SNAP_URL, "/audio/sfx/snap.wav");
        assert_eq!(CHIME_URL, "/audio/sfx/chime.wav");
    }

    #[test]
    fn native_play_calls_are_no_ops() {
        // Smoke test: on the native test target these helpers must not
        // panic, even with weird inputs.
        play_letter('A', 50);
        play_word("CICA", 0);
        play_snap(100);
        play_chime(VOLUME_DEFAULT);
    }
}
