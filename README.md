# GigLog

GigLog is a full-stack web application for freelancers and gig workers to manage their work across multiple clients. Users can track companies, jobs, work sessions, and payments — all in one place.

## Getting Started

### Requirements

- [Node.js](https://nodejs.org)
- [pnpm](https://pnpm.io)
- [Rust](https://www.rust-lang.org)
- [Docker](https://www.docker.com)
- [Python 3](https://www.python.org/downloads/)
- [just](https://github.com/casey/just)
- [cargo-watch](https://github.com/watchexec/cargo-watch)
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)

### Setup

1. Clone the repo

   ```sh
   git clone https://github.com/joegoggin/gig-log.git
   cd gig-log
   ```

2. Start PostgreSQL via Docker

   ```sh
   docker compose up -d postgres
   ```

3. Copy `api/.env.example` to `api/.env` and fill in values

   ```sh
   cp api/.env.example api/.env
   ```

   Logging defaults are included in `api/.env.example`:
   - `LOG_LEVEL` - Global log level (`info`, `debug`, etc.)
   - `LOG_HTTP_BODY_ENABLED` - Enables JSON request/response body logging
   - `LOG_HTTP_MAX_BODY_BYTES` - Max body size eligible for body logging

4. Install frontend dependencies

   ```sh
   pnpm --dir web install
   ```

5. Run database migrations (optional)

   ```sh
   just db-migrate
   ```

   The API now checks and applies pending migrations automatically at startup.
   If you want the API to auto-start Docker Compose in development, set
   `DOCKER_PREFLIGHT_ENABLED=true` in `api/.env`.
   Use this command when you want to run migrations manually.

6. Start the API

   ```sh
   just api
   ```

7. Start the frontend

   ```sh
   just web
   ```

### Local URLs

- App: <http://localhost:3000>
- Storybook: <http://localhost:6006>
- API docs: <http://localhost:7007>

## Tech Stack

- **Frontend** — [TanStack Router](https://tanstack.com/router/latest) (React, TypeScript, Vite)
- **Backend** — [Actix Web](https://actix.rs) (Rust)
- **Database** — [PostgreSQL](https://www.postgresql.org) with [SQLx](https://github.com/launchbadge/sqlx)
- **Styling** — [Sass](https://sass-lang.com) (SCSS modules + shared variables/mixins)
- **Component Documentation** — [Storybook](https://storybook.js.org)

## Scripts

All commands are run with [just](https://github.com/casey/just).

### API

#### `just api`

Run the API dev server with hot-reload (via `cargo-watch`) and a local Rustdoc server on port 7007.

```sh
just api
```

#### `just api-add <package>`

Add a Rust dependency to the API.

```sh
just api-add serde
```

#### `just api-remove <package>`

Remove a Rust dependency from the API.

```sh
just api-remove serde
```

#### `just api-clean`

Clean API build artifacts.

```sh
just api-clean
```

#### `just api-build`

Build the API.

```sh
just api-build
```

#### `just api-release`

Build and run the API in release mode.

```sh
just api-release
```

### Frontend

#### `just web`

Run Vite and Storybook in parallel (app on port 3000, Storybook on port 6006).

```sh
just web
```

#### `just web-add <package>`

Add a frontend dependency.

```sh
just web-add axios
```

#### `just web-remove <package>`

Remove a frontend dependency.

```sh
just web-remove axios
```

#### `just web-build`

Build the frontend for production.

```sh
just web-build
```

#### `just web-preview`

Preview the production build.

```sh
just web-preview
```

#### `just web-test`

Run frontend tests.

```sh
just web-test
```

#### `just web-lint`

Lint frontend code.

```sh
just web-lint
```

#### `just web-format`

Format frontend code.

```sh
just web-format
```

#### `just web-check`

Auto-format code and auto-fix lint issues.

```sh
just web-check
```

#### `just web-storybook`

Run Storybook.

```sh
just web-storybook
```

### Database

#### `just db-migrate`

Run database migrations with SQLx.

```sh
just db-migrate
```

### Utilities

#### `just posting`

Open the [Posting](https://github.com/darrenburns/posting) API client with the project's request collection.

```sh
just posting
```
