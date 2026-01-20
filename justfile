# api commands
api *args:
	cd api && cargo-watch -c -x "run -- {{args}}"

api-add *args:
	cd api && cargo add {{args}}

api-remove *args:
	cd api && cargo remove {{args}}

api-clean:
	cd api && cargo clean -p milo-api

api-build *args:
	cd api && cargo build {{args}}

api-release:
	cd api && cargo run --release

# web commands
web *args:
	cd web && pnpm dev {{args}}

web-add *args:
	cd web && pnpm add {{args}} 

web-remove *args:
	cd web && pnpm remove {{args}}

web-build *args:
	cd web && pnpm build {{args}}

web-preview *args:
	cd web && pnpm preview {{args}}

web-test *args:
	cd web && pnpm test {{args}}

web-lint *args:
	cd web && pnpm lint {{args}}

web-format *args:
	cd web && pnpm format {{args}}

web-check *args:
	cd web && pnpm check {{args}}

web-storybook *args:
	cd web && pnpm storybook
