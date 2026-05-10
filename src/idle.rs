//! Idle-replay + slot-tap repeat-instruction state (DESIGN.md §3 + §7).
//!
//! Pure model: holds the wall-clock timestamp of the most recent input
//! plus two cumulative counters (idle replays fired by the timer, slot
//! taps initiated by the kid). `puzzle_screen` polls [`should_replay`]
//! on a 1 s interval in the wasm runtime and calls [`note_replay`] when
//! it triggers the audio cue. Pointer events on the screen call
//! [`note_input`]. Slot taps call [`note_slot_tap`]. Tests drive this
//! struct directly with synthetic timestamps — no DOM, no `Date::now`.

/// Threshold for the idle audio replay (DESIGN §3: ~10 s of no input).
pub const IDLE_REPLAY_THRESHOLD_MS: f64 = 10_000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IdleReplay {
    pub last_input_ms: f64,
    pub idle_replays: u32,
    pub slot_replays: u32,
}

impl IdleReplay {
    pub fn new(now_ms: f64) -> Self {
        Self {
            last_input_ms: now_ms,
            idle_replays: 0,
            slot_replays: 0,
        }
    }

    pub fn note_input(&mut self, now_ms: f64) {
        self.last_input_ms = now_ms;
    }

    pub fn should_replay(&self, now_ms: f64, threshold_ms: f64) -> bool {
        now_ms - self.last_input_ms >= threshold_ms
    }

    pub fn note_replay(&mut self, now_ms: f64) {
        self.last_input_ms = now_ms;
        self.idle_replays = self.idle_replays.saturating_add(1);
    }

    pub fn note_slot_tap(&mut self, now_ms: f64) {
        self.last_input_ms = now_ms;
        self.slot_replays = self.slot_replays.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_state_has_zero_counters_and_no_replay() {
        let s = IdleReplay::new(1000.0);
        assert_eq!(s.idle_replays, 0);
        assert_eq!(s.slot_replays, 0);
        assert!(!s.should_replay(1500.0, IDLE_REPLAY_THRESHOLD_MS));
    }

    #[test]
    fn should_replay_at_threshold_boundary() {
        let s = IdleReplay::new(0.0);
        assert!(!s.should_replay(9_999.0, 10_000.0));
        assert!(s.should_replay(10_000.0, 10_000.0));
        assert!(s.should_replay(50_000.0, 10_000.0));
    }

    #[test]
    fn note_input_resets_the_idle_clock() {
        let mut s = IdleReplay::new(0.0);
        s.note_input(8_000.0);
        // 8s on the clock — 9s wall-clock means only 1s of idle.
        assert!(!s.should_replay(9_000.0, 10_000.0));
        // 18s wall-clock means 10s of idle since the input — boundary trips.
        assert!(s.should_replay(18_000.0, 10_000.0));
    }

    #[test]
    fn note_replay_bumps_counter_and_rearms_clock() {
        let mut s = IdleReplay::new(0.0);
        assert!(s.should_replay(15_000.0, 10_000.0));
        s.note_replay(15_000.0);
        assert_eq!(s.idle_replays, 1);
        // Right after a replay, we wait another full threshold.
        assert!(!s.should_replay(20_000.0, 10_000.0));
        assert!(s.should_replay(25_000.0, 10_000.0));
    }

    #[test]
    fn note_slot_tap_bumps_only_the_slot_counter() {
        let mut s = IdleReplay::new(0.0);
        s.note_slot_tap(2_000.0);
        assert_eq!(s.slot_replays, 1);
        assert_eq!(s.idle_replays, 0);
        // A slot tap is real input — it resets the idle clock too, so
        // the kid doesn't get a duplicate idle replay seconds later.
        assert!(!s.should_replay(11_000.0, 10_000.0));
        assert!(s.should_replay(12_000.0, 10_000.0));
    }

    #[test]
    fn multiple_idle_replays_sequenced_by_threshold() {
        let mut s = IdleReplay::new(0.0);
        for k in 1..=3 {
            let t = (k as f64) * 10_000.0;
            assert!(s.should_replay(t, 10_000.0));
            s.note_replay(t);
            assert_eq!(s.idle_replays, k);
        }
    }
}
