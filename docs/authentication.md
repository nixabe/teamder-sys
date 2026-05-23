# Authentication

The Teamder backend uses JSON Web Tokens (JWT) with HMAC-SHA256 signing for stateless authentication. Passwords are hashed with bcrypt.

---

## JWT Structure

### Claims

```json
{
  "sub": "user-uuid-string",
  "exp": 1234567890
}
```

| Claim | Type | Description |
|-------|------|-------------|
| `sub` | String | The user's `_id` (UUID v4) |
| `exp` | usize | Expiration timestamp (Unix epoch seconds) |

### Configuration

| Parameter | Value |
|-----------|-------|
| Algorithm | HS256 (HMAC-SHA256) |
| Secret | `JWT_SECRET` environment variable |
| Expiry | 30 days from token creation |
| Header | `Authorization: Bearer <token>` |

The signing secret is configured via the `JWT_SECRET` environment variable. The default development value is `teamder-dev-secret-change-in-production`. **Always set a strong, random secret in production.**

### Token Lifecycle

1. Token is created on registration or login
2. Token is valid for 30 days
3. Token is included in every authenticated request as `Authorization: Bearer <token>`
4. Token is verified on each request by the appropriate request guard
5. There is no token refresh mechanism -- clients must re-authenticate after expiry
6. There is no token revocation -- tokens remain valid until they expire

---

## Registration Flow

```
Client                          Server
  |                               |
  |  POST /auth/register          |
  |  { email, password, name }    |
  | ----------------------------> |
  |                               |  1. Validate required fields
  |                               |  2. Check email uniqueness
  |                               |  3. Hash password (bcrypt, cost 12)
  |                               |  4. Generate UUID v4 for user ID
  |                               |  5. Generate initials from name
  |                               |  6. Create user document
  |                               |  7. Create JWT token (30-day expiry)
  |  { token, user }              |
  | <---------------------------- |
```

**Validation**:
- `email`, `password`, and `name` are required (non-empty)
- Duplicate emails return 409 Conflict

**Password hashing**:
- Algorithm: bcrypt
- Cost factor: 12
- The hash is stored in the `password_hash` field
- The plaintext password is never stored

---

## Login Flow

```
Client                          Server
  |                               |
  |  POST /auth/login             |
  |  { email, password }          |
  | ----------------------------> |
  |                               |  1. Find user by email
  |                               |  2. Verify password against hash
  |                               |  3. Create JWT token (30-day expiry)
  |  { token, user }              |
  | <---------------------------- |
```

**Error handling**:
- If the email is not found OR the password does not match, the response is 401 with the message "Invalid email or password" (does not reveal which was wrong)

---

## Password Reset Flow

```
Client                          Server
  |                               |
  |  POST /auth/forgot-password   |
  |  { email }                    |
  | ----------------------------> |
  |                               |  1. Look up user by email
  |                               |  2. Generate 64-char hex reset token
  |                               |  3. Store token + 30-min expiry on user
  |  { success, reset_token }     |
  | <---------------------------- |
  |                               |
  |  POST /auth/reset-password    |
  |  { token, password }          |
  | ----------------------------> |
  |                               |  1. Find user by reset_token
  |                               |  2. Check token has not expired
  |                               |  3. Hash new password (bcrypt, cost 12)
  |                               |  4. Update password_hash
  |                               |  5. Clear reset_token fields
  |  { success: true }            |
  | <---------------------------- |
```

**Security notes**:
- If the email does not exist, `forgot-password` still returns success (prevents email enumeration)
- The reset token is a concatenation of two UUID v4 values (64 hex characters)
- The token expires after 30 minutes
- After successful password reset, the token is cleared and cannot be reused
- In the current implementation, the reset token is returned directly in the API response (suitable for development; in production, it would typically be sent via email)

---

## Request Guards

Request guards are Rocket's mechanism for extracting and validating data from incoming requests before the route handler runs. The Teamder backend defines four guards:

### AuthUser

**Purpose**: Requires a valid JWT token. Extracts the authenticated user's ID.

**Extraction**:
1. Read the `Authorization` header
2. Strip the `Bearer ` prefix
3. Verify the JWT signature and expiry using the `JWT_SECRET`
4. Extract the `sub` claim as `user_id`

**Usage**: Any route that requires authentication.

```rust
#[rocket::get("/users/me")]
pub async fn get_me(auth: AuthUser) -> ... {
    // auth.user_id is the authenticated user's ID
}
```

**Failure**: Returns 401 Unauthorized if the header is missing, the token is invalid, or the token has expired.

### OptionalAuth

**Purpose**: Extracts the authenticated user's ID if a valid token is present, but does not require authentication.

**Extraction**: Same as AuthUser, but returns `OptionalAuth(None)` instead of an error when the token is missing or invalid.

**Usage**: Routes that behave differently for authenticated vs. anonymous users (e.g., computing match scores, showing viewer-specific state).

```rust
#[rocket::get("/users/<id>")]
pub async fn get_user(viewer: OptionalAuth) -> ... {
    // viewer.0 is Some(user_id) if authenticated, None otherwise
}
```

**Failure**: Never fails. Always succeeds with `None` if authentication is absent or invalid.

### AdminUser

**Purpose**: Requires a valid JWT token AND the user must have `is_admin: true` in the database.

**Extraction**:
1. Verify the JWT (same as AuthUser)
2. Fetch the user document from the database
3. Check `user.is_admin == true`

**Usage**: Admin-only routes (dashboard, user management, skill CRUD, competition approval).

```rust
#[rocket::get("/admin/stats")]
pub async fn stats(_admin: AdminUser) -> ... { ... }
```

**Failure**:
- 401 Unauthorized if the token is invalid
- 403 Forbidden if the user is not an admin

### PublisherUser

**Purpose**: Requires a valid JWT token AND the user must have `is_publisher: true` OR `is_admin: true`.

**Extraction**:
1. Verify the JWT (same as AuthUser)
2. Fetch the user document from the database
3. Check `user.is_publisher || user.is_admin`

**Usage**: Publisher routes (creating/editing competitions, viewing registrations).

```rust
#[rocket::post("/competitions")]
pub async fn create_competition(auth: PublisherUser) -> ... { ... }
```

**Failure**:
- 401 Unauthorized if the token is invalid
- 403 Forbidden if the user is neither a publisher nor an admin

---

## Role-Based Access Summary

| Role | Flag | Capabilities |
|------|------|-------------|
| **Regular user** | (default) | Register, login, create/join projects and study groups, send invites, submit reviews, chat, bookmark |
| **Publisher** | `is_publisher: true` | All user capabilities + create/edit competitions, view registrations, submit for review |
| **Admin** | `is_admin: true` | All publisher capabilities + approve/reject competitions, manage users (promote/demote/delete), manage skills, view admin dashboard, export data |

**Promotion**: Admin status and publisher status are toggled by existing admins via `POST /admin/users/<id>/promote` and `POST /admin/users/<id>/publisher`.

---

## Security Considerations

1. **JWT Secret**: The `JWT_SECRET` must be a strong, random value in production. A weak secret allows token forgery.

2. **bcrypt Cost**: The cost factor is 12, which provides a good balance between security and performance. Each hash operation takes approximately 200-400ms.

3. **No Token Revocation**: There is no mechanism to invalidate a token before expiry. If a user's account is compromised, the token remains valid for up to 30 days. Consider implementing a token blacklist or shorter expiry for production use.

4. **Password in Update**: The PATCH `/users/<id>` endpoint accepts a `password` field, which re-hashes and updates the password. This is separate from the `change-password` endpoint (which requires the old password).

5. **CORS**: The server only accepts requests from whitelisted origins. See [Architecture](./architecture.md) for the full list.

6. **WebSocket Auth**: The WebSocket endpoint at `/chat/ws` accepts the JWT as a query parameter (`?token=<jwt>`) because the browser WebSocket API does not support custom headers. This means the token may appear in server access logs.
