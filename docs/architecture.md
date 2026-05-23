# Architecture

This document describes the internal structure, design decisions, and data flow of the Teamder backend.

---

## Workspace Structure

The project is a Cargo workspace with three crates, each with a single responsibility:

```
teamder-sys/
+-- Cargo.toml              Workspace root, shared dependencies
+-- Rocket.toml              Rocket server configuration
+-- .env.example             Environment variable template
+-- Dockerfile               Multi-stage production build
+-- uploads/                 User-uploaded files (avatars, portfolios, etc.)
+-- docs/                    This documentation
+-- crates/
    +-- teamder-core/        Pure domain layer (no I/O)
    |   +-- src/
    |       +-- lib.rs
    |       +-- error.rs         TeamderError enum, ErrorResponse
    |       +-- skills.rs        Bilingual skill catalog, match-score algorithm
    |       +-- models/
    |           +-- mod.rs
    |           +-- user.rs          User, UserResponse, CreateUserRequest, UpdateUserRequest
    |           +-- project.rs       Project, ProjectResponse, CreateProjectRequest, etc.
    |           +-- competition.rs   Competition, CompetitionResponse, Registration
    |           +-- competition_team.rs  CompetitionTeam, CompTeamMember
    |           +-- study_group.rs   StudyGroup, GroupMember, StudyNote
    |           +-- invite.rs        Invite
    |           +-- join_request.rs  JoinRequest, CreateJoinRequestBody, JoinRequestResponse
    |           +-- peer_review.rs   PeerReview, ReviewScores
    |           +-- notification.rs  Notification
    |           +-- message.rs       Message
    |           +-- bookmark.rs      Bookmark
    |           +-- skill_catalog.rs StoredSkillCategory, StoredSkillTag
    |           +-- project_update.rs ProjectUpdate
    |
    +-- teamder-db/          Database layer
    |   +-- src/
    |       +-- lib.rs
    |       +-- client.rs        DbClient -- MongoDB connection + repo accessors
    |       +-- seed.rs          Auto-seed skill catalog on first run
    |       +-- repos/
    |           +-- mod.rs
    |           +-- user_repo.rs
    |           +-- project_repo.rs
    |           +-- competition_repo.rs
    |           +-- competition_team_repo.rs
    |           +-- study_group_repo.rs
    |           +-- invite_repo.rs
    |           +-- join_request_repo.rs
    |           +-- peer_review_repo.rs
    |           +-- message_repo.rs
    |           +-- notification_repo.rs
    |           +-- bookmark_repo.rs
    |           +-- skill_catalog_repo.rs
    |           +-- project_update_repo.rs
    |
    +-- teamder-api/         HTTP / WebSocket layer
        +-- src/
            +-- main.rs          Rocket launch, CORS config, route mounting
            +-- lib.rs           Module declarations
            +-- auth.rs          JWT create_token / verify_token
            +-- guards.rs        AuthUser, OptionalAuth, AdminUser, PublisherUser
            +-- state.rs         AppState, ChatState
            +-- error.rs         ApiError wrapper, JSON error responder
            +-- routes/
                +-- mod.rs
                +-- health.rs
                +-- auth.rs
                +-- users.rs
                +-- uploads.rs
                +-- projects.rs
                +-- project_updates.rs
                +-- competitions.rs
                +-- competition_teams.rs
                +-- study_groups.rs
                +-- invites.rs
                +-- join_requests.rs
                +-- peer_reviews.rs
                +-- chat.rs
                +-- notifications.rs
                +-- bookmarks.rs
                +-- search.rs
                +-- skills.rs
                +-- admin.rs
                +-- admin_skills.rs
```

---

## Dependency Flow

```
teamder-core  (zero I/O dependencies)
      ^
      |
teamder-db    (depends on core for models + errors; adds mongodb, bson)
      ^
      |
teamder-api   (depends on db + core; adds Rocket, JWT, bcrypt)
```

**Why three crates?**

1. **Separation of concerns**: Domain models and business rules live in `core` with no database or HTTP dependencies. This makes them trivially testable.
2. **Compile-time guarantees**: `core` cannot accidentally import Rocket or MongoDB. The type system enforces the layering.
3. **Reusability**: `core` and `db` could be used by a CLI tool, a migration script, or a different web framework without pulling in Rocket.

---

## AppState

All shared state is bundled into a single struct, managed by Rocket:

```rust
pub struct AppState {
    pub db: DbClient,          // MongoDB connection + repo accessors
    pub jwt_secret: String,    // Secret for signing/verifying JWTs
    pub chat_state: ChatState, // Per-user broadcast channels for WebSocket
}
```

`AppState` is registered with `rocket.manage(app_state)` at startup and injected into route handlers via `&State<AppState>`.

### ChatState

Real-time messaging uses Tokio broadcast channels, one per user:

```rust
pub struct ChatState {
    pub channels: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
}
```

When a message is sent (via REST or WebSocket), it is:
1. Persisted to MongoDB
2. Broadcast to the recipient's channel
3. Forwarded to any connected WebSocket client for that user

Channels are created lazily on first access with a buffer capacity of 64 messages.

---

## Request Lifecycle

```
Client Request
     |
     v
[Rocket Router]  -- matches method + path to a route handler
     |
     v
[Request Guards]  -- AuthUser / AdminUser / PublisherUser / OptionalAuth
     |               extracted from Authorization header, JWT verified
     v
[Route Handler]   -- in crates/teamder-api/src/routes/*.rs
     |               business logic, validation
     v
[Repository]      -- in crates/teamder-db/src/repos/*.rs
     |               BSON query construction, MongoDB operations
     v
[MongoDB]         -- read/write via the mongodb driver
     |
     v
[Response]        -- JSON serialized via serde, returned to client
```

### Error Flow

```
Repository returns Err(TeamderError)
     |
     v
Route handler propagates via ? operator -> Result<Json<T>, ApiError>
     |
     v
ApiError implements Rocket's Responder trait
     |
     v
Client receives JSON:
{
  "error": {
    "code": "NOT_FOUND",         // derived from TeamderError variant
    "message": "User not found"  // human-readable message
  }
}
```

Error variant mapping:

| TeamderError variant | HTTP Status | Code string |
|---------------------|-------------|-------------|
| `NotFound(msg)` | 404 | `NOT_FOUND` |
| `Unauthorized(msg)` | 401 | `UNAUTHORIZED` |
| `Forbidden(msg)` | 403 | `FORBIDDEN` |
| `Validation(msg)` | 422 | `VALIDATION_ERROR` |
| `Conflict(msg)` | 409 | `CONFLICT` |
| `Database(msg)` | 500 | `DATABASE_ERROR` |
| `Internal(msg)` | 500 | `INTERNAL_ERROR` |

---

## CORS Configuration

Configured in `main.rs` with `rocket_cors`:

- **Allowed origins**: `http://localhost:3000`, `http://localhost:3001`, `https://teamder.watchandy.me`
- **Allowed methods**: GET, POST, PUT, PATCH, DELETE, OPTIONS
- **Allowed headers**: Authorization, Content-Type, Accept
- **Credentials**: Allowed

---

## Key Design Decisions

### No Foreign Keys

MongoDB does not enforce foreign keys. Referential integrity is handled at the application layer. For example, when a join request is accepted, the route handler:
1. Updates the join request status
2. Adds the user to the entity's member array
3. Creates a notification for the applicant

All three steps happen in the same async handler. There are no database-level transactions or constraints enforcing consistency.

### Denormalized Names

Several collections store denormalized copies of names (e.g., `reviewer_name` on `PeerReview`, `author_name` on `ProjectUpdate`, `competition_name` on `CompetitionTeam`). This avoids JOIN-equivalent lookups on read-heavy paths at the cost of potential staleness if a user renames themselves.

### Embedded Arrays vs. Separate Collections

Members, registrations, notes, and reviews are stored as embedded arrays within their parent documents (e.g., `project.team`, `competition.registrations`, `study_group.notes`). This is intentional for documents that are always read together and whose arrays stay reasonably small (typically under 100 items).

Entities that can grow unboundedly or need independent querying (messages, notifications, bookmarks, join requests) are stored in separate collections.

### UUID v4 for Document IDs

All `_id` fields use UUID v4 strings (via the `uuid` crate) rather than MongoDB's default ObjectId. This simplifies serialization and makes IDs URL-safe without conversion.

### Skill Match Scoring

The `teamder-core::skills` module implements a weighted match-score algorithm:

| Weight | Factor | Description |
|--------|--------|-------------|
| 40% | Skill overlap | Jaccard similarity of skill tag sets + level proximity bonus |
| 15% | Complementarity | How many skills the target has that the viewer lacks |
| 20% | Domain overlap | Category-level Jaccard similarity |
| 15% | Track record | Log-scaled projects_done * normalized rating |
| 10% | Availability | Alignment of availability status |

The score is computed on-the-fly when viewing a user profile (not precomputed).

---

## Static File Serving

Uploaded files are served by Rocket's built-in `FileServer`:

```rust
.mount("/uploads", FileServer::from("uploads"))
```

Files are stored at `uploads/<user_id>/<type>/<uuid>.<ext>` where type is one of: `avatar`, `portfolio`, `resume`, `banners`, `applications`.

---

## Auto-Seeding

On startup, the `seed_if_empty` function checks if the `skill_categories` collection is empty. If so, it seeds 12 skill categories and 60+ skill tags from the hardcoded `SKILL_CATALOG` constant. This ensures the skill catalog is always available without manual database setup.
