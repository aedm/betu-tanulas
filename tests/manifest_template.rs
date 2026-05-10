//! Sanity-check the source `assets/manifest.webmanifest`. The bundle
//! pipeline ships this file verbatim to `dist/public/manifest.webmanifest`,
//! and the home-screen install path has no fallback if the JSON is
//! malformed or a required key (name, icons, start_url) is missing.

use serde_json::Value;

const MANIFEST: &str = include_str!("../assets/manifest.webmanifest");

fn parsed() -> Value {
    serde_json::from_str(MANIFEST).expect("manifest.webmanifest must be valid JSON")
}

#[test]
fn manifest_is_valid_json() {
    let _ = parsed();
}

#[test]
fn manifest_has_required_install_fields() {
    let v = parsed();
    let obj = v.as_object().expect("top-level must be an object");
    for key in ["name", "short_name", "start_url", "display", "icons"] {
        assert!(
            obj.contains_key(key),
            "manifest missing required key {key:?}"
        );
    }
    assert_eq!(
        obj["display"], "standalone",
        "display must be 'standalone' so iOS Add-to-Home-Screen launches full-screen"
    );
    assert_eq!(obj["start_url"], "/", "start_url must be the app root");
}

#[test]
fn manifest_uses_hungarian_locale_and_bone_palette() {
    let v = parsed();
    assert_eq!(
        v["lang"], "hu",
        "lang must be 'hu' for Hungarian voice pack"
    );
    assert_eq!(
        v["theme_color"], "#FBFAF7",
        "theme_color must match --color-bone in tailwind.input.css"
    );
    assert_eq!(
        v["background_color"], "#FBFAF7",
        "background_color must match --color-bone for a seamless launch splash"
    );
}

#[test]
fn manifest_declares_192_and_512_png_icons() {
    let v = parsed();
    let icons = v["icons"].as_array().expect("icons must be an array");
    let mut sizes: Vec<&str> = icons
        .iter()
        .map(|i| i["sizes"].as_str().expect("each icon needs a sizes string"))
        .collect();
    sizes.sort();
    assert_eq!(
        sizes,
        vec!["192x192", "512x512"],
        "manifest must declare both the 192 and 512 PNGs (Chrome install prompt + iOS splash)"
    );
    for icon in icons {
        let src = icon["src"].as_str().unwrap();
        assert!(
            src.starts_with("/icons/"),
            "icon src must be an absolute path under /icons/, got {src}"
        );
        assert_eq!(
            icon["type"], "image/png",
            "icons must be declared as image/png"
        );
    }
}

#[test]
fn manifest_icon_files_exist_in_assets() {
    // Tests run from the crate root via `cargo test`. The icon paths
    // referenced by the manifest are root-absolute URLs at runtime; on
    // disk they live under assets/icons/.
    for relative in [
        "icon-192.png",
        "icon-512.png",
        "apple-touch-icon.png",
        "favicon-32.png",
    ] {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/icons")
            .join(relative);
        assert!(
            path.exists(),
            "expected icon file at {} (regenerate with `python3 tools/gen_icons.py`)",
            path.display()
        );
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
            "{} is not a valid PNG (bad magic)",
            path.display()
        );
    }
}
