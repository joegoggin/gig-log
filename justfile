# Check that cargo-watch is installed
[private]
_require-cargo-watch:
    @command -v cargo-watch > /dev/null 2>&1 || { echo "cargo-watch is not installed. Install it with: cargo install cargo-watch"; exit 1; }

# Check that sqlx-cli is installed
[private]
_require-sqlx-cli:
    @command -v sqlx > /dev/null 2>&1 || { echo "sqlx-cli is not installed. Install it with: cargo install sqlx-cli"; exit 1; }

# Check that miniserve is installed
[private]
_require-miniserve:
    @command -v miniserve > /dev/null 2>&1 || { echo "miniserve is not installed. Install it with: cargo install miniserve"; exit 1; }

# Start PostgreSQL via Docker Compose
db-up:
    docker compose up -d

# Stop PostgreSQL
db-down:
    docker compose down

# Run SQLx migrations
db-migrate: _require-sqlx-cli
    sqlx migrate run

# Start API with auto-reload
api: _require-cargo-watch
    cargo watch -x 'run -p gig-log-api'

# Start frontend with trunk serve
web:
    cd web && trunk serve

# Build API release
api-build:
    cargo build --release -p gig-log-api

# Build web release
web-build:
    cd web && trunk build --release

# Serve workspace docs on :7007 with watch
docs: _require-cargo-watch _require-miniserve
    #!/usr/bin/env bash
    # Kill any leftover doc server on port 7007
    lsof -ti :7007 | xargs -r kill 2>/dev/null || true
    mkdir -p target/doc
    cargo doc --workspace --document-private-items --color always || true
    bash scripts/generate-doc-index.sh
    miniserve --index index.html -p 7007 target/doc &
    SERVER_PID=$!
    trap "kill $SERVER_PID 2>/dev/null" EXIT
    echo "Docs available at http://localhost:7007"
    cargo watch -s 'cargo doc --workspace --document-private-items --color always && bash scripts/generate-doc-index.sh'

# Start all services
dev: db-up
    (script -qfec "just api" /dev/null 2>&1 | sed 's/^/\x1b[34m[API]\x1b[0m  | /') & \
    (script -qfec "just web" /dev/null 2>&1 | sed 's/^/\x1b[32m[WEB]\x1b[0m  | /') & \
    (script -qfec "just docs" /dev/null 2>&1 | sed 's/^/\x1b[33m[DOCS]\x1b[0m | /') & \
    wait 
