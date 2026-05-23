# API Reference

All endpoints are mounted under `/api/v1`. Requests and responses use JSON unless otherwise noted.

**Auth levels**:
- **None** -- no authentication required
- **Bearer** -- requires `Authorization: Bearer <token>` header
- **Admin** -- requires Bearer token + user must have `is_admin: true`
- **Publisher** -- requires Bearer token + user must have `is_publisher: true` OR `is_admin: true`

---

## Health

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | None | Returns server status |

**Response**:
```json
{ "status": "ok" }
```

---

## Auth

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/register` | None | Create a new user account |
| POST | `/auth/login` | None | Authenticate and receive a JWT |
| POST | `/auth/forgot-password` | None | Generate a password reset token |
| POST | `/auth/reset-password` | None | Reset password using a reset token |

### POST `/auth/register`

**Request body**:
```json
{
  "email": "string (required)",
  "password": "string (required)",
  "name": "string (required)",
  "role": "string?",
  "department": "string?",
  "university": "string? (default: Fu Jen Catholic University)",
  "year": "string? (default: N/A)",
  "bio": "[string]?",
  "skills": "[{ name, level }]?",
  "skill_tags": "[string]?",
  "languages": "[string]? (default: [Chinese, English])",
  "interests": "[string]?"
}
```

**Response** (200):
```json
{
  "token": "JWT string",
  "user": { ...UserResponse }
}
```

**Errors**: 422 (validation), 409 (email already registered)

**Side effects**: Creates user document, hashes password with bcrypt (cost 12), auto-generates initials from name.

### POST `/auth/login`

**Request body**:
```json
{
  "email": "string",
  "password": "string"
}
```

**Response** (200):
```json
{
  "token": "JWT string",
  "user": { ...UserResponse }
}
```

**Errors**: 401 (invalid email or password)

### POST `/auth/forgot-password`

**Request body**:
```json
{
  "email": "string"
}
```

**Response** (200):
```json
{
  "success": true,
  "reset_token": "64-char hex string"
}
```

If the email does not exist, returns success with an empty token (does not reveal email existence). The reset token expires after 30 minutes.

### POST `/auth/reset-password`

**Request body**:
```json
{
  "token": "string",
  "password": "string"
}
```

**Response** (200):
```json
{ "success": true }
```

**Errors**: 422 (invalid or expired reset token)

**Side effects**: Hashes new password, clears reset token fields.

---

## Users

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/users?page&limit&q` | None | List users (paginated, searchable) |
| GET | `/users/me` | Bearer | Get the authenticated user's profile |
| GET | `/users/<id>` | None (OptionalAuth) | Get a user by ID |
| PATCH | `/users/<id>` | Bearer | Update a user's profile |
| DELETE | `/users/<id>` | Bearer | Delete a user |
| POST | `/users/me/change-password` | Bearer | Change own password |
| POST | `/users/me/onboard` | Bearer | Mark user as onboarded |

### GET `/users`

**Query params**: `page` (default 1), `limit` (default 20), `q` (search string)

**Response**:
```json
{
  "users": [UserResponse],
  "total": 42,
  "page": 1,
  "limit": 20
}
```

### GET `/users/<id>`

If the caller is authenticated (OptionalAuth), the response includes a computed `match_score` (0-100) based on Jaccard similarity of skill tags between the viewer and the target user.

### PATCH `/users/<id>`

Only the user themselves or an admin can update a profile. All fields are optional. Accepts the full `UpdateUserRequest` shape including name, bio, skills, skill_tags, portfolio, social_links, availability, work_mode, gradient, notification preferences, etc.

### POST `/users/me/change-password`

**Request body**:
```json
{
  "old_password": "string",
  "new_password": "string"
}
```

**Errors**: 422 (current password is incorrect)

---

## Projects

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/projects?page&limit&status&q` | None | List projects (paginated, filterable) |
| GET | `/projects/my` | Bearer | List projects where the user is the lead |
| GET | `/projects/joined` | Bearer | List projects where the user is a team member |
| GET | `/projects/<id>` | None | Get project by ID |
| POST | `/projects` | Bearer | Create a new project |
| PATCH | `/projects/<id>` | Bearer | Update a project (lead only) |
| DELETE | `/projects/<id>` | Bearer | Delete a project (lead or admin) |
| GET | `/projects/<id>/recommend` | Bearer | Get recommended users for a project |
| POST | `/projects/<id>/complete` | Bearer | Mark a project as completed (lead only) |
| POST | `/projects/<id>/leave` | Bearer | Leave a project (non-lead members only) |
| POST | `/projects/<id>/remove-member/<user_id>` | Bearer | Remove a member (lead only) |
| POST | `/projects/<id>/set-role/<user_id>` | Bearer | Set a member's role (lead only) |

### POST `/projects`

**Request body**:
```json
{
  "name": "string (required)",
  "description": "string?",
  "goals": "string?",
  "roles": "[{ name, count_needed }]?",
  "skills": "[string]?",
  "deadline": "string?",
  "collab": "string?",
  "duration": "string?",
  "category": "string?",
  "is_public": "bool? (default true)",
  "join_mode": "string? (default 'direct')",
  "icon": "string? (default 'Pr')",
  "icon_bg": "string?",
  "banner_image": "string?"
}
```

**Side effects**: The creator is automatically added as the first team member with role "Lead".

### GET `/projects/<id>/recommend`

Returns up to 20 users ranked by skill overlap with the project's required skills, excluding existing team members. Each returned user includes a `match_score`.

### POST `/projects/<id>/set-role/<user_id>`

**Request body**:
```json
{ "role": "string" }
```

---

## Project Updates

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/projects/<id>/updates` | None | List updates for a project |
| POST | `/projects/<id>/updates` | Bearer | Post an update (team members only) |
| DELETE | `/projects/<id>/updates/<update_id>` | Bearer | Delete an update (author or lead only) |

### POST `/projects/<id>/updates`

**Request body**:
```json
{
  "kind": "progress | milestone | announcement | help_wanted",
  "title": "string",
  "body": "string?"
}
```

**Side effects**: Notifies all other team members.

---

## Competitions

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/competitions?page&limit&status&q` | None | List published competitions |
| GET | `/competitions/featured` | None | List featured competitions |
| GET | `/competitions/mine` | Publisher | List competitions owned by the publisher |
| GET | `/competitions/pending` | Admin | List competitions pending review |
| GET | `/competitions/<id>` | None (OptionalAuth) | Get competition by ID |
| POST | `/competitions` | Publisher | Create a competition |
| PATCH | `/competitions/<id>` | Publisher | Update a competition (owner or admin) |
| POST | `/competitions/<id>/register` | Bearer | Register for a competition |
| POST | `/competitions/<id>/interest` | Bearer | Toggle interest (like/unlike) |
| GET | `/competitions/<id>/registrations` | Publisher | List registrations (owner or admin) |
| POST | `/competitions/<id>/submit-review` | Publisher | Submit draft for admin review |
| POST | `/competitions/<id>/approve` | Admin | Approve a pending competition |
| POST | `/competitions/<id>/reject` | Admin | Reject a pending competition |
| POST | `/competitions/<id>/winners` | Admin | Set competition winners |

### POST `/competitions/<id>/register`

**Request body**:
```json
{
  "team_name": "string?",
  "motivation": "string?",
  "skills": "[string]?",
  "contact_email": "string?"
}
```

**Errors**: 409 (already registered)

### POST `/competitions/<id>/interest`

Toggles the user's interest status. Returns:
```json
{ "interested": true }
```

### GET `/competitions/<id>`

The response includes computed fields based on the viewer:
- `registration_count` -- total registrations
- `interested_count` -- total interested users
- `is_registered_by_viewer` -- whether the authenticated viewer is registered
- `is_interested_by_viewer` -- whether the authenticated viewer is interested
- `registrations` -- full list (only if the viewer is the publisher/admin)

### POST `/competitions/<id>/reject`

**Request body**:
```json
{ "note": "string?" }
```

### POST `/competitions/<id>/winners`

**Request body**:
```json
{ "winner_user_ids": ["string"] }
```

---

## Competition Teams

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/competition-teams` | Bearer | Create a team for a competition |
| GET | `/competition-teams/<id>` | None | Get team by ID |
| PATCH | `/competition-teams/<id>` | Bearer | Update team (lead only) |
| POST | `/competition-teams/<id>/apply` | Bearer | Apply to join a team |
| POST | `/competition-teams/<id>/accept/<user_id>` | Bearer | Accept a team applicant (lead only) |
| GET | `/competition-teams/<id>/applications` | Bearer | List pending applications (lead only) |
| POST | `/competition-teams/<id>/leave` | Bearer | Leave a team (non-lead only) |
| GET | `/competition-teams/competition/<comp_id>` | None | List teams for a competition |
| GET | `/competition-teams/mine` | Bearer | List teams where the user is lead |

### POST `/competition-teams`

**Request body**:
```json
{
  "competition_id": "string",
  "competition_name": "string",
  "name": "string",
  "description": "string?",
  "max_members": "u8? (default 5)",
  "looking_for": "[string]?",
  "open_roles": "[string]?"
}
```

**Side effects**: Creator is added as first member with role "Lead".

### POST `/competition-teams/<id>/apply`

**Request body**:
```json
{ "message": "string?" }
```

**Side effects**: Creates a join request, notifies the team lead.

### POST `/competition-teams/<id>/accept/<user_id>`

**Side effects**: Adds user to team, updates join request status to "accepted", notifies the applicant. If the team reaches `max_members`, status is auto-set to "full".

---

## Study Groups

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/study-groups?page&limit&open_only` | None | List study groups |
| GET | `/study-groups/joined` | Bearer | List groups the user has joined |
| GET | `/study-groups/<id>` | None | Get a study group by ID |
| POST | `/study-groups` | Bearer | Create a study group |
| POST | `/study-groups/<id>/join` | Bearer | Join a study group |
| POST | `/study-groups/<id>/checkin` | Bearer | Daily check-in (members only) |
| POST | `/study-groups/<id>/notes` | Bearer | Add a study note (members only) |
| DELETE | `/study-groups/<id>/notes/<note_id>` | Bearer | Delete a note (author or creator) |
| POST | `/study-groups/<id>/leave` | Bearer | Leave the group (non-creator only) |
| POST | `/study-groups/<id>/progress` | Bearer | Set current week (creator only) |
| POST | `/study-groups/<id>/complete` | Bearer | Mark as completed (creator only) |
| PATCH | `/study-groups/<id>` | Bearer | Update group settings (creator only) |
| DELETE | `/study-groups/<id>` | Bearer | Delete the group (creator or admin) |

### POST `/study-groups`

**Request body**:
```json
{
  "name": "string (required)",
  "goal": "string?",
  "icon": "string? (default 'Sg')",
  "icon_bg": "string?",
  "subject": "string? (default 'General')",
  "tags": "[string]?",
  "max_members": "u8? (default 6)",
  "schedule": "string?",
  "duration_weeks": "u8?",
  "is_open": "bool? (default true)",
  "join_mode": "string? (default 'direct')",
  "banner_image": "string?",
  "description": "string?"
}
```

### POST `/study-groups/<id>/notes`

**Request body**:
```json
{
  "title": "string",
  "body": "string"
}
```

### POST `/study-groups/<id>/progress`

**Request body**:
```json
{ "current_week": 3 }
```

### POST `/study-groups/<id>/complete`

**Side effects**: Sets status to "completed", notifies all members.

---

## Invites

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/invites` | Bearer | List invites for the authenticated user |
| GET | `/invites/<id>` | Bearer | Get invite by ID (sender or recipient) |
| POST | `/invites` | Bearer | Send an invite |
| POST | `/invites/<id>/respond` | Bearer | Accept or decline an invite |
| PATCH | `/invites/<id>/read` | Bearer | Mark an invite as read |
| POST | `/invites/read-all` | Bearer | Mark all invites as read |
| DELETE | `/invites/<id>` | Bearer | Delete an invite (sender only) |

### POST `/invites`

**Request body**:
```json
{
  "to_user_id": "string",
  "project_id": "string?",
  "study_group_id": "string?",
  "competition_team_id": "string?",
  "message": "string?"
}
```

Exactly one of `project_id`, `study_group_id`, or `competition_team_id` should be provided. Invites expire after 7 days. Duplicate pending invites are rejected (409).

**Side effects**: Creates a notification for the recipient.

### POST `/invites/<id>/respond`

**Request body**:
```json
{ "accept": true }
```

**Side effects on accept**: Adds the user to the referenced project/study group/competition team. Notifies the sender.

**Side effects on decline**: Notifies the sender.

---

## Join Requests

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/join-requests` | Bearer | Submit a join request |
| GET | `/join-requests/incoming` | Bearer | List requests where the user is the entity owner |
| GET | `/join-requests/sent` | Bearer | List requests the user has sent |
| POST | `/join-requests/<id>/respond` | Bearer | Accept or decline (entity owner only) |

### POST `/join-requests`

**Request body** (rich application form):
```json
{
  "entity_type": "project | study_group | competition_team",
  "entity_id": "string",
  "message": "string?",
  "motivation": "string?",
  "role_wanted": "string?",
  "hours_per_week": "string?",
  "portfolio_url": "string?",
  "relevant_experience": "string?",
  "availability_start": "string?",
  "can_meet_in_person": "bool?",
  "additional_links": "[string]?",
  "comm_channels": "[string]?",
  "timezone": "string?",
  "agreed_to_coc": "bool?",
  "skill_confidence": "[string]?"
}
```

**Side effects**: Resolves entity name and owner from the database, creates notification for the owner.

### POST `/join-requests/<id>/respond`

**Request body**:
```json
{
  "accept": true,
  "note": "string?"
}
```

**Side effects on accept**:
- Adds the user to the entity (project/study group/competition team)
- For projects: assigns `role_wanted`, increments role filled count, auto-activates project if all roles are filled
- For competition teams: auto-sets status to "full" if `max_members` reached
- Notifies the applicant

**Side effects on decline**: Notifies the applicant with optional note.

---

## Peer Reviews

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/reviews/for/<user_id>` | None | Get all reviews for a user |
| POST | `/reviews` | Bearer | Submit a peer review |

### POST `/reviews`

**Request body**:
```json
{
  "reviewee_id": "string",
  "project_id": "string?",
  "study_group_id": "string?",
  "project_name": "string?",
  "scores": {
    "skill": 1-5,
    "communication": 1-5,
    "reliability": 1-5,
    "teamwork": 1-5
  },
  "body": "string?",
  "endorsed_skills": "[string]?"
}
```

**Side effects**:
- Recalculates the reviewee's average rating across all reviews
- Pushes a cached review summary to the reviewee's `reviews` array on the user document
- Creates a notification for the reviewee

---

## Chat

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/chat/conversations` | Bearer | List conversations with partner metadata |
| GET | `/chat/messages/<partner_id>` | Bearer | Get message history with a partner |
| POST | `/chat/messages` | Bearer | Send a message |
| GET | `/chat/ws?token=<jwt>` | Token in query | WebSocket connection for real-time messages |

### GET `/chat/conversations`

**Response**: Array of conversation summaries:
```json
[
  {
    "partner_id": "string",
    "partner_name": "string",
    "partner_avatar": "string?",
    "partner_initials": "string",
    "partner_gradient": "string",
    "last_message": "string",
    "unread_count": 3,
    "updated_at": "ISO 8601"
  }
]
```

### GET `/chat/messages/<partner_id>`

Returns all messages between the authenticated user and the partner. **Side effect**: marks all messages from the partner as read.

### POST `/chat/messages`

**Request body**:
```json
{
  "to_user_id": "string",
  "content": "string"
}
```

**Side effects**: Persists the message and broadcasts to the recipient's WebSocket channel.

### GET `/chat/ws?token=<jwt>`

Upgrades to a WebSocket connection. The JWT is passed as a query parameter (not a header) because the browser WebSocket API does not support custom headers.

**Incoming messages** (client to server):
```json
{
  "to_user_id": "string",
  "content": "string"
}
```

**Outgoing messages** (server to client): Full `Message` objects as JSON strings, pushed when another user sends a message to the connected user.

---

## Notifications

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/notifications?limit` | Bearer | List notifications (default limit 50) |
| POST | `/notifications/<id>/read` | Bearer | Mark a notification as read |
| POST | `/notifications/read-all` | Bearer | Mark all notifications as read |

Notification `kind` values: `invite`, `invite_accepted`, `invite_declined`, `join_request`, `join_accepted`, `join_declined`, `review`, `message`, `project_update`, `competition_recommend`, `system`

---

## Bookmarks

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/bookmarks` | Bearer | List all bookmarks for the user |
| POST | `/bookmarks` | Bearer | Add a bookmark |
| POST | `/bookmarks/remove` | Bearer | Remove a bookmark |

### POST `/bookmarks`

**Request body**:
```json
{
  "kind": "user | project | competition | study_group | competition_team",
  "entity_id": "string",
  "label": "string?"
}
```

### POST `/bookmarks/remove`

**Request body**:
```json
{
  "kind": "string",
  "entity_id": "string"
}
```

---

## Skills

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/skills/catalog` | None | Get the full skill catalog |

**Response**:
```json
{
  "categories": [
    { "id": "frontend", "label": "Frontend", "label_zh": "...", "order": 0, ... }
  ],
  "tags": [
    { "id": "uuid", "name": "React", "name_zh": "React", "category_key": "frontend", "order": 0, "is_active": true, ... }
  ]
}
```

---

## Uploads

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/uploads/avatar` | Bearer | Upload an avatar image |
| POST | `/uploads/portfolio` | Bearer | Upload a portfolio file |
| POST | `/uploads/resume` | Bearer | Upload a resume |
| POST | `/uploads/banner` | Bearer | Upload a banner image |
| POST | `/uploads/application` | Bearer | Upload an application attachment |
| DELETE | `/uploads?path=<url_path>` | Bearer | Delete an uploaded file |

All upload endpoints accept `multipart/form-data` with a `file` field.

**Portfolio uploads** also accept `title` and `kind` form fields.

**Response** for all uploads:
```json
{ "url": "/uploads/<user_id>/<type>/<uuid>.<ext>" }
```

Files are stored on disk at `uploads/<user_id>/<type>/`. Filenames are sanitized and replaced with UUID v4 to prevent collisions.

**DELETE `/uploads?path=<url_path>`**: The path must start with `/uploads/<user_id>/` and must not contain `..`. Only the owning user can delete their files.

---

## Search

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/search?q&kind&limit` | None | Cross-entity search |

**Query params**:
- `q` (required) -- search query string
- `kind` (optional) -- filter by entity type: `user`, `project`, `competition`, `study_group`
- `limit` (optional, default 20) -- max results

**Response**:
```json
{
  "results": [
    {
      "kind": "user | project | competition | study_group",
      "id": "string",
      "name": "string",
      "description": "string?",
      "icon": "string?"
    }
  ]
}
```

Searches across users, projects, competitions, and study groups simultaneously. Results are capped at `limit` total.

---

## Admin

All admin endpoints require the `AdminUser` guard (Bearer token + `is_admin: true`).

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/admin/stats` | Admin | Dashboard statistics |
| GET | `/admin/timeseries?range` | Admin | Time-series data for charts |
| GET | `/admin/users?page&limit` | Admin | List all users (paginated) |
| GET | `/admin/projects` | Admin | List all projects |
| GET | `/admin/study-groups` | Admin | List all study groups |
| GET | `/admin/competitions` | Admin | List all competitions (all statuses) |
| POST | `/admin/users/<id>/promote` | Admin | Toggle admin status |
| POST | `/admin/users/<id>/publisher` | Admin | Toggle publisher status |
| POST | `/admin/projects/<id>/promote` | Admin | Toggle project promoted status |
| DELETE | `/admin/users/<id>` | Admin | Delete a user |
| DELETE | `/admin/projects/<id>` | Admin | Delete a project |
| GET | `/admin/export/users.csv` | Admin | Export all users as CSV |

### GET `/admin/stats`

**Response**:
```json
{
  "users": 150,
  "projects": 42,
  "competitions": 8,
  "study_groups": 15
}
```

### GET `/admin/timeseries?range`

**Query params**: `range` = `7d`, `30d`, `90d`, `365d`, or `1y` (default `30d`)

**Response**: Array of daily buckets with `date`, `users`, `projects` counts.

### Admin Skills

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/admin/skills` | Admin | List all skill categories and tags |
| POST | `/admin/skills/categories` | Admin | Create a skill category |
| PATCH | `/admin/skills/categories/<key>` | Admin | Update a skill category |
| DELETE | `/admin/skills/categories/<key>` | Admin | Delete a skill category |
| POST | `/admin/skills/tags` | Admin | Create a skill tag |
| PATCH | `/admin/skills/tags/<id>` | Admin | Update a skill tag |
| DELETE | `/admin/skills/tags/<id>` | Admin | Delete a skill tag |

### POST `/admin/skills/categories`

**Request body**:
```json
{
  "key": "string (becomes the _id)",
  "label": "string",
  "label_zh": "string",
  "order": "i32?"
}
```

### POST `/admin/skills/tags`

**Request body**:
```json
{
  "name": "string",
  "name_zh": "string",
  "category_key": "string",
  "order": "i32?",
  "is_active": "bool? (default true)"
}
```
