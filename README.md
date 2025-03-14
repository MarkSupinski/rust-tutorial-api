# Rust Task Management API

A modern task management API built with Rust, using Axum for the web framework, SQLx for database operations, and NSQ for message queuing.

## Features

- RESTful API endpoints for task management
- PostgreSQL database with SQLx for type-safe database operations
- NSQ integration for event publishing
- CORS support for cross-origin requests
- Structured logging with tracing
- Docker support for easy deployment

## API Endpoints

- `GET /tasks` - List all tasks
- `POST /tasks` - Create a new task
- `GET /tasks/:id` - Get a specific task
- `PUT /tasks/:id` - Update a task

## Prerequisites

- Rust 1.70 or later
- PostgreSQL
- NSQ (optional, for event publishing)
- Docker and Docker Compose (optional, for containerized deployment)

## Environment Variables

Create a `.env` file in the project root with the following variables:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/tasks
NSQD_URL=127.0.0.1:4150  # Optional, for NSQ integration
```

## Local Development

1. Install dependencies:
   ```bash
   cargo build
   ```

2. Set up the database:
   ```bash
   cargo sqlx migrate run
   ```

3. Run the application:
   ```bash
   cargo run
   ```

The API will be available at `http://localhost:3000`.

## Docker Deployment

1. Build and run with Docker Compose:
   ```bash
   docker-compose up --build
   ```

2. The API will be available at `http://localhost:3000`.

## API Usage Examples

### Create a Task

```bash
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Complete project", "description": "Finish the Rust API tutorial"}'
```

### List Tasks

```bash
curl http://localhost:3000/tasks
```

### Get a Task

```bash
curl http://localhost:3000/tasks/1
```

### Update a Task

```bash
curl -X PUT http://localhost:3000/tasks/1 \
  -H "Content-Type: application/json" \
  -d '{"completed": true}'
```

## Project Structure

```
.
├── src/
│   ├── main.rs      # Application entry point and route handlers
│   └── db.rs        # Database operations
├── migrations/      # SQLx migrations
├── Dockerfile      # Container configuration
├── docker-compose.yml
└── Cargo.toml      # Project dependencies
```

## Dependencies

- `axum`: Web framework
- `sqlx`: Database toolkit
- `tokio`: Async runtime
- `chrono`: Date/time handling
- `serde`: Serialization/deserialization
- `tower-http`: HTTP middleware
- `tracing`: Logging and diagnostics
- `tokio-nsq`: NSQ client for message queuing

## License

MIT 