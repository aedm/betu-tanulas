//! Sanity-check the source `index.html` template at the crate root: it
//! is what `dx bundle` copies into `dist/public/index.html` (with
//! placeholder substitution). The defaults dx ships do *not* set
//! `viewport-fit=cover` or any iOS-PWA hints, so we override and want
//! a compile-time guarantee that the override stays in place.

const TEMPLATE: &str = include_str!("../index.html");

#[test]
fn viewport_meta_enables_safe_area_cover() {
    assert!(
        TEMPLATE.contains("viewport-fit=cover"),
        "viewport meta must include viewport-fit=cover so env(safe-area-inset-*) \
         resolves to non-zero on notched phones; got:\n{TEMPLATE}"
    );
    assert!(
        TEMPLATE.contains("width=device-width"),
        "viewport meta must still pin width to device-width; got:\n{TEMPLATE}"
    );
}

#[test]
fn html_lang_is_hungarian() {
    assert!(
        TEMPLATE.contains(r#"<html lang="hu">"#),
        "<html lang=\"hu\"> required for screen-reader pronunciation; got:\n{TEMPLATE}"
    );
}

#[test]
fn ios_pwa_meta_tags_present() {
    for needle in [
        r#"name="theme-color""#,
        r#"name="apple-mobile-web-app-capable""#,
        r#"name="apple-mobile-web-app-status-bar-style""#,
    ] {
        assert!(
            TEMPLATE.contains(needle),
            "expected meta {needle:?} in index.html; got:\n{TEMPLATE}"
        );
    }
}

#[test]
fn template_keeps_dx_placeholders_intact() {
    // dx bundle replaces these — if they vanish, the title and main
    // mount point silently break.
    assert!(TEMPLATE.contains("{app_title}"), "missing {{app_title}}");
    assert!(TEMPLATE.contains(r#"id="main""#), "missing #main mount");
}

#[test]
fn pwa_manifest_and_icons_linked() {
    // The bundle pipeline copies these to the same paths under
    // dist/public/. If a future scaffold drops the <link>s, "Add to
    // Home Screen" silently regresses to a Safari-rendered fallback
    // icon and the install prompt never offers standalone display.
    for needle in [
        r#"rel="manifest" href="/manifest.webmanifest""#,
        r#"rel="apple-touch-icon" href="/icons/apple-touch-icon.png""#,
        r#"rel="icon" type="image/png" sizes="32x32" href="/icons/favicon-32.png""#,
        r#"rel="icon" type="image/png" sizes="192x192" href="/icons/icon-192.png""#,
    ] {
        assert!(
            TEMPLATE.contains(needle),
            "expected link {needle:?} in index.html; got:\n{TEMPLATE}"
        );
    }
}
