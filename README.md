# Betű Tanulás

Magyar betűkirakó kicsiknek — drag-and-drop letter puzzle for Hungarian
early readers. Mobile-first, runs in the browser, no install.

## What it is

A static web app where the player drags shuffled uppercase letters into
the empty slots of a short Hungarian word, hinted by an emoji (e.g. 🐱
for **CICA**). Right drops snap and lock; wrong drops fly back. No ads,
no IAP, no fail state.

Designed for one specific 6-year-old, but free to play.

## Run locally

Prerequisites: Rust (`rust-toolchain.toml` pins the version), the
[Dioxus CLI](https://dioxuslabs.com), and the standalone
[Tailwind CSS CLI](https://tailwindcss.com/blog/standalone-cli) on
PATH as `tailwindcss`.

```sh
cargo install dioxus-cli      # if not present
make tw                       # compile assets/tailwind.css
make serve                    # dx serve --platform web
```

Or run the production bundle:

```sh
make bundle                   # writes dist/public/
```

## Tests

```sh
make test                     # cargo test --all-targets
```

Word-list validation (`tests/words_validation.rs`) checks every entry
in `assets/words.json` against the v1 hard constraints: uppercase A–Z
only, no accented characters, no Hungarian digraph sequences, tier
N → N+2 letters, every entry has an emoji.

End-to-end Playwright tests live in a separate package (added later in
the project plan) and run on a mobile viewport against the built
`dist/`.

## Repo layout

```
.
├── Cargo.toml
├── Dioxus.toml
├── Makefile
├── rust-toolchain.toml
├── DESIGN.md             # locked v1 design — do not silently revisit
├── assets/
│   ├── tailwind.input.css
│   ├── tailwind.css      # generated; committed so cargo build works
│   ├── words.json        # 41 Hungarian words across 3 tiers
│   └── audio/            # phonemes + chimes (placeholder until recorded)
├── public/               # static files copied verbatim into dist/
├── src/
│   └── main.rs           # entry point
├── tests/
│   └── words_validation.rs
└── .github/workflows/
    └── ci.yml            # test + bundle + deploy
```

## Deploy

Cloudflare Pages, project name `betu-tanulas`. CI deploys on push to
`main`. Credentials are minted via the `aedm/cloudflare-deploy` vault
(`pages-basic` preset). The `*.pages.dev` URL is recorded at the top of
`DESIGN.md` once the first deploy lands.

## License

TBD.
