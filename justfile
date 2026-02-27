# Installation Checks
[private]
_require-cargo-watch:
    @command -v cargo-watch > /dev/null 2>&1 || { echo "cargo-watch is not installed. Install it with: cargo install cargo-watch"; exit 1; }

[private]
_require-sqlx-cli:
    @command -v sqlx > /dev/null 2>&1 || { echo "sqlx-cli is not installed. Install it with: cargo install sqlx-cli"; exit 1; }

[private]
_require-miniserve:
    @command -v miniserve > /dev/null 2>&1 || { echo "miniserve is not installed. Install it with: cargo install miniserve"; exit 1; }

# Database
db-up:
    docker compose up -d

db-down:
    docker compose down

db-migrate: _require-sqlx-cli
    sqlx migrate run

# API
api: _require-cargo-watch
    cargo watch -x 'run -p gig-log-api'

api-build:
    cargo build --release -p gig-log-api

api-add *args:
	cd api && cargo add {{args}}

api-remove *args:
	cd api && cargo remove {{args}}

# Web
web:
    cd web && trunk serve

web-build:
    cd web && trunk build --release

web-add *args:
	cd web && cargo add {{args}}

api-remove *arg:
	cd web && cd remove {{args}}

# Development
dev-tools *args:
	cargo run -p gig-log-dev-tools -- {{args}}

docs:
    cargo run -p gig-log-dev-tools -- docs

dev: db-up
    cargo build -p gig-log-dev-tools
    cargo run -p gig-log-dev-tools -- dev

setup *args:
    cargo run -p gig-log-dev-tools -- setup {{args}}
