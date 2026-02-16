# api commands
api *args:
	#!/usr/bin/env bash
	set -euo pipefail
	cd api

	# Dev-only docs server for Rustdoc output.
	# Docs URL: http://localhost:7007
	mkdir -p target/doc
	printf '%s\n' \
		'<!doctype html>' \
		'<html lang="en">' \
		'<head>' \
		'  <meta charset="utf-8">' \
		'  <meta http-equiv="refresh" content="0; url=/api/">' \
		'  <title>Redirecting...</title>' \
		'</head>' \
		'<body>' \
		'  <p>Redirecting to <a href="/api/">/api/</a>...</p>' \
		'  <script>location.replace("/api/");</script>' \
		'</body>' \
		'</html>' \
		> target/doc/index.html
	echo "Serving API docs at http://localhost:7007"
	echo "Docs logs: /tmp/gig-log-docs-watch.log, /tmp/gig-log-docs-server.log"

	cargo-watch -c -x "run -- {{args}}" &
	api_watch_pid=$!

	cargo-watch -c -s 'cargo doc --no-deps --document-private-items && printf '"'"'%s\n'"'"' '"'"'<!doctype html>'"'"' '"'"'<html lang="en">'"'"' '"'"'<head>'"'"' '"'"'  <meta charset="utf-8">'"'"' '"'"'  <meta http-equiv="refresh" content="0; url=/api/">'"'"' '"'"'  <title>Redirecting...</title>'"'"' '"'"'</head>'"'"' '"'"'<body>'"'"' '"'"'  <p>Redirecting to <a href="/api/">/api/</a>...</p>'"'"' '"'"'  <script>location.replace("/api/");</script>'"'"' '"'"'</body>'"'"' '"'"'</html>'"'"' > target/doc/index.html' >/tmp/gig-log-docs-watch.log 2>&1 &
	docs_watch_pid=$!

	python3 -m http.server 7007 --directory target/doc >/tmp/gig-log-docs-server.log 2>&1 &
	docs_server_pid=$!

	cleanup() {
		kill "$api_watch_pid" "$docs_watch_pid" "$docs_server_pid" 2>/dev/null || true
	}

	trap cleanup INT TERM EXIT
	wait "$api_watch_pid" "$docs_watch_pid" "$docs_server_pid"

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

web-tailscale:
	cd web && pnpm dev:tailscale

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

# database commands
db-migrate *args:
	cd api && sqlx migrate run

# posting 
posting *args:
	posting --collection ./api/.posting
