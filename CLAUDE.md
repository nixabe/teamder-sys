# teamder-sys — Rust Backend

## What this project is

REST API backend for **Teamder**, a team-matching platform for university students. Users discover collaborators, post projects, join competitions, form study groups, exchange peer reviews, and chat. The API also handles file uploads (avatars, portfolios, resumes) and admin analytics.

The companion frontend lives at `C:\coding\Teamder` (Next.js). See that repo's `CLAUDE.md`.

---

## Stack

| Layer | Choice |
|-------|--------|
| Language | Rust 2021 |
| Web framework | Rocket 0.5 |
| Database | MongoDB 3 (async, via `mongodb` crate) |
| Auth | JWT (`jsonwebtoken` 9, HS256), `bcrypt` for passwords |
| Async runtime | Tokio 1 |
| Serialisation | `serde` + `serde_json` |
| CORS | `rocket_cors` 0.6 |
| WebSockets | `rocket_ws` 0.1 (chat) |
| Logging | `tracing` + `tracing-subscriber` |

---

## Workspace layout

```
teamder-sys/
├── Cargo.toml              # Workspace root — pin shared dep versions here
├── Rocket.toml             # Rocket server config (port, workers, limits)
├── .env / .env.example     # Environment variable overrides
├── uploads/                # Runtime file storage (gitignored)
│   └── <user_id>/
│       ├── avatar/         # Profile photo
│       ├── portfolio/      # Portfolio files
│       └── resume/         # CV/resume files
└── crates/
    ├── teamder-core/       # Pure domain — models, error types, skill matching
    ├── teamder-db/         # MongoDB repository layer
    └── teamder-api/        # Rocket binary — routes, guards, state
```

---

## Running locally

```bash
cp .env.example .env        # then edit as needed
~/.cargo/bin/cargo run -p teamder-api
# → http://localhost:8000
```

MongoDB must be running locally on port 27017 (default). The server seeds the skills catalog on first run if it is empty.

---

## Environment variables

| Variable | Default | Notes |
|----------|---------|-------|
| `MONGODB_URI` | `mongodb://localhost:27017` | Full connection URI |
| `DB_NAME` | `teamder` | Database name |
| `JWT_SECRET` | `teamder-dev-secret-change-me` | **Change in production** |
| `RUST_LOG` | `info` | Tracing level (`debug`, `info`, `warn`, `error`) |

---

## API — base path `/api/v1`

All protected routes require `Authorization: Bearer <jwt>`.  
`—` = public, `Bearer` = any authenticated user, `Admin` = `is_admin: true` in JWT claims.

### Auth

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/register` | — | Register; returns `{token, user}` |
| POST | `/auth/login` | — | Login; returns `{token, user}` |
| POST | `/auth/forgot-password` | — | Issue password reset token |
| POST | `/auth/reset-password` | — | Consume reset token, set new password |

### Users

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/users` | — | List users; `?q=` search, `?limit=`, `?skip=` |
| GET | `/users/me` | Bearer | Current user profile |
| GET | `/users/:id` | — | Profile by ID |
| PATCH | `/users/:id` | Bearer | Update own profile (or admin) |
| DELETE | `/users/:id` | Bearer | Delete own account (or admin) |
| POST | `/users/me/change-password` | Bearer | Change password |
| POST | `/users/me/onboard` | Bearer | Mark onboarding complete |

### Uploads

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/uploads/avatar` | Bearer | Upload profile photo (image/* only); sets `user.avatar_url` |
| POST | `/uploads/portfolio` | Bearer | Upload portfolio file; appends to `user.portfolio` |
| POST | `/uploads/resume` | Bearer | Upload resume; sets `user.resume_url` |
| DELETE | `/uploads?path=…` | Bearer | Delete own uploaded file |

Uploaded files are served as static files under `/uploads/<user_id>/…` by Rocket's `FileServer`.

### Invites

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/invites` | Bearer | All invites sent/received by current user |
| GET | `/invites/:id` | Bearer | Single invite |
| POST | `/invites` | Bearer | Send invite — **409 Conflict if a pending invite already exists** for the same sender→recipient+context |
| POST | `/invites/:id/respond` | Bearer | Accept or decline `{accept: bool}` (recipient only) |
| DELETE | `/invites/:id` | Bearer | Delete (sender only) |

### Projects

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/projects` | — | List; `?status=`, `?q=` |
| GET | `/projects/my` | Bearer | Led by current user |
| GET | `/projects/joined` | Bearer | Projects user is a member of |
| GET | `/projects/:id` | — | Project detail |
| POST | `/projects` | Bearer | Create |
| PATCH | `/projects/:id` | Bearer | Update (owner or admin) |
| DELETE | `/projects/:id` | Bearer | Delete (owner or admin) |
| GET | `/projects/:id/recommend` | — | Recommend users by skill match |

### Join Requests

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/join-requests` | Bearer | Apply to join a project or group |
| GET | `/join-requests/incoming` | Bearer | Applications received as owner |
| GET | `/join-requests/sent` | Bearer | Applications sent by current user |
| POST | `/join-requests/:id/respond` | Bearer | Accept or decline (owner only) |

### Competitions

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/competitions` | — | List all |
| GET | `/competitions/featured` | — | Featured only |
| GET | `/competitions/:id` | — | Detail |
| POST | `/competitions` | Admin | Create |
| POST | `/competitions/:id/register` | Bearer | Register team |
| POST | `/competitions/:id/interest` | Bearer | Express interest |

### Study Groups

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/study-groups` | — | List; `?open=true` |
| GET | `/study-groups/:id` | — | Detail |
| POST | `/study-groups` | Bearer | Create |
| POST | `/study-groups/:id/join` | Bearer | Join |
| POST | `/study-groups/:id/checkin` | Bearer | Daily check-in |
| GET | `/study-groups/joined` | Bearer | Groups current user is in |

### Peer Reviews

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/peer-reviews/for/:user_id` | — | Reviews written about a user |
| POST | `/peer-reviews` | Bearer | Submit review with scores (1–5 for skill, communication, reliability, teamwork) |

### Chat

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/chat/conversations` | Bearer | List conversation partners |
| GET | `/chat/messages/:partner_id` | Bearer | Message history |
| POST | `/chat/messages` | Bearer | Send message |

### Notifications

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/notifications` | Bearer | List unread+recent; `?limit=` |
| POST | `/notifications/:id/read` | Bearer | Mark as read |

### Skills

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/skills/catalog` | — | All skill names in the catalog |
| POST | `/admin/skills` | Admin | Add skill to catalog |
| DELETE | `/admin/skills/:name` | Admin | Remove skill |

### Search

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/search` | — | Cross-entity search; `?q=&type=users\|projects\|…` |

### Admin

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/admin/stats` | Admin | Platform-wide counts |
| GET | `/admin/timeseries` | Admin | Growth data; `?range=30d\|90d\|365d` |
| GET | `/admin/users` | Admin | Full user list |
| GET | `/admin/projects` | Admin | Full project list |
| POST | `/admin/users/:id/promote` | Admin | Toggle admin flag `?value=true\|false` |
| POST | `/admin/projects/:id/promote` | Admin | Toggle `is_promoted` |
| GET | `/admin/export/users.csv` | Admin | CSV export |

### Health

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | `{status: "ok", service, version}` |

---

## Authentication details

- JWT: HS256, 30-day expiry
- Claims: `sub` (user_id), `is_admin`, `iat`, `exp`
- Token stored by frontend in `localStorage` as `teamder_token`
- Password hashing: bcrypt cost=12
- Password reset: single-use hex token with expiry stored on the user document; cleared after use

---

## Database (MongoDB)

Database name: `teamder` (configurable via `DB_NAME`)

### Collections

**`users`** — Core user document. Key fields:
- `_id`: UUID string
- `email`, `password_hash`, `name`, `initials`, `role`, `department`, `university`, `year`
- `avatar_url`: optional — set by `POST /uploads/avatar`; displayed instead of gradient+initials
- `gradient`: CSS gradient string (default avatar background when no `avatar_url`)
- `skills: [{name, level}]`, `skill_tags: [String]`
- `work_mode`: `"remote" | "hybrid" | "in_person"`
- `availability`: `"open_for_collab" | "busy" | "unavailable"`
- `portfolio: [{title, kind, description?, url?}]`
- `resume_url`, `headline`, `bio: [String]`, `languages`, `social_links`, `interests`, `timezone`, `goals`
- `rating`, `projects_done`, `collaborations` (cached aggregates)
- `is_admin`, `onboarded`, `notify_email`, `notify_in_app`, `is_public`
- `reset_token`, `reset_token_expires_at` (password reset flow)

**`invites`** — Collaboration invitations. Key fields:
- `from_user_id`, `to_user_id`, `project_id?`, `study_group_id?`, `message?`
- `status`: `"pending" | "accepted" | "declined" | "expired"`
- `created_at`, `expires_at` (7 days after creation)
- Duplicate prevention: `find_pending_between()` ensures only one pending invite per sender→recipient+context

**`projects`** — Team projects. Key fields:
- `lead_user_id`, `name`, `icon`, `icon_bg`, `status`, `description`, `goals`
- `roles: [{name, count_needed, filled}]`, `skills: [String]`
- `team: [{user_id, initials, color, joined_at}]`
- `collab`, `join_mode`, `is_promoted`, `is_public`

**`notifications`** — In-app notification feed.
- `kind`: `"invite" | "invite_accepted" | "invite_declined" | "review" | "message" | …`
- `link`: deep-link path (e.g. `"/invites"`)
- `read`: bool

**`peer_reviews`** — Post-project reviews.
- `scores: {skill, communication, reliability, teamwork}` (1–5 each)
- `endorsed_skills: [String]`

Other collections: `study_groups`, `competitions`, `competition_teams`, `join_requests`, `messages`, `bookmarks`, `skill_catalog`, `project_updates`

---

## Crate responsibilities

### `teamder-core`
- `error.rs` — `TeamderError` enum: `NotFound | Unauthorized | Forbidden | Validation | Conflict | Database | Internal`
- `models/` — Plain Rust structs with `Serialize/Deserialize`. Each model has a `new()` constructor and a `*Response` DTO that strips sensitive fields (`password_hash`, `reset_token`).
- `skills.rs` — Skill validation against catalog + match-score algorithm (shared-skills + shared-project-history percentage)

### `teamder-db`
- `client.rs` — `DbClient::connect(uri, db_name)` — MongoDB connection pool
- `seed.rs` — Seeds skill catalog on startup if empty
- `repos/` — One struct per collection. All `async fn` returning `Result<_, TeamderError>`. Pattern: `find_by_id`, `find_many_by_ids`, `list`, `search`, `create`, `update`, `delete`

### `teamder-api`
- `main.rs` — Connects DB, builds Rocket, mounts all route groups, attaches `AppState`, serves `/uploads` as static files
- `state.rs` — `AppState { users, invites, projects, … all repos … }`
- `error.rs` — `ApiError` wraps `TeamderError` → JSON `{"error": {"message": "…"}}`
- `auth.rs` — `create_token()` / `verify_token()`
- `guards.rs` — `AuthUser` (Bearer token) and `OptionalAuth` request guards
- `routes/` — One module per resource group

---

## Adding a new feature (checklist)

1. Add model struct in `teamder-core/src/models/<name>.rs` with `new()` + `*Response` DTO
2. Add `pub mod <name>;` to `teamder-core/src/models/mod.rs`
3. Write repo in `teamder-db/src/repos/<name>_repo.rs`
4. Export from `teamder-db/src/repos/mod.rs` and add field to `DbClient` in `client.rs`
5. Add repo field to `AppState` in `teamder-api/src/state.rs`
6. Write routes in `teamder-api/src/routes/<name>.rs`
7. Add `pub mod <name>;` to `routes/mod.rs` and mount in `lib.rs`
8. Add matching API client function in `C:\coding\Teamder\lib\api.ts`

---

## Error handling conventions

- Route handlers return `ApiResult<T>` = `Result<Json<T>, ApiError>`
- Use `?` to propagate `TeamderError` (it implements `Into<ApiError>` via the `error.rs` `From` impl)
- Notifications on side-effects (invite sent, invite responded) use `if let Err(e) = …{ tracing::warn!(…) }` — failures do not abort the primary action
- Validation failures return `TeamderError::Validation` → HTTP 422
- Duplicate/conflict conditions return `TeamderError::Conflict` → HTTP 409

---

## File uploads

- Files stored at `uploads/<user_id>/<subdir>/<uuid>-<original_name>.<ext>` relative to the working directory
- URL returned as `/uploads/<user_id>/<subdir>/<filename>` and stored on the user document
- Frontend converts these paths to absolute URLs using `fileUrl()` from `lib/api.ts`
- Avatar: image files only (content-type checked); replaces `user.avatar_url`
- Portfolio: any file type; appended to `user.portfolio` array
- Resume: any file type; replaces `user.resume_url`
