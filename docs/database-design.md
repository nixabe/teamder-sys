# Database Design

The Teamder backend uses MongoDB. All collections live in a single database (default name: `teamder`). There are 14 collections total.

**Conventions**:
- All `_id` fields are UUID v4 strings (not MongoDB ObjectId)
- Timestamps are stored as BSON DateTime (ISO 8601 in JSON)
- Embedded arrays are used for data that is always read with the parent document
- Separate collections are used for unbounded or independently queried data
- Denormalized names (e.g., `author_name`) avoid cross-collection lookups at read time

---

## Collection Overview

| Collection | Description | Approx. growth |
|------------|-------------|----------------|
| `users` | User accounts and profiles | One per registered user |
| `projects` | Collaborative projects | Created by users |
| `competitions` | Competitions published by publishers | Publisher-created, admin-approved |
| `competition_teams` | Teams formed for competitions | User-created |
| `study_groups` | Study groups with check-ins and notes | User-created |
| `invites` | Direct invites to join entities | One per invite sent |
| `join_requests` | Applications to join entities | One per application |
| `peer_reviews` | Peer reviews between collaborators | One per review |
| `notifications` | In-app notifications | System-generated |
| `messages` | Direct messages between users | One per message |
| `bookmarks` | User bookmarks of entities | One per bookmark |
| `skill_categories` | Skill category definitions | Admin-managed, seeded on startup |
| `skill_tags` | Individual skill tag definitions | Admin-managed, seeded on startup |
| `project_updates` | Status updates posted to projects | Team-member-created |

---

## users

User accounts, profiles, skills, portfolio, and cached reviews.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `email` | String | -- | Unique email address |
| `password_hash` | String | -- | bcrypt hash (cost 12) |
| `name` | String | -- | Display name |
| `initials` | String | `""` | Auto-generated from name (e.g., "JD") |
| `role` | String | `""` | Self-described role (e.g., "Frontend Developer") |
| `department` | String | `""` | Academic department |
| `university` | String | `"Fu Jen Catholic University"` | University name |
| `year` | String | `"N/A"` | Academic year |
| `location` | String? | `null` | Location/city |
| `bio` | [String] | `[]` | Multi-paragraph biography |
| `skills` | [Skill] | `[]` | Skills with levels: `{ name: String, level: u8 (0-100) }` |
| `skill_tags` | [String] | `[]` | Flat list of skill names for matching |
| `gradient` | String | `""` | CSS gradient string for avatar background |
| `work_mode` | String? | `null` | "remote", "hybrid", "on_site" |
| `availability` | String? | `null` | "open_for_collab", "busy", "unavailable" |
| `hours_per_week` | String? | `null` | Available hours per week |
| `languages` | [String] | `["Chinese", "English"]` | Spoken languages |
| `portfolio` | [PortfolioItem] | `[]` | `{ title, kind, description?, url? }` |
| `reviews` | [CachedReview] | `[]` | Denormalized review summaries |
| `match_score` | u8? | `null` | Computed at read time, not persisted |
| `rating` | f32 | `0.0` | Average peer review rating (recalculated on review) |
| `projects_done` | u32 | `0` | Completed project count |
| `collaborations` | u32 | `0` | Total collaboration count |
| `avatar_url` | String? | `null` | Path to uploaded avatar |
| `resume_url` | String? | `null` | Path to uploaded resume |
| `reset_token` | String? | `null` | Password reset token (64-char hex) |
| `reset_token_expires_at` | DateTime? | `null` | Reset token expiry (30 min from creation) |
| `is_admin` | bool | `false` | Admin privilege flag |
| `is_publisher` | bool | `false` | Publisher privilege flag |
| `is_public` | bool | `true` | Profile visibility |
| `onboarded` | bool | `false` | Whether user completed onboarding |
| `headline` | String? | `null` | Short tagline |
| `notify_email` | bool | `true` | Email notification preference |
| `notify_in_app` | bool | `true` | In-app notification preference |
| `social_links` | [SocialLink] | `[]` | `{ label, url }` |
| `interests` | [String] | `[]` | Interest tags |
| `timezone` | String? | `null` | User's timezone |
| `goals` | String? | `null` | Career/academic goals |
| `free_days` | [String] | `[]` | Days available for collaboration |
| `created_at` | DateTime | -- | Account creation time |
| `updated_at` | DateTime | -- | Last profile update time |

**Indexes**: Unique index on `email`. Text index on `name` and `email` for search.

**Sub-types**:

- **Skill**: `{ name: String, level: u8 }`
- **PortfolioItem**: `{ title: String, kind: String, description?: String, url?: String }`
- **SocialLink**: `{ label: String, url: String }`
- **CachedReview**: `{ reviewer_id, reviewer_name, project_name, stars: u8, body, created_at }`

---

## projects

Collaborative projects with team management and role tracking.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `name` | String | -- | Project name |
| `lead_user_id` | String | -- | ID of the project lead |
| `icon` | String | `"Pr"` | Display icon text |
| `icon_bg` | String | `""` | Icon background color |
| `status` | String | `"recruiting"` | "recruiting", "active", "completed" |
| `description` | String | `""` | Project description |
| `goals` | String? | `null` | Project goals |
| `roles` | [ProjectRole] | `[]` | `{ name, count_needed: u8, filled: u8 }` |
| `skills` | [String] | `[]` | Required skill tags |
| `team` | [TeamMember] | `[]` | Current team members |
| `deadline` | String? | `null` | Project deadline |
| `collab` | String? | `null` | Collaboration style |
| `duration` | String? | `null` | Expected duration |
| `category` | String? | `null` | Project category |
| `is_public` | bool | `true` | Public visibility |
| `join_mode` | String | `"direct"` | "direct" or "application" |
| `is_promoted` | bool | `false` | Admin-promoted flag |
| `banner_image` | String? | `null` | Banner image URL |
| `created_at` | DateTime | -- | Creation time |
| `updated_at` | DateTime | -- | Last update time |

**Sub-types**:

- **ProjectRole**: `{ name: String, count_needed: u8, filled: u8 }`
- **TeamMember**: `{ user_id, initials, color, joined_at: DateTime, role?: String }`

**Auto-activation**: When a join request is accepted and all roles reach `filled >= count_needed`, the project status auto-transitions from "recruiting" to "active".

---

## competitions

Competitions published by publishers, approved by admins.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `name` | String | -- | Competition name |
| `organizer` | String | -- | Organizing entity name |
| `icon` | String | `"Cp"` | Display icon text |
| `icon_bg` | String | `""` | Icon background color |
| `status` | String | `"open"` | "open", "closed", "completed" |
| `prize` | String | `""` | Prize description |
| `team_size_min` | u8 | `2` | Minimum team size |
| `team_size_max` | u8 | `5` | Maximum team size |
| `deadline` | String? | `null` | Registration deadline |
| `duration` | String | `""` | Competition duration |
| `tags` | [String] | `[]` | Topic tags |
| `description` | String | `""` | Full description |
| `is_featured` | bool | `false` | Featured on homepage |
| `banner_image` | String? | `null` | Banner image URL |
| `publish_status` | String | `"published"` | "draft", "pending_review", "published", "rejected" |
| `publisher_id` | String? | `null` | ID of the publishing user |
| `rejected_note` | String? | `null` | Admin rejection reason |
| `registrations` | [Registration] | `[]` | Embedded registrations |
| `interested_user_ids` | [String] | `[]` | User IDs who expressed interest |
| `winners` | [String] | `[]` | Winner user IDs |
| `created_at` | DateTime | -- | Creation time |
| `updated_at` | DateTime | -- | Last update time |

**Sub-types**:

- **Registration**: `{ user_id, team_name?, registered_at: DateTime, motivation?, skills?: [String], contact_email? }`

**Publishing workflow**: draft -> pending_review (publisher submits) -> published (admin approves) / rejected (admin rejects with note).

---

## competition_teams

Teams formed by users within a competition.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `competition_id` | String | -- | Reference to competition |
| `competition_name` | String | -- | Denormalized competition name |
| `name` | String | -- | Team name |
| `description` | String | `""` | Team description |
| `lead_user_id` | String | -- | Team lead user ID |
| `members` | [CompTeamMember] | `[]` | Team members |
| `max_members` | u8 | `5` | Maximum member count |
| `looking_for` | [String] | `[]` | Skills the team is looking for |
| `open_roles` | [String] | `[]` | Open role names |
| `status` | String | `"recruiting"` | "recruiting", "full" |
| `created_at` | DateTime | -- | Creation time |
| `updated_at` | DateTime | -- | Last update time |

**Sub-types**:

- **CompTeamMember**: `{ user_id, name, initials, role?: String, joined_at: DateTime }`

**Auto-full**: When `members.len() >= max_members`, status is auto-set to "full".

---

## study_groups

Study groups with check-in tracking, shared notes, and weekly progress.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `name` | String | -- | Group name |
| `goal` | String | `""` | Study goal |
| `icon` | String | `"Sg"` | Display icon text |
| `icon_bg` | String | `""` | Icon background color |
| `subject` | String | `"General"` | Subject area |
| `tags` | [String] | `[]` | Topic tags |
| `members` | [GroupMember] | `[]` | Group members with check-in data |
| `max_members` | u8 | `6` | Maximum member count |
| `schedule` | String | `""` | Meeting schedule |
| `duration_weeks` | u8 | `0` | Planned duration in weeks |
| `current_week` | u8 | `1` | Current progress week |
| `is_open` | bool | `true` | Open for new members |
| `status` | String | `"active"` | "active", "completed" |
| `join_mode` | String | `"direct"` | "direct" or "application" |
| `banner_image` | String? | `null` | Banner image URL |
| `notes` | [StudyNote] | `[]` | Shared study notes |
| `description` | String? | `null` | Group description |
| `created_by` | String | -- | Creator user ID |
| `created_at` | DateTime | -- | Creation time |
| `updated_at` | DateTime | -- | Last update time |

**Sub-types**:

- **GroupMember**: `{ user_id, initials, color, joined_at: DateTime, last_checkin?: DateTime, streak: u32 }`
- **StudyNote**: `{ id: String, author_id, author_name, title, body, created_at: DateTime }`

---

## invites

Direct invitations from one user to another to join a project, study group, or competition team.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `from_user_id` | String | -- | Sender user ID |
| `to_user_id` | String | -- | Recipient user ID |
| `project_id` | String? | `null` | Target project (if applicable) |
| `study_group_id` | String? | `null` | Target study group (if applicable) |
| `competition_team_id` | String? | `null` | Target competition team (if applicable) |
| `message` | String? | `null` | Personal message |
| `status` | String | `"pending"` | "pending", "accepted", "declined" |
| `is_read` | bool | `false` | Whether the recipient has read it |
| `created_at` | DateTime | -- | Creation time |
| `expires_at` | DateTime | -- | Expiration time (7 days from creation) |

**Constraints**: Only one pending invite between the same sender-recipient pair for the same entity at a time (enforced at application level).

---

## join_requests

Applications from users to join projects, study groups, or competition teams. Supports a rich application form with detailed fields.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `from_user_id` | String | -- | Applicant user ID |
| `entity_type` | String | -- | "project", "study_group", or "competition_team" |
| `entity_id` | String | -- | Target entity ID |
| `entity_name` | String | `""` | Denormalized entity name |
| `owner_id` | String | -- | Entity owner user ID |
| `message` | String? | `null` | Cover message |
| `status` | String | `"pending"` | "pending", "accepted", "declined" |
| `motivation` | String? | `null` | Why the user wants to join |
| `role_wanted` | String? | `null` | Which role the user is applying for |
| `hours_per_week` | String? | `null` | Time commitment |
| `portfolio_url` | String? | `null` | Portfolio link |
| `relevant_experience` | String? | `null` | Relevant experience description |
| `availability_start` | String? | `null` | When the user can start |
| `can_meet_in_person` | bool? | `null` | In-person meeting preference |
| `additional_links` | [String] | `[]` | Extra links (GitHub, LinkedIn, etc.) |
| `comm_channels` | [String] | `[]` | Preferred communication channels |
| `timezone` | String? | `null` | Applicant's timezone |
| `agreed_to_coc` | bool | `false` | Agreed to code of conduct |
| `skill_confidence` | [String] | `[]` | Skills the applicant is confident in |
| `created_at` | DateTime | -- | Submission time |

**Constraints**: Only one pending request per user per entity at a time (enforced at application level).

---

## peer_reviews

Multi-dimensional peer reviews submitted after collaborating on a project or study group.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `reviewer_id` | String | -- | Reviewer user ID |
| `reviewer_name` | String | -- | Denormalized reviewer name |
| `reviewee_id` | String | -- | Reviewed user ID |
| `project_id` | String? | `null` | Associated project (if applicable) |
| `study_group_id` | String? | `null` | Associated study group (if applicable) |
| `project_name` | String | `""` | Denormalized project/group name |
| `scores` | ReviewScores | -- | Multi-dimensional scores |
| `body` | String | `""` | Written review text |
| `endorsed_skills` | [String] | `[]` | Skills endorsed by the reviewer |
| `created_at` | DateTime | -- | Review submission time |

**Sub-types**:

- **ReviewScores**: `{ skill: u8, communication: u8, reliability: u8, teamwork: u8 }` (each 1-5)
- **average()**: `(skill + communication + reliability + teamwork) / 4.0`

---

## notifications

In-app notifications generated by system events.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `user_id` | String | -- | Recipient user ID |
| `kind` | String | -- | Notification type (see below) |
| `title` | String | -- | Notification title |
| `body` | String | `""` | Notification body text |
| `link` | String? | `null` | Deep link path |
| `read` | bool | `false` | Read status |
| `created_at` | DateTime | -- | Creation time |

**Kind values**: `invite`, `invite_accepted`, `invite_declined`, `join_request`, `join_accepted`, `join_declined`, `review`, `message`, `project_update`, `competition_recommend`, `system`

---

## messages

Direct messages between two users.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `from_user_id` | String | -- | Sender user ID |
| `to_user_id` | String | -- | Recipient user ID |
| `content` | String | -- | Message text |
| `read` | bool | `false` | Read status |
| `created_at` | DateTime | -- | Send time |

**Indexes**: Compound index on `(from_user_id, to_user_id)` and `created_at` for conversation queries.

---

## bookmarks

User bookmarks of any entity type.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `user_id` | String | -- | Bookmarking user ID |
| `kind` | String | -- | "user", "project", "competition", "study_group", "competition_team" |
| `entity_id` | String | -- | Bookmarked entity ID |
| `label` | String | `""` | Optional display label |
| `created_at` | DateTime | -- | Bookmark creation time |

**Constraints**: Unique on `(user_id, kind, entity_id)` at application level.

---

## skill_categories

Skill category definitions. Auto-seeded on first startup with 12 categories.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String | -- | Category key (e.g., "frontend", "backend") |
| `label` | String | -- | English label |
| `label_zh` | String | -- | Traditional Chinese label |
| `order` | i32 | -- | Display order |
| `created_at` | DateTime | -- | Creation time |
| `updated_at` | DateTime | -- | Last update time |

**Seeded categories**: frontend, backend, mobile, database, devops, design, data, blockchain, pm, marketing, business, other

---

## skill_tags

Individual skill tag definitions, each belonging to a category. Auto-seeded on first startup with 60+ tags.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `name` | String | -- | English skill name |
| `name_zh` | String | -- | Traditional Chinese skill name |
| `category_key` | String | -- | Reference to skill_categories._id |
| `order` | i32 | -- | Display order within category |
| `is_active` | bool | `true` | Whether the tag is active/visible |
| `created_at` | DateTime | -- | Creation time |
| `updated_at` | DateTime | -- | Last update time |

---

## project_updates

Status updates, milestones, and announcements posted to projects by team members.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `_id` | String (UUID) | -- | Primary key |
| `project_id` | String | -- | Parent project ID |
| `author_id` | String | -- | Author user ID |
| `author_name` | String | -- | Denormalized author name |
| `kind` | String | -- | "progress", "milestone", "announcement", "help_wanted" |
| `title` | String | -- | Update title |
| `body` | String | `""` | Update body text |
| `created_at` | DateTime | -- | Post time |

---

## Design Notes

### ID Strategy

All documents use UUID v4 strings as `_id` rather than MongoDB's native ObjectId. This provides:
- URL-safe identifiers without encoding
- Consistent format across all collections
- No dependency on MongoDB-specific types in the domain layer

### Embedded vs. Referenced

| Pattern | Used for | Rationale |
|---------|----------|-----------|
| **Embedded array** | team members, registrations, notes, roles | Always read with parent; bounded size |
| **Separate collection** | messages, notifications, bookmarks, reviews | Unbounded growth; independently queried |
| **Denormalized field** | author_name, reviewer_name, entity_name | Avoids cross-collection lookups on reads |

### Consistency Model

There are no database-level transactions. Consistency is maintained at the application layer:
- When a join request is accepted, the handler updates the request status AND adds the member in the same async handler
- When a review is created, the handler creates the review document AND updates the reviewee's cached rating
- Notification creation is fire-and-forget (failures are silently ignored)

### Response DTOs

Each main collection has a corresponding Response type that strips sensitive fields:
- `UserResponse` omits `password_hash`, `reset_token`, and `reset_token_expires_at`
- `CompetitionResponse` adds computed fields (`registration_count`, `interested_count`, viewer-state booleans) and conditionally includes `registrations` based on viewer permissions
