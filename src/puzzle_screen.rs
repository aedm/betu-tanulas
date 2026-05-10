use dioxus::prelude::*;

use crate::audio;
use crate::game::Game;
use crate::idle::IdleReplay;
use crate::progress;
use crate::puzzle::{DropOutcome, TileState};
use crate::t;

const SNAP_RADIUS_PX: f64 = 40.0;

#[component]
pub fn PuzzleScreen(game: Signal<Game>) -> Element {
    let mut idle = use_signal(|| IdleReplay::new(now_ms()));
    install_idle_replay_timer(game, idle);

    let g = game.read();
    let p = &g.current_puzzle;
    let dragging_idx = p.dragging_tile();
    let won = g.is_won();
    let word = p.word.word.clone();
    let emoji = p.word.emoji.clone();
    let wrong_drops = p.wrong_drops;
    let current_tier = g.current_tier;
    let total_in_tier = g.words.iter().filter(|w| w.tier == current_tier).count();
    let done_in_tier = g
        .words
        .iter()
        .filter(|w| w.tier == current_tier && g.is_completed(&w.word))
        .count();
    let idle_snapshot = *idle.read();

    rsx! {
        section {
            class: "betu-screen",
            "data-word": "{word}",
            "data-dragging": if dragging_idx.is_some() { "true" } else { "false" },
            "data-won": if won { "true" } else { "false" },
            "data-wrong-drops": "{wrong_drops}",
            "data-idle-replays": "{idle_snapshot.idle_replays}",
            "data-slot-replays": "{idle_snapshot.slot_replays}",
            onpointermove: move |evt| {
                if won {
                    return;
                }
                if game.read().current_puzzle.dragging_tile().is_none() {
                    return;
                }
                let coords = evt.client_coordinates();
                game.write()
                    .current_puzzle
                    .pointer_move(evt.pointer_id(), (coords.x, coords.y));
                idle.write().note_input(now_ms());
            },
            onpointerup: move |evt| {
                if won {
                    return;
                }
                if game.read().current_puzzle.dragging_tile().is_none() {
                    return;
                }
                let coords = evt.client_coordinates();
                let pid = evt.pointer_id();
                {
                    let mut w = game.write();
                    w.current_puzzle.pointer_move(pid, (coords.x, coords.y));
                }
                let centers = measure_slot_centers();
                let (outcome, volume, completed_word) = {
                    let mut w = game.write();
                    let outcome = w.current_puzzle.release(pid, &centers, SNAP_RADIUS_PX);
                    let completed = if w.is_won() {
                        Some(w.current_word().word.clone())
                    } else {
                        None
                    };
                    (outcome, w.progress.volume, completed)
                };
                if matches!(outcome, DropOutcome::Snapped { .. }) {
                    audio::play_snap(volume);
                }
                if let Some(word) = completed_word {
                    audio::play_chime(volume);
                    audio::play_word(&word, volume);
                }
                idle.write().note_input(now_ms());
            },
            onpointercancel: move |evt| {
                if game.read().current_puzzle.dragging_tile().is_none() {
                    return;
                }
                game.write().current_puzzle.cancel(evt.pointer_id());
                idle.write().note_input(now_ms());
            },
            div {
                class: "betu-puzzle-header",
                button {
                    class: "betu-home",
                    r#type: "button",
                    aria_label: t!("puzzle.home"),
                    "data-testid": "puzzle-home",
                    onclick: move |_| {
                        idle.write().note_input(now_ms());
                        game.write().go_to_menu();
                    },
                    "🏠"
                }
                span {
                    class: "betu-puzzle-progress",
                    "data-testid": "puzzle-progress",
                    aria_label: t!("puzzle.progress"),
                    "{current_tier} · {done_in_tier}/{total_in_tier}"
                }
            }
            div {
                class: "betu-emoji",
                role: "img",
                aria_label: "{word}",
                "{emoji}"
            }
            div {
                class: "betu-row betu-slots",
                aria_label: "slots",
                for (idx, slot) in p.slots.iter().enumerate() {
                    {
                        let target_for_drag = match dragging_idx {
                            Some(i) => slot.is_none() && p.is_correct_target(i, idx),
                            None => false,
                        };
                        let letter = slot
                            .and_then(|tile_idx| p.tiles.get(tile_idx).map(|t| t.letter));
                        rsx! {
                            div {
                                key: "slot-{idx}",
                                class: "betu-cell betu-slot",
                                "data-slot-index": "{idx}",
                                "data-filled": if slot.is_some() { "true" } else { "false" },
                                "data-target": if target_for_drag { "true" } else { "false" },
                                onclick: move |_| {
                                    let now = now_ms();
                                    let (volume, word_to_say) = {
                                        let g = game.read();
                                        if g.is_won()
                                            || g.current_puzzle.dragging_tile().is_some()
                                        {
                                            (None, None)
                                        } else {
                                            (
                                                Some(g.progress.volume),
                                                Some(g.current_word().word.clone()),
                                            )
                                        }
                                    };
                                    if let (Some(v), Some(w)) = (volume, word_to_say) {
                                        idle.write().note_slot_tap(now);
                                        audio::play_word(&w, v);
                                    }
                                },
                                {letter.map(|c| c.to_string()).unwrap_or_default()}
                            }
                        }
                    }
                }
            }
            div {
                class: "betu-row betu-tiles",
                aria_label: "tiles",
                for (idx, tile) in p.tiles.iter().enumerate() {
                    {
                        let placed = matches!(tile.state, TileState::Placed { .. });
                        let dragging = matches!(tile.state, TileState::Dragging { .. });
                        let style = match tile.state {
                            TileState::Dragging {
                                pointer,
                                origin_center,
                                ..
                            } => {
                                let dx = pointer.0 - origin_center.0;
                                let dy = pointer.1 - origin_center.1;
                                format!(
                                    "transform: translate({dx}px, {dy}px); z-index: 50; \
                                     transition: none; touch-action: none;"
                                )
                            }
                            _ => "touch-action: none;".to_string(),
                        };
                        rsx! {
                            div {
                                key: "tile-{idx}",
                                class: "betu-cell betu-tile",
                                "data-tile-index": "{idx}",
                                "data-placed": if placed { "true" } else { "false" },
                                "data-dragging": if dragging { "true" } else { "false" },
                                style: "{style}",
                                onpointerdown: move |evt| {
                                    idle.write().note_input(now_ms());
                                    if won {
                                        return;
                                    }
                                    if matches!(
                                        game
                                            .read()
                                            .current_puzzle
                                            .tiles
                                            .get(idx)
                                            .map(|t| t.state),
                                        Some(TileState::Placed { .. })
                                    ) {
                                        return;
                                    }
                                    let coords = evt.client_coordinates();
                                    let pointer = (coords.x, coords.y);
                                    let origin_center =
                                        pickup_origin_center(&evt).unwrap_or(pointer);
                                    let pid = evt.pointer_id();
                                    let (picked, volume, letter) = {
                                        let mut w = game.write();
                                        let letter = w
                                            .current_puzzle
                                            .tiles
                                            .get(idx)
                                            .map(|t| t.letter);
                                        let picked = w
                                            .current_puzzle
                                            .pickup(idx, pid, pointer, origin_center);
                                        (picked, w.progress.volume, letter)
                                    };
                                    if picked {
                                        capture_pointer_for_event(&evt, pid);
                                        if let Some(c) = letter {
                                            audio::play_letter(c, volume);
                                        }
                                    }
                                },
                                "{tile.letter}"
                            }
                        }
                    }
                }
            }
            if won {
                WinOverlay { emoji: emoji.clone(), game, idle }
            }
        }
    }
}

#[component]
fn WinOverlay(emoji: String, game: Signal<Game>, idle: Signal<IdleReplay>) -> Element {
    let mut idle = idle;
    rsx! {
        div {
            class: "betu-emoji-rain",
            aria_hidden: "true",
            for i in 0..10 {
                span {
                    key: "rain-{i}",
                    class: "betu-rain-drop",
                    style: "--i: {i};",
                    "{emoji}"
                }
            }
        }
        button {
            class: "betu-next",
            r#type: "button",
            aria_label: t!("puzzle.next"),
            "data-testid": "betu-next",
            onclick: move |_| {
                idle.write().note_input(now_ms());
                {
                    let mut g = game.write();
                    g.advance_to_next();
                    progress::save(&g.progress);
                }
            },
            "➡️"
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn now_ms() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

#[cfg(target_arch = "wasm32")]
fn install_idle_replay_timer(game: Signal<Game>, mut idle: Signal<IdleReplay>) {
    use crate::idle::IDLE_REPLAY_THRESHOLD_MS;
    use dioxus::core::use_hook_with_cleanup;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    use_hook_with_cleanup(
        move || {
            let cb = Closure::wrap(Box::new(move || {
                let now = now_ms();
                let snapshot = *idle.peek();
                if !snapshot.should_replay(now, IDLE_REPLAY_THRESHOLD_MS) {
                    return;
                }
                let g = game.peek();
                if g.is_won() || g.current_puzzle.dragging_tile().is_some() {
                    return;
                }
                let word = g.current_word().word.clone();
                let volume = g.progress.volume;
                drop(g);
                audio::play_word(&word, volume);
                idle.write().note_replay(now);
            }) as Box<dyn FnMut()>);

            let window = web_sys::window().expect("window must exist on wasm");
            let handle = window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    1_000,
                )
                .expect("set_interval must succeed");
            (handle, cb.into_js_value())
        },
        |(handle, _cb): (i32, wasm_bindgen::JsValue)| {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(handle);
            }
        },
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn install_idle_replay_timer(_game: Signal<Game>, _idle: Signal<IdleReplay>) {}

#[cfg(target_arch = "wasm32")]
fn pickup_origin_center(evt: &Event<PointerData>) -> Option<(f64, f64)> {
    use dioxus::web::WebEventExt;
    use wasm_bindgen::JsCast;
    let we = evt.try_as_web_event()?;
    let target = we.current_target()?;
    let el = target.dyn_into::<web_sys::Element>().ok()?;
    let rect = el.get_bounding_client_rect();
    Some((
        rect.left() + rect.width() / 2.0,
        rect.top() + rect.height() / 2.0,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn pickup_origin_center(_evt: &Event<PointerData>) -> Option<(f64, f64)> {
    None
}

#[cfg(target_arch = "wasm32")]
fn capture_pointer_for_event(evt: &Event<PointerData>, pointer_id: i32) {
    use dioxus::web::WebEventExt;
    use wasm_bindgen::JsCast;
    let Some(we) = evt.try_as_web_event() else {
        return;
    };
    let Some(target) = we.current_target() else {
        return;
    };
    let Ok(el) = target.dyn_into::<web_sys::Element>() else {
        return;
    };
    let _ = el.set_pointer_capture(pointer_id);
}

#[cfg(not(target_arch = "wasm32"))]
fn capture_pointer_for_event(_evt: &Event<PointerData>, _pointer_id: i32) {}

#[cfg(target_arch = "wasm32")]
fn measure_slot_centers() -> Vec<(f64, f64)> {
    use wasm_bindgen::JsCast;
    let Some(window) = web_sys::window() else {
        return Vec::new();
    };
    let Some(document) = window.document() else {
        return Vec::new();
    };
    let Ok(list) = document.query_selector_all(".betu-slot") else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(list.length() as usize);
    for i in 0..list.length() {
        let Some(node) = list.item(i) else { continue };
        let Ok(el) = node.dyn_into::<web_sys::Element>() else {
            continue;
        };
        let attr = el.get_attribute("data-slot-index");
        let rect = el.get_bounding_client_rect();
        let cx = rect.left() + rect.width() / 2.0;
        let cy = rect.top() + rect.height() / 2.0;
        let idx = attr
            .as_deref()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(out.len());
        if out.len() <= idx {
            out.resize(idx + 1, (0.0, 0.0));
        }
        out[idx] = (cx, cy);
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn measure_slot_centers() -> Vec<(f64, f64)> {
    Vec::new()
}
