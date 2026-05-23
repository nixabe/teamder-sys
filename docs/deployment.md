# Deployment

This guide covers building, containerizing, and deploying the Teamder backend for production.

---

## Production Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `MONGODB_URI` | Yes | MongoDB connection string (e.g., `mongodb+srv://user:pass@cluster/teamder`) |
| `DB_NAME` | Yes | Database name (default: `teamder`) |
| `JWT_SECRET` | Yes | **Must be a strong, random value.** Generate with `openssl rand -hex 64` |
| `RUST_LOG` | No | Log level (recommended: `info` for production) |
| `ROCKET_ADDRESS` | No | Bind address (default: `127.0.0.1`; use `0.0.0.0` in containers) |
| `ROCKET_PORT` | No | Bind port (default: `8000`) |

---

## Building for Release

### Local Build

```bash
cargo build --release -p teamder-api
```

The binary is output to `target/release/teamder-api`. Release builds enable optimizations (LTO, code-gen units, etc.) and are significantly faster than debug builds.

### Running the Release Binary

```bash
# Set environment variables
export MONGODB_URI="mongodb+srv://..."
export DB_NAME="teamder"
export JWT_SECRET="your-strong-random-secret-here"
export RUST_LOG="info"
export ROCKET_ADDRESS="0.0.0.0"
export ROCKET_PORT="8000"

# Run
./target/release/teamder-api
```

---

## Docker

The project includes a multi-stage Dockerfile.

### Dockerfile Overview

**Builder stage** (rust:1.95):
1. Copies the full workspace
2. Runs `cargo build --release`

**Runtime stage** (debian:bookworm-slim):
1. Installs `ca-certificates` (for TLS connections to MongoDB Atlas, etc.)
2. Copies the compiled binary from the builder
3. Exposes port 3000 (note: the Dockerfile exposes 3000, but the default Rocket config uses 8000; set `ROCKET_PORT` accordingly)
4. Runs the binary

### Building the Docker Image

```bash
docker build -t teamder-api .
```

### Running the Container

```bash
docker run -d \
  --name teamder-api \
  -p 8000:8000 \
  -e MONGODB_URI="mongodb://host.docker.internal:27017" \
  -e DB_NAME="teamder" \
  -e JWT_SECRET="your-strong-random-secret-here" \
  -e RUST_LOG="info" \
  -e ROCKET_ADDRESS="0.0.0.0" \
  -e ROCKET_PORT="8000" \
  -v $(pwd)/uploads:/app/uploads \
  teamder-api
```

**Important notes**:
- Set `ROCKET_ADDRESS=0.0.0.0` so Rocket binds to all interfaces inside the container
- Mount the `uploads/` volume to persist user-uploaded files across container restarts
- Use `host.docker.internal` to connect to MongoDB on the host machine (Docker Desktop); for Linux, use `--network host` or the host's IP address

### Docker Compose Example

```yaml
version: "3.8"

services:
  mongo:
    image: mongo:7
    ports:
      - "27017:27017"
    volumes:
      - mongo-data:/data/db

  api:
    build: .
    ports:
      - "8000:8000"
    environment:
      MONGODB_URI: mongodb://mongo:27017
      DB_NAME: teamder
      JWT_SECRET: change-this-to-a-strong-random-secret
      RUST_LOG: info
      ROCKET_ADDRESS: "0.0.0.0"
      ROCKET_PORT: "8000"
    volumes:
      - ./uploads:/app/uploads
    depends_on:
      - mongo

volumes:
  mongo-data:
```

---

## MongoDB Setup

### Requirements

- MongoDB 6.0+ recommended
- No special configuration required; the application creates collections on first write
- The skill catalog is auto-seeded on first startup if the `skill_categories` collection is empty

### Recommended Indexes

While the application works without manually created indexes, the following improve query performance:

```javascript
// Users
db.users.createIndex({ "email": 1 }, { unique: true });
db.users.createIndex({ "name": "text", "email": "text" });

// Projects
db.projects.createIndex({ "lead_user_id": 1 });
db.projects.createIndex({ "team.user_id": 1 });
db.projects.createIndex({ "name": "text", "description": "text" });

// Competitions
db.competitions.createIndex({ "publisher_id": 1 });
db.competitions.createIndex({ "publish_status": 1 });
db.competitions.createIndex({ "name": "text", "description": "text" });

// Competition Teams
db.competition_teams.createIndex({ "competition_id": 1 });
db.competition_teams.createIndex({ "lead_user_id": 1 });

// Study Groups
db.study_groups.createIndex({ "members.user_id": 1 });
db.study_groups.createIndex({ "name": "text" });

// Messages
db.messages.createIndex({ "from_user_id": 1, "to_user_id": 1, "created_at": -1 });

// Notifications
db.notifications.createIndex({ "user_id": 1, "created_at": -1 });

// Bookmarks
db.bookmarks.createIndex({ "user_id": 1, "kind": 1, "entity_id": 1 }, { unique: true });

// Join Requests
db.join_requests.createIndex({ "owner_id": 1, "status": 1 });
db.join_requests.createIndex({ "from_user_id": 1, "entity_id": 1, "status": 1 });

// Invites
db.invites.createIndex({ "to_user_id": 1, "status": 1 });

// Peer Reviews
db.peer_reviews.createIndex({ "reviewee_id": 1 });
```

### MongoDB Atlas

For production, MongoDB Atlas is recommended:

1. Create a cluster (M10+ for production workloads)
2. Create a database user with read/write access to the `teamder` database
3. Whitelist the server's IP address (or use `0.0.0.0/0` for containers with dynamic IPs)
4. Set `MONGODB_URI` to the Atlas connection string (SRV format)

---

## File Storage

### Uploads Directory

User-uploaded files are stored on the local filesystem at `uploads/` relative to the working directory. The directory structure is:

```
uploads/
  <user-id>/
    avatar/       Avatar images
    portfolio/    Portfolio files
    resume/       Resume files
    banners/      Banner images
    applications/ Application attachments
```

**Production considerations**:

1. **Persistence**: Mount the `uploads/` directory as a Docker volume or use a persistent disk
2. **Backup**: Include the `uploads/` directory in backup procedures
3. **CDN**: For production with many users, consider serving uploads through a CDN or object storage (S3, GCS) instead of Rocket's built-in FileServer
4. **Size limits**: Rocket.toml sets file upload limits at 20 MiB per file, 22 MiB per form

### Static File Serving

Uploaded files are served by Rocket's `FileServer` at the `/uploads/` URL path:

```
GET /uploads/<user-id>/avatar/<filename>
```

There is no authentication on file access. Files are publicly accessible if the URL is known. File URLs use UUID v4 filenames, providing obscurity but not security.

---

## CORS Configuration

The production CORS whitelist includes:

```
http://localhost:3000
http://localhost:3001
https://teamder.watchandy.me
```

To add or change allowed origins, modify the `allowed_origins` list in `crates/teamder-api/src/main.rs`:

```rust
let allowed_origins = AllowedOrigins::some_exact(&[
    "http://localhost:3000",
    "http://localhost:3001",
    "https://teamder.watchandy.me",
    "https://your-production-domain.com",  // Add your domain here
]);
```

This requires a rebuild. For more flexible configuration, consider reading allowed origins from an environment variable.

---

## Security Considerations

### JWT Secret

- **Must** be changed from the development default
- Generate a cryptographically random value: `openssl rand -hex 64`
- Store securely (environment variable, secrets manager)
- Rotating the secret invalidates all existing tokens

### bcrypt Cost Factor

The password hashing cost is hardcoded at 12. This is appropriate for most deployments. Increasing the cost to 13 or 14 doubles or quadruples the time per hash, which improves security at the expense of login/registration latency.

### Rate Limiting

The current implementation does **not** include rate limiting. For production, consider:

- A reverse proxy (Nginx, Caddy) with rate limiting rules
- Rate limiting on authentication endpoints to prevent brute-force attacks
- Rate limiting on file upload endpoints to prevent abuse

### HTTPS

Rocket does not handle TLS termination in this configuration. Use a reverse proxy:

```
Client -> Nginx/Caddy (TLS) -> Rocket (HTTP on localhost:8000)
```

### File Upload Security

- Filenames are sanitized (non-alphanumeric characters stripped)
- Files are renamed to UUID v4 to prevent path traversal and name collisions
- The delete endpoint validates that the path belongs to the authenticated user and does not contain `..`
- There is no virus scanning or file type validation beyond extension extraction

### Database Security

- MongoDB connection should use authentication in production
- Use MongoDB Atlas with network-level access control
- The application does not use MongoDB transactions; consider implications for consistency

---

## Reverse Proxy Configuration

### Nginx Example

```nginx
server {
    listen 443 ssl;
    server_name teamder-api.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    client_max_body_size 25M;

    location / {
        proxy_pass http://127.0.0.1:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket support for chat
    location /api/v1/chat/ws {
        proxy_pass http://127.0.0.1:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400;
    }
}
```

### Key Points

- Set `client_max_body_size` to at least 25M to accommodate file uploads (Rocket's limit is 22 MiB for form data)
- The WebSocket endpoint requires HTTP/1.1 upgrade headers
- Set a long `proxy_read_timeout` for WebSocket connections to prevent premature disconnection

---

## Health Check

Use the health endpoint for load balancer and container orchestration health checks:

```
GET /api/v1/health
```

Returns `200 OK` with `{ "status": "ok" }` when the server is running. Note: this endpoint does not verify MongoDB connectivity; it only confirms the Rocket server is accepting requests.

---

## Monitoring

### Logging

The server uses the `tracing` crate with `tracing-subscriber`. Control log output with `RUST_LOG`:

```bash
# Production recommended
RUST_LOG=info

# Debugging
RUST_LOG=teamder_api=debug,teamder_db=debug,rocket=info

# Verbose (includes MongoDB driver logs)
RUST_LOG=debug
```

Logs are written to stdout in a human-readable format. For structured logging in production, consider configuring `tracing-subscriber` with a JSON formatter.
