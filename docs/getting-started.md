# Getting Started

This guide walks through everything needed to run the Teamder backend locally.

---

## Prerequisites

| Requirement | Minimum version | Notes |
|-------------|----------------|-------|
| **Rust toolchain** | 1.75+ (2021 edition) | Install via [rustup.rs](https://rustup.rs/) |
| **MongoDB** | 6.0+ | Local install or Docker; must be accessible at `localhost:27017` |
| **Git** | 2.x | For cloning the repo |

Optional:

- **Docker** -- for containerized deployment (see [Deployment](./deployment.md))
- **mongosh** -- for inspecting the database directly

---

## Clone and Setup

```bash
git clone <repo-url> teamder-sys
cd teamder-sys
```

### Environment Variables

Copy the example environment file and adjust as needed:

```bash
cp .env.example .env
```

The `.env` file contains:

| Variable | Default | Description |
|----------|---------|-------------|
| `MONGODB_URI` | `mongodb://localhost:27017` | MongoDB connection string |
| `DB_NAME` | `teamder` | Name of the MongoDB database |
| `JWT_SECRET` | `teamder-dev-secret-change-in-production` | Secret key for signing JWT tokens. **Change in production.** |
| `RUST_LOG` | `info` | Log level filter (trace, debug, info, warn, error) |
| `ROCKET_ADDRESS` | `127.0.0.1` | Address the server binds to |
| `ROCKET_PORT` | `8000` | Port the server listens on |

### Rocket Configuration

The `Rocket.toml` at the workspace root configures:

- **Port**: 8000
- **Address**: 127.0.0.1
- **Upload limits**: file = 20 MiB, data-form = 22 MiB

---

## Running Locally

### Start MongoDB

If using a local MongoDB installation:

```bash
mongod --dbpath /path/to/data
```

Or with Docker:

```bash
docker run -d --name teamder-mongo -p 27017:27017 mongo:7
```

### Start the API server

```bash
cargo run -p teamder-api
```

On first launch:

1. The server connects to MongoDB and pings to verify
2. The skill catalog is auto-seeded if the `skill_categories` collection is empty (12 categories, 60+ skill tags)
3. The server starts on `http://127.0.0.1:8000`

You should see log output like:

```
INFO Connected to MongoDB database 'teamder'
INFO Seeding skill catalog...
INFO Skill catalog seeded (12 categories)
INFO Rocket has launched from http://127.0.0.1:8000
```

### Verify It Works

```bash
curl http://localhost:8000/api/v1/health
```

Expected response:

```json
{ "status": "ok" }
```

---

## Running with the Frontend

The Next.js frontend (in the sibling `Teamder/` directory) expects the backend at `http://localhost:8000/api/v1`.

```bash
# Terminal 1: Backend
cd teamder-sys
cargo run -p teamder-api

# Terminal 2: Frontend
cd Teamder
npm install
npm run dev
```

The frontend runs on `http://localhost:3000`. CORS is pre-configured to allow requests from `localhost:3000` and `localhost:3001`.

---

## Development Tips

### Faster Compilation

For faster incremental builds during development:

```bash
# Use the nightly toolchain for faster compile times (optional)
rustup override set nightly

# Build only the API crate
cargo build -p teamder-api
```

### Log Levels

Control log verbosity with `RUST_LOG`:

```bash
# Show all debug logs
RUST_LOG=debug cargo run -p teamder-api

# Show only Rocket and Teamder logs
RUST_LOG=teamder_api=debug,rocket=info cargo run -p teamder-api

# Show MongoDB driver queries
RUST_LOG=mongodb=debug cargo run -p teamder-api
```

### Running Tests

```bash
# Run all tests across the workspace
cargo test

# Run tests for a specific crate
cargo test -p teamder-core
```

---

## Common Issues and Troubleshooting

### "Connection refused" on MongoDB

**Symptom**: The server panics at startup with a MongoDB connection error.

**Fix**: Ensure MongoDB is running on the URI specified in `.env`. The default is `mongodb://localhost:27017`.

```bash
# Check if MongoDB is running
mongosh --eval "db.adminCommand('ping')"
```

### Port 8000 already in use

**Symptom**: `Rocket failed to bind to 127.0.0.1:8000`.

**Fix**: Either stop the process using port 8000, or change the port:

```bash
ROCKET_PORT=8001 cargo run -p teamder-api
```

### Upload directory permissions

**Symptom**: File uploads fail with an internal error.

**Fix**: Ensure the `uploads/` directory exists and is writable at the workspace root:

```bash
mkdir -p uploads
```

The server creates subdirectories automatically (`uploads/<user_id>/avatar/`, etc.), but the parent directory must exist.

### JWT secret warning

**Symptom**: Authentication works but tokens are signed with a weak secret.

**Fix**: Always set a strong, random `JWT_SECRET` in production. The default value is only for development. Generate one with:

```bash
openssl rand -hex 64
```

### CORS errors from the frontend

**Symptom**: Browser console shows `Access-Control-Allow-Origin` errors.

**Fix**: The server allows origins `http://localhost:3000`, `http://localhost:3001`, and `https://teamder.watchandy.me`. If you are running the frontend on a different port, you need to add it to the CORS configuration in `crates/teamder-api/src/main.rs`.
