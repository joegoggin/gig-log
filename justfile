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
docs:
    cargo run -p gig-log-dev-tools -- docs

# Start all services in TUI mode
dev: db-up
    cargo build -p gig-log-dev-tools
    cargo run -p gig-log-dev-tools -- dev
