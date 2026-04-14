# teamder-sys — Rust Backend

## Overview
REST API backend for the Teamder platform. Built with Rocket 0.5, MongoDB 3, and JWT authentication.

## Workspace layout
```
teamder-sys/
├── Cargo.toml          # Workspace root — shared dependency versions here
├── .env.example        # Environment variable template
└── crates/
    ├── teamder-core/   # Pure domain models & error types (no I/O)
    ├── teamder-db/     # MongoDB repository layer
    └── teamder-api/    # Rocket web server (binary crate)
```

## Running locally
```bash
cp .env.example .env      # edit values as needed
cargo run -p teamder-api
# → http://localhost:8000
```

## Environment variables
| Variable      | Default                              | Notes                        |
|---------------|--------------------------------------|------------------------------|
| `MONGODB_URI` | `mongodb://localhost:27017`          | Full MongoDB connection URI  |
| `DB_NAME`     | `teamder`                            | Database name                |
| `JWT_SECRET`  | `teamder-dev-secret-…`               | Change in production         |
| `RUST_LOG`    | `info`                               | Tracing log level            |

## API base path: `/api/v1`

### Auth (no auth required)
| Method | Path | Description |
|--------|------|-------------|
| POST | `/auth/register` | Register a new user, returns `{token, user}` |
| POST | `/auth/login` | Login, returns `{token, user}` |

### Users
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/users` | — | List users; `?q=` for search |
| GET | `/users/me` | Bearer | Current user profile |
| GET | `/users/:id` | — | Get user by ID |
| PATCH | `/users/:id` | Bearer | Update own profile (or admin) |
| DELETE | `/users/:id` | Bearer | Delete own account (or admin) |

### Projects
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/projects` | — | List; `?status=`, `?q=` filters |
| GET | `/projects/my` | Bearer | Projects led by current user |
| GET | `/projects/:id` | — | Get project |
| POST | `/projects` | Bearer | Create project |
| PATCH | `/projects/:id` | Bearer | Update (owner or admin) |
| DELETE | `/projects/:id` | Bearer | Delete (owner or admin) |

### Competitions
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/competitions` | — | List all |
| GET | `/competitions/featured` | — | Featured only |
| GET | `/competitions/:id` | — | Get competition |
| POST | `/competitions` | Admin | Create competition |
| POST | `/competitions/:id/register` | Bearer | Register for competition |

### Study Groups
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/study-groups` | — | List; `?open=true` for open only |
| GET | `/study-groups/:id` | — | Get group |
| POST | `/study-groups` | Bearer | Create group |
| POST | `/study-groups/:id/join` | Bearer | Join group |
| POST | `/study-groups/:id/checkin` | Bearer | Daily check-in |

### Invites
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/invites` | Bearer | Invites sent/received by current user |
| GET | `/invites/:id` | Bearer | Get single invite |
| POST | `/invites` | Bearer | Send invite |
| POST | `/invites/:id/respond` | Bearer | Accept or decline `{accept: bool}` |

### Admin
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/admin/stats` | Admin | Platform-wide counts |
| GET | `/admin/users` | Admin | Full user list (limit 200) |
| GET | `/admin/projects` | Admin | Full project list (limit 200) |

### Health
| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Returns `{status: "ok", service, version}` |

## Authentication
All protected routes require:
```
Authorization: Bearer <jwt_token>
```
JWT tokens are HS256-signed, valid for 30 days. Admin routes additionally require `is_admin: true` in the token claims.

## Crate responsibilities

### `teamder-core`
- `error.rs` — `TeamderError` enum (NotFound, Unauthorized, Forbidden, Validation, Database, Conflict, Internal)
- `models/` — Plain Rust structs with Serde derives; no I/O. Each model has a `new()` constructor and a `*Response` DTO (excludes sensitive fields like `password_hash`).

### `teamder-db`
- `client.rs` — `DbClient::connect(uri, db_name)` creates the MongoDB connection
- `repos/` — One repository struct per collection. All methods are `async` and return `Result<_, TeamderError>`.

### `teamder-api`
- `main.rs` — `#[launch]` sets up CORS, connects DB, mounts all route groups
- `state.rs` — `AppState` (all repos + jwt_secret) injected via `rocket::State`
- `error.rs` — `ApiError` wraps `TeamderError` and implements `Responder` → JSON error body
- `auth.rs` — `create_token` / `verify_token`
- `guards.rs` — `AuthUser` and `AdminUser` request guards

## Adding a new feature
1. Add the model struct in `teamder-core/src/models/`
2. Add `pub mod <model>;` to `teamder-core/src/models/mod.rs`
3. Write a repo in `teamder-db/src/repos/<model>_repo.rs`
4. Export it from `teamder-db/src/repos/mod.rs`
5. Add the repo to `AppState` in `teamder-api/src/state.rs`
6. Write routes in `teamder-api/src/routes/<model>.rs`
7. Declare the module in `routes/mod.rs` and mount in `main.rs`
