use dioxus::prelude::*;

use crate::puzzle::{Puzzle, TileState};
use crate::word::Word;

const SNAP_RADIUS_PX: f64 = 40.0;

#[component]
pub fn PuzzleScreen(word: Word) -> Element {
    let initial = Puzzle::new(word.clone(), None);
    let mut puzzle = use_signal(|| initial);

    let p = puzzle.read();
    let dragging_idx = p.dragging_tile();

    rsx! {
        section {
            class: "betu-screen",
            "data-word": "{p.word.word}",
            "data-dragging": if dragging_idx.is_some() { "true" } else { "false" },
            onpointermove: move |evt| {
                if puzzle.read().dragging_tile().is_none() {
                    return;
                }
                let coords = evt.client_coordinates();
                puzzle.write().pointer_move(evt.pointer_id(), (coords.x, coords.y));
            },
            onpointerup: move |evt| {
                if puzzle.read().dragging_tile().is_none() {
                    return;
                }
                let coords = evt.client_coordinates();
                let pid = evt.pointer_id();
                {
                    let mut w = puzzle.write();
                    w.pointer_move(pid, (coords.x, coords.y));
                }
                let centers = measure_slot_centers();
                puzzle.write().release(pid, &centers, SNAP_RADIUS_PX);
            },
            onpointercancel: move |evt| {
                puzzle.write().cancel(evt.pointer_id());
            },
            div {
                class: "betu-emoji",
                role: "img",
                aria_label: "{p.word.word}",
                "{p.word.emoji}"
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
                                    if matches!(
                                        puzzle.read().tiles.get(idx).map(|t| t.state),
                                        Some(TileState::Placed { .. })
                                    ) {
                                        return;
                                    }
                                    let coords = evt.client_coordinates();
                                    let pointer = (coords.x, coords.y);
                                    let origin_center =
                                        pickup_origin_center(&evt).unwrap_or(pointer);
                                    let pid = evt.pointer_id();
                                    let picked = puzzle
                                        .write()
                                        .pickup(idx, pid, pointer, origin_center);
                                    if picked {
                                        capture_pointer_for_event(&evt, pid);
                                    }
                                },
                                "{tile.letter}"
                            }
                        }
                    }
                }
            }
        }
    }
}

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
