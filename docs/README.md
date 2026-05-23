# Teamder Backend -- Documentation

> Rust/Rocket REST API powering the Teamder team-matching platform.

## Table of Contents

- [Getting Started](./getting-started.md) -- prerequisites, setup, and running locally
- [Architecture](./architecture.md) -- workspace structure, design decisions, request lifecycle
- [API Reference](./api-reference.md) -- every endpoint, grouped by domain
- [Database Design](./database-design.md) -- all 14 MongoDB collections with field-level detail
- [Authentication](./authentication.md) -- JWT, guards, role-based access
- [Deployment](./deployment.md) -- building, Docker, production configuration

---

## What is Teamder?

Teamder is a team-matching platform for university students at Fu Jen Catholic University. It helps students:

- **Discover collaborators** by skill, availability, and domain interest
- **Post or join projects** with role-based recruiting and auto-activation
- **Enter competitions** published by approved publishers, form competition teams
- **Create study groups** with check-in streaks, shared notes, and weekly progress
- **Exchange invites** and respond to join requests with rich application forms
- **Review teammates** after collaborating (multi-dimensional peer reviews)
- **Chat in real time** via REST and WebSocket messaging

---

## Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Language | Rust | 2021 edition |
| Web Framework | Rocket | 0.5 |
| Database | MongoDB | 3.x driver (mongodb crate) |
| BSON | bson | 2.x (with chrono support) |
| Auth | JWT (HS256) + bcrypt | jsonwebtoken 9 / bcrypt 0.17 |
| Real-time | WebSocket | rocket_ws 0.1 |
| CORS | rocket_cors | 0.6 |
| Validation | validator | 0.19 |
| Logging | tracing + tracing-subscriber | 0.1 / 0.3 |
| Env | dotenvy | 0.15 |
| Async runtime | Tokio | 1.x (full features) |

---

## Workspace Layout

The project is a Cargo workspace with three crates:

```
teamder-sys/
  crates/
    teamder-core/    Pure domain models, error types, skill catalog
    teamder-db/      MongoDB client, repositories, seed data
    teamder-api/     Rocket application, routes, guards, auth
```

Dependency flow: `teamder-core` <- `teamder-db` <- `teamder-api`

---

## Companion Frontend

The Next.js frontend lives at `../Teamder` and communicates with this API at `http://localhost:8000/api/v1`. See the frontend CLAUDE.md for its own documentation.

---

## Quick Start

```bash
# Prerequisites: Rust toolchain, MongoDB running on localhost:27017

cp .env.example .env
cargo run -p teamder-api

# Verify: GET http://localhost:8000/api/v1/health
# Expected: { "status": "ok" }
```

For full setup instructions, see [Getting Started](./getting-started.md).
