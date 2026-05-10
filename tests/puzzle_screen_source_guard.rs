// Source-shape regression guard for `src/puzzle_screen.rs`.
//
// Two bugs in this file came from the same Dioxus 0.7 delegation
// gotcha: `evt.current_target()` returns the *mount root*, not the
// element the listener was attached to in the rsx. The first instance
// (in `pickup_origin_center`) was fixed by PR #14; the second (in
// `capture_pointer_for_event`) was fixed by the patch this test ships
// with.
//
// The proper behavior — capture the pointer on the tile, not on
// whatever `current_target` resolves to — only manifests under real
// touch input; synthetic Playwright `dispatchEvent` doesn't establish
// browser-level pointer-capture state, so an e2e assertion on
// `hasPointerCapture` fails uniformly regardless of fix status. This
// test is therefore a *shape* check on the source file: it asserts the
// fixed pattern is in place and the buggy one is not.
//
// If a future refactor genuinely re-introduces `current_target()` for
// a non-bubbling event (legitimate use), update this guard to scope
// the assertion more narrowly.

const SRC: &str = include_str!("../src/puzzle_screen.rs");

fn extract_fn(src: &str, name: &str) -> String {
    let needle = format!("fn {name}(");
    let start = src.find(&needle).expect("function not found");
    let after = &src[start..];
    let mut depth = 0i32;
    let mut started = false;
    let mut end = 0usize;
    for (i, ch) in after.char_indices() {
        match ch {
            '{' => {
                depth += 1;
                started = true;
            }
            '}' => {
                depth -= 1;
                if started && depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    after[..end].to_string()
}

#[test]
fn capture_pointer_for_event_does_not_use_current_target() {
    // Walk up from `evt.target()` to `.betu-tile` instead. With
    // `current_target` you set capture on the dioxus mount root; real
    // browsers then re-target every subsequent pointer event to the
    // root, bypassing `.betu-screen`'s `onpointermove` handler — drag
    // freezes after the first render. See provenance comment above.
    let body = extract_fn(SRC, "capture_pointer_for_event");
    assert!(
        !body.contains("current_target()"),
        "capture_pointer_for_event must use target() + closest(\".betu-tile\"), \
         not current_target() (which resolves to the mount root under \
         Dioxus 0.7's bubbling-event delegation). Body:\n{body}"
    );
    assert!(
        body.contains(".closest(\".betu-tile\")"),
        "capture_pointer_for_event must walk up to the .betu-tile via \
         closest() so capture lives on the tile. Body:\n{body}"
    );
    assert!(
        body.contains("set_pointer_capture(pointer_id)"),
        "capture_pointer_for_event must still call set_pointer_capture. \
         Body:\n{body}"
    );
}

#[test]
fn pickup_origin_center_does_not_use_current_target() {
    // Same gotcha, fixed in PR #14. Guard kept here so a future
    // "let me clean up these duplicate target() walks" refactor can't
    // silently revive it.
    let body = extract_fn(SRC, "pickup_origin_center");
    assert!(
        !body.contains("current_target()"),
        "pickup_origin_center must use target() + closest(\".betu-tile\") to \
         read the tile's bounding rect, not current_target() (mount root). \
         Body:\n{body}"
    );
    assert!(
        body.contains(".closest(\".betu-tile\")"),
        "pickup_origin_center must walk up to .betu-tile. Body:\n{body}"
    );
}
