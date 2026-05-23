# Teamder Backend — Rust/Rocket API (v2.0)

## What this project is

Backend API for Teamder, a team-matching platform for university students. Provides REST endpoints + WebSocket chat. Built with Rust, Rocket, and MongoDB.

## Companion frontend

The frontend lives at `E:\QQ-Balls\Teamder` — Next.js 16 app on `http://localhost:3000`.

---

## Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (2021 edition) |
| Web Framework | Rocket 0.5 (JSON + WebSocket) |
| Database | MongoDB 3 (async driver) |
| Auth | jsonwebtoken 9 (HS256) + bcrypt 0.17 |
| CORS | rocket_cors 0.6 |
| Serialization | serde + serde_json |
| Validation | validator 0.19 |
| Time | chrono 0.4 |
| IDs | uuid 1 (v4) |
| Error handling | thiserror + anyhow |
| Logging | tracing + tracing-subscriber |

---

## Running locally

```bash
# Requires: Rust toolchain, MongoDB running on localhost:27017
cargo run -p teamder-api    # → http://localhost:8000
```

On first boot, the skill catalog (12 categories, 100+ skills) is seeded automatically.

Environment variables (or `.env` file):
- `MONGODB_URI` — default `mongodb://localhost:27017`
- `DB_NAME` — default `teamder`
- `JWT_SECRET` — default `teamder-dev-secret-change-in-production`
- `RUST_LOG` — default `info`

---

## Crate structure

```
crates/
├── teamder-core/          # Domain models, errors, skill catalog
│   └── src/
│       ├── models/        # 13 model files (user, project, competition, etc.)
│       ├── error.rs       # TeamderError enum (7 variants → HTTP status codes)
│       └── skills.rs      # Bilingual skill catalog + match scoring
├── teamder-db/            # MongoDB repository layer
│   └── src/
│       ├── client.rs      # DbClient wrapper with repo accessors
│       ├── repos/         # 13 repo files (one per collection)
│       └── seed.rs        # Skill catalog seeding
└── teamder-api/           # Rocket routes + guards + JWT
    └── src/
        ├── main.rs        # Rocket launch, CORS, route mounting
        ├── auth.rs        # JWT create/verify (30-day expiry)
        ├── guards.rs      # AuthUser, OptionalAuth, AdminUser, PublisherUser
        ├── state.rs       # AppState (DbClient + JWT secret + ChatState)
        ├── error.rs       # ApiError → JSON response
        └── routes/        # 18 route modules (50+ endpoints)
```

---

## Database (MongoDB, 14 collections)

users, projects, competitions, competition_teams, study_groups, invites, join_requests, peer_reviews, notifications, messages, bookmarks, skill_categories, skill_tags, project_updates

All IDs are UUID v4 strings. Timestamps are `DateTime<Utc>`.

---

## API endpoints (base: /api/v1)

| Group | Path Prefix | Endpoints |
|-------|------------|-----------|
| Auth | `/auth` | register, login, forgot-password, reset-password |
| Users | `/users` | list, me, get, update, delete, change-password, onboard |
| Projects | `/projects` | CRUD, my, joined, recommend, complete, leave, member management |
| Project Updates | `/projects/:id/updates` | list, create, delete |
| Competitions | `/competitions` | CRUD, featured, register, interest, registrations, publish workflow, winners |
| Competition Teams | `/competition-teams` | CRUD, apply, accept, applications, leave |
| Study Groups | `/study-groups` | CRUD, join, checkin, notes, progress, complete |
| Invites | `/invites` | CRUD, respond (auto-adds to entity), read tracking |
| Join Requests | `/join-requests` | create, incoming, sent, respond (with side effects) |
| Peer Reviews | `/reviews` | for-user, create (recalculates rating) |
| Chat | `/chat` | conversations, messages, send, WebSocket |
| Notifications | `/notifications` | list, mark-read, mark-all-read |
| Bookmarks | `/bookmarks` | list, add, remove |
| Search | `/search` | cross-entity search |
| Skills | `/skills` | catalog |
| Uploads | `/uploads` | avatar, portfolio, resume, banner, application, delete |
| Admin | `/admin` | stats, timeseries, user/project management, CSV export |
| Admin Skills | `/admin/skills` | CRUD categories + tags |
| Health | `/health` | status check |

---

## Key side effects

- **Accept join request**: adds member to entity + auto-activates projects when roles filled + auto-fulls competition teams
- **Accept invite**: adds member to entity (fixed from v1 bug) + notifies sender
- **Create peer review**: recalculates user rating + embeds in user doc
- **Post project update**: notifies all team members
- **Complete study group**: notifies all members

---

## Auth flow

1. Register/login → returns `{token, user}`
2. JWT stored client-side, sent as `Authorization: Bearer <token>`
3. 30-day expiry, HS256 signing
4. Guards: AuthUser, AdminUser, PublisherUser extract user from JWT
