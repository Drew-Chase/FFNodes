# FFNodes
Have you ever wanted to encode many video or audio files at once, across multiple devices? FFNodes is a Rust-based tool designed to simplify this process by leveraging the power of ffmpeg and distributed computing.

## Table of Contents
- [Features](#features)
- [Architecture](#architecture)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [API Documentation](#api-documentation)
- [Technology](#technology)

## Features
- Distributed encoding across multiple devices
- GUID-based authentication for secure client connections
- Priority-based job queue (largest and most complex files first)
- Smart file re-probing (only probes when file modification time changes)
- Automatic file system watching with batch processing
- Client heartbeat monitoring with automatic job reassignment on timeout
- FFmpeg template system with hardware acceleration support
- Real-time WebSocket progress monitoring
- Automatic job recovery and retry with exponential backoff
- Support for 47+ video/audio formats

## Architecture

FFNodes uses a client-server architecture for distributed video encoding:

### System Components

1. **Server** - Central coordinator that manages:
   - Media file discovery and metadata extraction
   - Job queue with priority-based assignment
   - Client registration and heartbeat monitoring
   - File system watching for automatic job creation
   - REST API and WebSocket endpoints

2. **Clients** - Worker nodes that:
   - Connect to the server via GUID authentication
   - Request encoding jobs from the queue
   - Execute FFmpeg with hardware acceleration
   - Report progress and completion status
   - Send periodic heartbeats

### Core Subsystems

#### Media File Scanner
- Recursively scans configured directories for supported video/audio files
- Extracts metadata using FFprobe (duration, resolution, bitrate, codec, etc.)
- Calculates encoding complexity: `(width × height) × bitrate × duration`
- Smart re-probing: only re-probes files when `last_modified` timestamp changes
- Processes up to 10 files concurrently

#### File Watcher
- Monitors configured directories for file system changes using `notify`
- Batch processing every 30 seconds (configurable) to reduce overhead
- Automatically adds new files to database and creates encoding jobs
- Removes deleted files from database and cancels associated jobs

#### Job Queue
- Priority-based queue: `priority = file_size × encoding_complexity / (retry_count + 1)`
- Largest and most complex files encode first
- Failed jobs automatically deprioritized based on retry count
- In-memory cache with SQLite persistence
- Job states: `pending` → `assigned` → `in_progress` → `completed`/`failed`

#### Client Manager
- GUID-based authentication (server generates UUID on first launch)
- Tracks client metadata: display name, computer name, connection time
- Heartbeat monitoring with configurable timeout (default 300 seconds)
- Automatic disconnection detection and job reassignment

#### Job Scheduler
- Background task runs every 60 seconds
- Detects stale jobs (assigned but no progress for > timeout)
- Automatically requeues stale jobs when clients disconnect or timeout
- Increments retry count and recalculates priority

#### Template System
- Server provides FFmpeg command template to clients
- Template format: `-c:v h264{HWACCEL_CODE} -preset medium -crf 23 -i {INPUT} {OUTPUT}`
- Clients specify hardware acceleration:
  - `_nvenc` - NVIDIA NVENC
  - `_amf` - AMD AMF
  - `_qsv` - Intel Quick Sync
  - `` (empty) - Software encoding
- Template parser performs simple string replacement

### Data Flow

1. **Server Startup**:
   - Load configuration (or generate with new GUID)
   - Initialize SQLite database with WAL mode
   - Start job scheduler for stale job monitoring
   - Start file watcher for automatic discovery
   - Perform initial media file scan
   - Start HTTP server and WebSocket endpoint

2. **Client Connection**:
   - Client sends handshake with server GUID, display name, computer name
   - Server validates GUID and registers client
   - Server returns auth token (client ID) and FFmpeg template

3. **Job Assignment**:
   - Client requests job with client ID
   - Server retrieves highest priority pending job
   - Server assigns job to client and updates state
   - Server returns job details with input/output paths

4. **Job Execution**:
   - Client starts job (updates state to `in_progress`)
   - Client executes FFmpeg with template and hardware acceleration
   - Client sends periodic progress updates (frame, fps, speed)
   - Client sends heartbeats to maintain connection

5. **Job Completion**:
   - Client reports completion with output metadata (size, bitrate)
   - OR client reports failure with error message (increments retry count)
   - Server updates job state and media file record
   - Job removed from queue or requeued if failed

6. **Monitoring**:
   - WebSocket connection provides real-time progress updates
   - REST endpoints provide system status, client list, active jobs

### Database Schema

**media_files** table:
- Stores file metadata and processing state
- Primary key: `path` (file path)
- Fields: size, bitrate, duration, resolution, frames, encoding_complexity, retry_count, processed, last_modified

**encoding_jobs** table:
- Stores job queue state
- Primary key: `id` (UUID)
- Fields: media_file_path, status, priority, assigned_client, timestamps, output details, error_message
- Foreign keys: media_files(path), clients(id)

**clients** table:
- Stores connected client information
- Primary key: `id` (UUID)
- Fields: display_name, computer_name, connected_at, last_heartbeat, disconnected_at

**encoding_progress** table:
- Stores real-time encoding progress
- Primary key: `job_id`
- Fields: frame, fps, bitrate, speed, updated_at
- Foreign key: encoding_jobs(id)

## Installation
FFNodes is available on Windows, Linux, and MacOS, for both x86 and ARM architectures.
> **Note:** FFNodes requires ffmpeg to be installed on your system and accessible from the command line. You can find ffmpeg [here](https://ffmpeg.org/download.html).
### Server
The server is a simple REST API that can be used to manage encoding jobs. You can Install the server in a few ways:
1. Using Cargo Install: `cargo install ffnodes-server --git https://github.com/Drew-Chase/FFNodes.git`
2. Using our installer script: `curl -fsSL https://raw.githubusercontent.com/Drew-Chase/FFNodes/main/install.sh | bash` (linux only) 
3. Using our MSI installer found in the [releases page](https://github.com/Drew-Chase/FFNodes/releases) (windows only)
4. Building from source: `git clone https://github.com/Drew-Chase/FFNodes.git && cd FFNodes && cargo build --release --bin ffnodes-server`.
5. Using Docker: `docker run -p 8080:8080 ghcr.io/drew-chase/ffnodes-server:latest`
6. Using Homebrew: `brew install drew-chase/tap/ffnodes-server` (macos only)
7. Using winget: `winget install FFNodes.FFNodesServer` (windows only)
### Client
The client is a GUI application that can be used to manage encoding jobs. You can install the client in a few ways:
1. Using Cargo Install: `cargo install ffnodes-client --git https://github.com/Drew-Chase/FFNodes.git`
2. Using our appimage file found in the [releases page](https://github.com/Drew-Chase/FFNodes/releases) (linux only)
3. Using our MSI installer found in the [releases page](https://github.com/Drew-Chase/FFNodes/releases) (windows only)
4. Building from source: `git clone https://github.com/Drew-Chase/FFNodes.git && cd FFNodes && cargo build --release --bin ffnodes-client`.
5. Using Homebrew: `brew install drew-chase/tap/ffnodes-client` (macos only)
6. Using winget: `winget install FFNodes.FFNodesClient` (windows only)

## Configuration

The server uses a `config.json` file for configuration. On first launch, it will generate a default configuration with a new server GUID.

### Configuration File Location
- Debug mode: `target/dev-env/server/config.json`
- Production: `./config.json` (in the same directory as the executable)

### Configuration Options

```json
{
  "port": 8080,
  "ffmpeg": "path/to/ffmpeg",
  "ffprobe": "path/to/ffprobe",
  "watch_directories": [
    "/path/to/video/library1",
    "/path/to/video/library2"
  ],
  "server_guid": "550e8400-e29b-41d4-a716-446655440000",
  "ffmpeg_template": "-c:v h264{HWACCEL_CODE} -preset medium -crf 23 -i {INPUT} {OUTPUT}",
  "client_timeout_seconds": 300,
  "notify_batch_interval_seconds": 30,
  "max_concurrent_jobs_per_client": 4
}
```

### Configuration Fields

- **port** - HTTP server port (default: 8080)
- **ffmpeg** - Path to FFmpeg binary (auto-detected if in PATH)
- **ffprobe** - Path to FFprobe binary (auto-detected if in PATH)
- **watch_directories** - Array of directories to monitor for media files
- **server_guid** - Unique server identifier for client authentication (auto-generated)
- **ffmpeg_template** - FFmpeg command template with placeholders:
  - `{HWACCEL_CODE}` - Hardware acceleration suffix (e.g., `_nvenc`, `_amf`, `_qsv`, or empty)
  - `{INPUT}` - Input file path
  - `{OUTPUT}` - Output file path
- **client_timeout_seconds** - Time before reassigning jobs from unresponsive clients (default: 300)
- **notify_batch_interval_seconds** - File watcher batch processing interval (default: 30)
- **max_concurrent_jobs_per_client** - Maximum jobs per client (default: 4)

### Supported Video Formats

FFNodes automatically detects and processes these formats:
webm, mkv, flv, vob, ogv, ogg, rrc, gifv, mng, mov, avi, qt, wmv, yuv, rm, asf, amv, mp4, m4p, m4v, mpg, mp2, mpeg, mpe, mpv, m4v, svi, 3gp, 3g2, mxf, roq, nsv, flv, f4v, f4p, f4a, f4b, mts, m2ts, ts

## Usage

### Starting the Server

1. Launch the server:
   ```bash
   ffnodes-server
   ```

2. On first launch, the server will:
   - Generate `config.json` with a new GUID
   - Create `app.db` SQLite database
   - Display the server GUID in logs (needed for client connections)
   - Start the HTTP server on configured port

3. The server will log:
   ```
   Server GUID: 550e8400-e29b-41d4-a716-446655440000
   Starting production server at http://127.0.0.1:8080...
   ```

### Client Workflow

1. **Handshake**: Connect to the server with GUID
   ```
   POST /api/handshake
   {
     "server_guid": "550e8400-e29b-41d4-a716-446655440000",
     "display_name": "My Workstation",
     "computer_name": "DESKTOP-ABC123"
   }
   ```
   Response includes auth token and FFmpeg template.

2. **Request Job**: Poll for available jobs
   ```
   POST /api/jobs/request/{client_id}
   ```
   Returns highest priority job or 204 No Content if queue is empty.

3. **Start Job**: Mark job as in progress
   ```
   POST /api/jobs/{job_id}/start
   ```

4. **Report Progress**: Send periodic updates
   ```
   POST /api/jobs/{job_id}/progress
   {
     "frame": 1250,
     "fps": 45.2,
     "bitrate": "2500kbits/s",
     "speed": "1.5x"
   }
   ```

5. **Complete Job**: Report success or failure
   ```
   POST /api/jobs/{job_id}/complete
   {
     "output_size": 125829120,
     "output_bitrate": 2500000
   }
   ```
   or
   ```
   POST /api/jobs/{job_id}/fail
   {
     "error": "FFmpeg process exited with code 1"
   }
   ```

6. **Heartbeat**: Maintain connection
   ```
   POST /api/heartbeat/{client_id}
   ```
   Send every 60 seconds to avoid timeout.

### Monitoring

- **System Status**: `GET /api/status`
  ```json
  {
    "total_media_files": 1523,
    "pending_jobs": 45,
    "active_jobs": 8,
    "connected_clients": 3
  }
  ```

- **Connected Clients**: `GET /api/clients`
  ```json
  [
    {
      "id": "client-uuid",
      "display_name": "My Workstation",
      "computer_name": "DESKTOP-ABC123",
      "active_jobs": 2,
      "connected_at": "2025-01-15T10:30:00Z"
    }
  ]
  ```

- **Active Jobs**: `GET /api/jobs/active`
  ```json
  [
    {
      "id": "job-uuid",
      "media_file_path": "/videos/movie.mp4",
      "status": "in_progress",
      "assigned_client": "client-uuid",
      "priority": 15000000,
      "started_at": "2025-01-15T10:35:00Z"
    }
  ]
  ```

- **Real-time Progress**: WebSocket at `ws://localhost:8080/api/ws/progress`
  - Receives JSON events for job assignments, progress updates, completions, and client connections

## API Documentation

For complete API specification with request/response schemas, see [API.md](API.md).

### API Endpoints Summary

**Authentication**
- `POST /api/handshake` - Register client with server GUID

**Job Management**
- `POST /api/jobs/request/{client_id}` - Request next job
- `POST /api/jobs/{job_id}/start` - Mark job as started
- `POST /api/jobs/{job_id}/progress` - Update progress
- `POST /api/jobs/{job_id}/complete` - Mark job complete
- `POST /api/jobs/{job_id}/fail` - Mark job failed
- `GET /api/jobs/active` - Get all active jobs

**Monitoring**
- `GET /api/status` - Get system status
- `GET /api/clients` - Get connected clients
- `POST /api/heartbeat/{client_id}` - Send heartbeat

**WebSocket**
- `GET /api/ws/progress` - Real-time progress updates

## Technology
FFNodes is built using Rust for performance and safety.
### Server
The server uses the following technologies:
- [Actix Web](https://actix.rs/) - For the REST API
- [SQLX](https://crates.io/crates/sqlx) with SQLite - For storing job metadata and found media files
- [Tokio](https://tokio.rs/) - For asynchronous tasks
- [Notify](https://crates.io/crates/notify) - For watching for media files
### Client
The client uses the following technologies:
- [Tauri](https://tauri.app/)
- [React](https://reactjs.org/)
- [Typescript](https://www.typescriptlang.org/)
- [TailwindCSS](https://tailwindcss.com/)
- [HeroUI](https://heroui.com)
- [React Router](https://reactrouter.com/)