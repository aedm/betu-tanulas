.PHONY: help audio tw serve build bundle test fmt clippy clean e2e e2e-install

TAILWIND ?= tailwindcss

help:
	@echo "Targets:"
	@echo "  audio       - Regenerate assets/audio/ (silent stubs + synthesized SFX)"
	@echo "  tw          - Compile Tailwind CSS (assets/tailwind.input.css -> assets/tailwind.css)"
	@echo "  serve       - dx serve (local dev server)"
	@echo "  bundle      - Build production bundle to dist/ (includes audio copy)"
	@echo "  test        - cargo test"
	@echo "  fmt         - cargo fmt"
	@echo "  clippy      - cargo clippy --all-targets -- -D warnings"
	@echo "  e2e-install - npm ci + playwright install in e2e/"
	@echo "  e2e         - Run Playwright e2e against dist/public/ (requires bundle)"
	@echo "  clean       - Remove build artifacts"

audio:
	python3 tools/gen_audio.py

tw:
	$(TAILWIND) -i assets/tailwind.input.css -o assets/tailwind.css --minify

serve: tw
	dx serve --platform web

bundle: tw
	dx bundle --platform web --release
	mkdir -p dist/public/audio dist/public/icons
	cp -R assets/audio/letter assets/audio/word assets/audio/sfx dist/public/audio/
	cp assets/icons/*.png dist/public/icons/
	cp assets/manifest.webmanifest dist/public/manifest.webmanifest

test: tw
	cargo test --all-targets

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings

e2e-install:
	cd e2e && npm install
	cd e2e && npx playwright install --with-deps webkit chromium

e2e:
	cd e2e && npx playwright test

clean:
	cargo clean
	rm -rf dist
	rm -rf e2e/node_modules e2e/test-results e2e/playwright-report e2e/blob-report
