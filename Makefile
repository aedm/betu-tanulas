.PHONY: help audio tw serve build bundle test fmt clippy clean

TAILWIND ?= tailwindcss

help:
	@echo "Targets:"
	@echo "  audio    - Regenerate assets/audio/ (silent stubs + synthesized SFX)"
	@echo "  tw       - Compile Tailwind CSS (assets/tailwind.input.css -> assets/tailwind.css)"
	@echo "  serve    - dx serve (local dev server)"
	@echo "  bundle   - Build production bundle to dist/ (includes audio copy)"
	@echo "  test     - cargo test"
	@echo "  fmt      - cargo fmt"
	@echo "  clippy   - cargo clippy --all-targets -- -D warnings"
	@echo "  clean    - Remove build artifacts"

audio:
	python3 tools/gen_audio.py

tw:
	$(TAILWIND) -i assets/tailwind.input.css -o assets/tailwind.css --minify

serve: tw
	dx serve --platform web

bundle: tw
	dx bundle --platform web --release
	mkdir -p dist/public/audio
	cp -R assets/audio/letter assets/audio/word assets/audio/sfx dist/public/audio/

test: tw
	cargo test --all-targets

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean
	rm -rf dist
