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

db-add *args: _require-sqlx-cli
    cd api && sqlx migrate add -r {{args}}

db-migrate: _require-sqlx-cli
	cd api && sqlx migrate run 

db-revert: _require-sqlx-cli
	cd api && sqlx migrate revert

db-info: _require-sqlx-cli
	cd api && sqlx migrate info

db-test-setup: _require-sqlx-cli
    . ./.env.test && test -n "$TEST_DATABASE_URL" && cd api && sqlx database setup -D "$TEST_DATABASE_URL"

# Testing
test:
    cargo test --workspace

test-unit:
    cargo test --workspace --lib

# API
api: _require-cargo-watch
    cargo watch -x 'run -p gig-log-api'

api-build:
    cargo build -p gig-log-api

api-release:
	cargo build --release -p gig-log-api

api-add *args:
    cd api && cargo add {{args}}

api-remove *args:
    cd api && cargo remove {{args}}

# Web
web:
    cd web && SASS_PATH=styles trunk serve

web-build:
    cd web && SASS_PATH=styles trunk build

web-release:
	cd web && SASS_PATH=styles trunk build --release

web-add *args:
    cd web && cargo add {{args}}

web-remove *args:
    cd web && cargo remove {{args}}

# Common
common-build:
	cargo build -p gig-log-common

common-release:
	cargo build --release -p gig-log-common

common-add *args:
	cd common && cargo add {{args}}

# Development
dev-tools *args:
    cargo run -p gig-log-dev-tools -- {{args}}

dev-tools-build:
	cargo build -p gig-log-dev-tools

dev-tools-add *args:
	cd dev-tools && cargo add {{args}}

dev-tools-remove *args:
	cd dev-tools && cargo remove {{args}}

docs:
    cargo run -p gig-log-dev-tools -- docs

dev: db-up
    cargo build -p gig-log-dev-tools
    cargo run -p gig-log-dev-tools -- dev

setup *args:
    cargo run -p gig-log-dev-tools -- setup {{args}}

db-viewer:
	cargo run -p gig-log-dev-tools -- db-viewer
